//! # Nova Bookmarks Manager
//!
//! A comprehensive bookmark management system with folder hierarchy support,
//! import/export capabilities, and cross-device synchronization.
//!
//! ## Features
//!
//! - **Folder-based organization**: Nested folder structure for organizing bookmarks
//! - **Import/Export**: Support for HTML (Netscape bookmark format) and JSON
//! - **Sync engine**: Incremental sync with conflict resolution
//! - **Search**: Full-text search across all bookmark fields
//! - **Favorites**: Quick-access pinned bookmarks
//!
//! ## Architecture
//!
//! Bookmarks are stored in a tree structure where each node is either a
//! [`BookmarkFolder`] or a [`BookmarkEntry`]. The [`BookmarkManager`] provides
//! the primary API for all operations.

use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur during bookmark operations.
#[derive(Error, Debug)]
pub enum BookmarkError {
    /// A bookmark with the given ID was not found.
    #[error("bookmark not found: {0}")]
    NotFound(String),

    /// A folder with the given ID was not found.
    #[error("folder not found: {0}")]
    FolderNotFound(String),

    /// A duplicate bookmark or folder already exists at the target location.
    #[error("duplicate entry: {0}")]
    DuplicateEntry(String),

    /// I/O error during import/export or persistence.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization or deserialization failed.
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// The provided URL is malformed.
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    /// Sync conflict could not be resolved automatically.
    #[error("sync conflict: {0}")]
    SyncConflict(String),

    /// A generic catch-all error.
    #[error("bookmark error: {0}")]
    Other(String),
}

/// Convenience type alias for results from this module.
pub type Result<T> = std::result::Result<T, BookmarkError>;

// ---------------------------------------------------------------------------
// Data models
// ---------------------------------------------------------------------------

/// A single bookmark pointing to a URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkEntry {
    /// Unique identifier for this bookmark.
    pub id: String,

    /// Display title of the bookmark.
    pub title: String,

    /// Target URL.
    pub url: String,

    /// Optional favicon URL or data URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,

    /// Optional user-defined tags for categorization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Timestamp when the bookmark was created.
    pub created_at: DateTime<Utc>,

    /// Timestamp when the bookmark was last modified.
    pub updated_at: DateTime<Utc>,

    /// Timestamp of the last visit, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_visited: Option<DateTime<Utc>>,

    /// Whether this bookmark is pinned to the favorites bar.
    #[serde(default)]
    pub is_favorite: bool,

    /// Parent folder ID. `None` means it lives at the root level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

/// A folder that can contain other folders and bookmarks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkFolder {
    /// Unique identifier for this folder.
    pub id: String,

    /// Display name of the folder.
    pub name: String,

    /// Parent folder ID. `None` means it lives at the root level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// IDs of child folders, in display order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,

    /// IDs of bookmarks directly in this folder, in display order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bookmarks: Vec<String>,

    /// Timestamp when the folder was created.
    pub created_at: DateTime<Utc>,

    /// Timestamp when the folder was last modified.
    pub updated_at: DateTime<Utc>,
}

/// The complete bookmark data model used for import/export and sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkStore {
    /// Version of the data format.
    pub version: u32,

    /// All bookmarks keyed by ID.
    pub entries: HashMap<String, BookmarkEntry>,

    /// All folders keyed by ID.
    pub folders: HashMap<String, BookmarkFolder>,

    /// IDs of top-level folders, in display order.
    pub root_folders: Vec<String>,

    /// IDs of bookmarks at the root level, in display order.
    pub root_bookmarks: Vec<String>,

    /// Timestamp of the last sync.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced: Option<DateTime<Utc>>,
}

/// Represents the format to use for bookmark import/export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON format (native, preserves all metadata).
    Json,
    /// Netscape Bookmark File Format (HTML), widely compatible.
    Html,
}

/// The result of a sync operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Number of entries added during sync.
    pub added: usize,
    /// Number of entries updated during sync.
    pub updated: usize,
    /// Number of conflicts that were automatically resolved.
    pub conflicts_resolved: usize,
    /// Number of conflicts that require manual resolution.
    pub conflicts_remaining: usize,
    /// Timestamp of the sync.
    pub synced_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Bookmark manager
// ---------------------------------------------------------------------------

/// The main bookmark manager providing all bookmark-related operations.
///
/// # Examples
///
/// ```no_run
/// use nova_features::bookmarks::BookmarkManager;
///
/// let mut manager = BookmarkManager::new();
/// let entry = manager.add_bookmark(
///     "Nova Browser",
///     "https://nova.browser",
///     None,
///     vec!["browser".into()],
/// );
/// ```
pub struct BookmarkManager {
    store: BookmarkStore,
    /// Path to the on-disk persistence file.
    storage_path: Option<PathBuf>,
}

impl BookmarkManager {
    /// Creates a new, empty bookmark manager.
    pub fn new() -> Self {
        info!("Initializing bookmark manager");
        Self {
            store: BookmarkStore {
                version: 1,
                entries: HashMap::new(),
                folders: HashMap::new(),
                root_folders: Vec::new(),
                root_bookmarks: Vec::new(),
                last_synced: None,
            },
            storage_path: None,
        }
    }

    /// Creates a new bookmark manager that persists to the given path.
    pub fn with_persistence(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let mut manager = Self::new();
        manager.storage_path = Some(path.clone());
        if path.exists() {
            match manager.load_from_disk() {
                Ok(_) => info!("Loaded bookmarks from disk: {:?}", path),
                Err(e) => warn!("Failed to load bookmarks from disk: {}", e),
            }
        }
        manager
    }

    // -----------------------------------------------------------------------
    // Bookmark CRUD
    // -----------------------------------------------------------------------

    /// Adds a new bookmark and returns the created entry.
    ///
    /// # Arguments
    ///
    /// * `title` - Display title for the bookmark.
    /// * `url` - Target URL.
    /// * `parent_id` - Optional parent folder ID.
    /// * `tags` - Optional tags for categorization.
    pub fn add_bookmark(
        &mut self,
        title: &str,
        url: &str,
        parent_id: Option<&str>,
        tags: Vec<String>,
    ) -> Result<BookmarkEntry> {
        let now = Utc::now();
        let id = Uuid::new_v4().to_string();

        let entry = BookmarkEntry {
            id: id.clone(),
            title: title.to_string(),
            url: url.to_string(),
            favicon: None,
            tags,
            created_at: now,
            updated_at: now,
            last_visited: None,
            is_favorite: false,
            parent_id: parent_id.map(|s| s.to_string()),
        };

        debug!("Adding bookmark: {} -> {}", title, url);

        if let Some(ref pid) = entry.parent_id {
            let folder = self
                .store
                .folders
                .get_mut(pid)
                .ok_or_else(|| BookmarkError::FolderNotFound(pid.clone()))?;
            folder.bookmarks.push(id.clone());
            folder.updated_at = now;
        } else {
            self.store.root_bookmarks.push(id.clone());
        }

        self.store.entries.insert(id, entry.clone());
        self.maybe_persist()?;
        info!("Bookmark added: {}", title);
        Ok(entry)
    }

    /// Removes a bookmark by ID, including its entry from any parent folder.
    pub fn remove_bookmark(&mut self, id: &str) -> Result<()> {
        let entry = self
            .store
            .entries
            .remove(id)
            .ok_or_else(|| BookmarkError::NotFound(id.to_string()))?;

        // Remove from parent folder or root
        if let Some(ref pid) = entry.parent_id {
            if let Some(folder) = self.store.folders.get_mut(pid) {
                folder.bookmarks.retain(|bid| bid != id);
                folder.updated_at = Utc::now();
            }
        } else {
            self.store.root_bookmarks.retain(|bid| bid != id);
        }

        debug!("Removed bookmark: {}", id);
        self.maybe_persist()?;
        Ok(())
    }

    /// Updates an existing bookmark's fields. Only the provided fields are changed.
    pub fn update_bookmark(
        &mut self,
        id: &str,
        title: Option<&str>,
        url: Option<&str>,
        tags: Option<Vec<String>>,
        is_favorite: Option<bool>,
    ) -> Result<BookmarkEntry> {
        let entry = self
            .store
            .entries
            .get_mut(id)
            .ok_or_else(|| BookmarkError::NotFound(id.to_string()))?;

        if let Some(t) = title {
            entry.title = t.to_string();
        }
        if let Some(u) = url {
            entry.url = u.to_string();
        }
        if let Some(t) = tags {
            entry.tags = t;
        }
        if let Some(f) = is_favorite {
            entry.is_favorite = f;
        }
        entry.updated_at = Utc::now();

        let updated = entry.clone();
        debug!("Updated bookmark: {}", id);
        self.maybe_persist()?;
        Ok(updated)
    }

    /// Moves a bookmark to a different folder (or to root if `new_parent_id` is `None`).
    pub fn move_bookmark(&mut self, id: &str, new_parent_id: Option<&str>) -> Result<()> {
        let entry = self
            .store
            .entries
            .get_mut(id)
            .ok_or_else(|| BookmarkError::NotFound(id.to_string()))?;

        // Remove from old parent
        if let Some(ref old_pid) = entry.parent_id {
            if let Some(folder) = self.store.folders.get_mut(old_pid) {
                folder.bookmarks.retain(|bid| bid != id);
            }
        } else {
            self.store.root_bookmarks.retain(|bid| bid != id);
        }

        // Add to new parent
        if let Some(new_pid) = new_parent_id {
            let folder = self
                .store
                .folders
                .get_mut(new_pid)
                .ok_or_else(|| BookmarkError::FolderNotFound(new_pid.to_string()))?;
            folder.bookmarks.push(id.to_string());
            folder.updated_at = Utc::now();
            entry.parent_id = Some(new_pid.to_string());
        } else {
            self.store.root_bookmarks.push(id.to_string());
            entry.parent_id = None;
        }

        entry.updated_at = Utc::now();
        debug!("Moved bookmark {} to {:?}", id, new_parent_id);
        self.maybe_persist()?;
        Ok(())
    }

    /// Retrieves a bookmark by its ID.
    pub fn get_bookmark(&self, id: &str) -> Option<&BookmarkEntry> {
        self.store.entries.get(id)
    }

    /// Returns all bookmarks as a slice.
    pub fn all_bookmarks(&self) -> Vec<&BookmarkEntry> {
        self.store.entries.values().collect()
    }

    /// Returns all favorite (pinned) bookmarks.
    pub fn favorites(&self) -> Vec<&BookmarkEntry> {
        self.store
            .entries
            .values()
            .filter(|e| e.is_favorite)
            .collect()
    }

    /// Searches bookmarks by title, URL, or tags.
    pub fn search(&self, query: &str) -> Vec<&BookmarkEntry> {
        let query = query.to_lowercase();
        self.store
            .entries
            .values()
            .filter(|e| {
                e.title.to_lowercase().contains(&query)
                    || e.url.to_lowercase().contains(&query)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&query))
            })
            .collect()
    }

    /// Records a visit to a bookmark, updating its `last_visited` timestamp.
    pub fn record_visit(&mut self, id: &str) -> Result<()> {
        let entry = self
            .store
            .entries
            .get_mut(id)
            .ok_or_else(|| BookmarkError::NotFound(id.to_string()))?;
        entry.last_visited = Some(Utc::now());
        self.maybe_persist()?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Folder CRUD
    // -----------------------------------------------------------------------

    /// Creates a new folder.
    pub fn create_folder(
        &mut self,
        name: &str,
        parent_id: Option<&str>,
    ) -> Result<BookmarkFolder> {
        let now = Utc::now();
        let id = Uuid::new_v4().to_string();

        let folder = BookmarkFolder {
            id: id.clone(),
            name: name.to_string(),
            parent_id: parent_id.map(|s| s.to_string()),
            children: Vec::new(),
            bookmarks: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        if let Some(ref pid) = folder.parent_id {
            let parent = self
                .store
                .folders
                .get_mut(pid)
                .ok_or_else(|| BookmarkError::FolderNotFound(pid.clone()))?;
            parent.children.push(id.clone());
            parent.updated_at = now;
        } else {
            self.store.root_folders.push(id.clone());
        }

        debug!("Created folder: {}", name);
        self.store.folders.insert(id, folder.clone());
        self.maybe_persist()?;
        Ok(folder)
    }

    /// Removes a folder and all its contents recursively.
    pub fn remove_folder(&mut self, id: &str) -> Result<()> {
        let folder = self
            .store
            .folders
            .remove(id)
            .ok_or_else(|| BookmarkError::FolderNotFound(id.to_string()))?;

        // Recursively remove children
        for child_id in &folder.children {
            let _ = self.remove_folder(child_id);
        }

        // Remove bookmarks in this folder
        for bookmark_id in &folder.bookmarks {
            self.store.entries.remove(bookmark_id);
        }

        // Remove from parent
        if let Some(ref pid) = folder.parent_id {
            if let Some(parent) = self.store.folders.get_mut(pid) {
                parent.children.retain(|cid| cid != id);
            }
        } else {
            self.store.root_folders.retain(|fid| fid != id);
        }

        debug!("Removed folder and contents: {}", id);
        self.maybe_persist()?;
        Ok(())
    }

    /// Renames a folder.
    pub fn rename_folder(&mut self, id: &str, new_name: &str) -> Result<()> {
        let folder = self
            .store
            .folders
            .get_mut(id)
            .ok_or_else(|| BookmarkError::FolderNotFound(id.to_string()))?;
        folder.name = new_name.to_string();
        folder.updated_at = Utc::now();
        debug!("Renamed folder {} to {}", id, new_name);
        self.maybe_persist()?;
        Ok(())
    }

    /// Returns a reference to a folder.
    pub fn get_folder(&self, id: &str) -> Option<&BookmarkFolder> {
        self.store.folders.get(id)
    }

    /// Returns all folders.
    pub fn all_folders(&self) -> Vec<&BookmarkFolder> {
        self.store.folders.values().collect()
    }

    /// Returns the bookmarks within a specific folder.
    pub fn bookmarks_in_folder(&self, folder_id: &str) -> Result<Vec<&BookmarkEntry>> {
        let folder = self
            .store
            .folders
            .get(folder_id)
            .ok_or_else(|| BookmarkError::FolderNotFound(folder_id.to_string()))?;
        let entries: Vec<&BookmarkEntry> = folder
            .bookmarks
            .iter()
            .filter_map(|bid| self.store.entries.get(bid))
            .collect();
        Ok(entries)
    }

    // -----------------------------------------------------------------------
    // Import / Export
    // -----------------------------------------------------------------------

    /// Exports all bookmarks to the given file path in the specified format.
    pub fn export(&self, path: &Path, format: ExportFormat) -> Result<()> {
        match format {
            ExportFormat::Json => self.export_json(path),
            ExportFormat::Html => self.export_html(path),
        }
    }

    /// Imports bookmarks from the given file path, auto-detecting the format.
    pub fn import(&mut self, path: &Path) -> Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let count = if content.trim_start().starts_with("<!DOCTYPE NETSCAPE-Bookmark-file-1>")
            || content.trim_start().starts_with("<META")
        {
            self.import_html(&content)?
        } else {
            self.import_json(&content)?
        };
        self.maybe_persist()?;
        info!("Imported {} bookmarks from {:?}", count, path);
        Ok(count)
    }

    /// Exports to JSON format.
    fn export_json(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.store)?;
        std::fs::write(path, json)?;
        info!("Exported bookmarks to JSON: {:?}", path);
        Ok(())
    }

    /// Imports from JSON content and returns the number of entries imported.
    fn import_json(&mut self, content: &str) -> Result<usize> {
        let imported: BookmarkStore = serde_json::from_str(content)?;
        let count = imported.entries.len();
        // Merge: add entries that don't yet exist
        for (id, entry) in imported.entries {
            if !self.store.entries.contains_key(&id) {
                self.store.entries.insert(id, entry);
            }
        }
        for (id, folder) in imported.folders {
            if !self.store.folders.contains_key(&id) {
                self.store.folders.insert(id, folder);
            }
        }
        for fid in imported.root_folders {
            if !self.store.root_folders.contains(&fid) {
                self.store.root_folders.push(fid);
            }
        }
        for bid in imported.root_bookmarks {
            if !self.store.root_bookmarks.contains(&bid) {
                self.store.root_bookmarks.push(bid);
            }
        }
        Ok(count)
    }

    /// Exports to Netscape Bookmark HTML format.
    fn export_html(&self, path: &Path) -> Result<()> {
        let mut html = String::from(
            "<!DOCTYPE NETSCAPE-Bookmark-file-1>\n\
             <META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n\
             <TITLE>Bookmarks</TITLE>\n\
             <H1>Bookmarks</H1>\n\
             <DL><p>\n",
        );

        // Root bookmarks
        for bid in &self.store.root_bookmarks {
            if let Some(entry) = self.store.entries.get(bid) {
                html.push_str(&format!(
                    "    <DT><A HREF=\"{}\" ADD_DATE=\"{}\">{}</A>\n",
                    escape_html(&entry.url),
                    entry.created_at.timestamp(),
                    escape_html(&entry.title),
                ));
            }
        }

        // Root folders
        for fid in &self.store.root_folders {
            self.write_folder_html(&mut html, fid, 1)?;
        }

        html.push_str("</DL><p>\n");
        std::fs::write(path, html)?;
        info!("Exported bookmarks to HTML: {:?}", path);
        Ok(())
    }

    /// Recursively writes a folder subtree in Netscape HTML format.
    fn write_folder_html(&self, html: &mut String, folder_id: &str, depth: usize) -> Result<()> {
        let folder = self
            .store
            .folders
            .get(folder_id)
            .ok_or_else(|| BookmarkError::FolderNotFound(folder_id.to_string()))?;
        let indent = "    ".repeat(depth);
        html.push_str(&format!(
            "{}<DT><H3 ADD_DATE=\"{}\">{}</H3>\n{}<DL><p>\n",
            indent,
            folder.created_at.timestamp(),
            escape_html(&folder.name),
            indent,
        ));

        for bid in &folder.bookmarks {
            if let Some(entry) = self.store.entries.get(bid) {
                html.push_str(&format!(
                    "{}    <DT><A HREF=\"{}\" ADD_DATE=\"{}\">{}</A>\n",
                    indent,
                    escape_html(&entry.url),
                    entry.created_at.timestamp(),
                    escape_html(&entry.title),
                ));
            }
        }

        for child_id in &folder.children {
            self.write_folder_html(html, child_id, depth + 1)?;
        }

        html.push_str(&format!("{}</DL><p>\n", indent));
        Ok(())
    }

    /// Imports from Netscape HTML content.
    fn import_html(&mut self, _content: &str) -> Result<usize> {
        // HTML bookmark parsing is complex; for a production implementation
        // this would use a proper HTML parser. Here we provide a simple
        // regex-based approach as a placeholder.
        use regex::Regex;
        let link_re = Regex::new(r#"<A\s+HREF="([^"]*)"[^>]*>([^<]*)</A>"#).unwrap();
        let mut count = 0;

        for cap in link_re.captures_iter(_content) {
            let url = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let title = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            if !url.is_empty() {
                let _ = self.add_bookmark(&title, &url, None, Vec::new());
                count += 1;
            }
        }

        info!("Imported {} bookmarks from HTML", count);
        Ok(count)
    }

    // -----------------------------------------------------------------------
    // Sync
    // -----------------------------------------------------------------------

    /// Synchronizes the local bookmark store with a remote store.
    ///
    /// Uses a last-write-wins strategy for conflicting entries. Returns a
    /// [`SyncResult`] describing what was changed.
    pub fn sync(&mut self, remote: &BookmarkStore) -> Result<SyncResult> {
        let mut result = SyncResult {
            added: 0,
            updated: 0,
            conflicts_resolved: 0,
            conflicts_remaining: 0,
            synced_at: Utc::now(),
        };

        // Merge remote entries
        for (id, remote_entry) in &remote.entries {
            match self.store.entries.get(id) {
                Some(local_entry) => {
                    if remote_entry.updated_at > local_entry.updated_at {
                        self.store.entries.insert(id.clone(), remote_entry.clone());
                        result.updated += 1;
                        debug!("Sync updated bookmark: {}", id);
                    } else if remote_entry.updated_at == local_entry.updated_at
                        && remote_entry.url != local_entry.url
                    {
                        // Conflict: timestamps equal but content differs
                        result.conflicts_remaining += 1;
                        warn!("Sync conflict for bookmark: {}", id);
                    }
                }
                None => {
                    self.store.entries.insert(id.clone(), remote_entry.clone());
                    result.added += 1;
                    debug!("Sync added bookmark: {}", id);
                }
            }
        }

        // Merge remote folders
        for (id, remote_folder) in &remote.folders {
            if !self.store.folders.contains_key(id) {
                self.store.folders.insert(id.clone(), remote_folder.clone());
                debug!("Sync added folder: {}", id);
            }
        }

        self.store.last_synced = Some(result.synced_at);
        self.maybe_persist()?;
        info!(
            "Sync complete: +{} added, ~{} updated, !{} conflicts",
            result.added, result.updated, result.conflicts_remaining
        );
        Ok(result)
    }

    /// Returns a reference to the internal store (useful for preparing sync payloads).
    pub fn store(&self) -> &BookmarkStore {
        &self.store
    }

    /// Returns the number of bookmarks.
    pub fn bookmark_count(&self) -> usize {
        self.store.entries.len()
    }

    /// Returns the number of folders.
    pub fn folder_count(&self) -> usize {
        self.store.folders.len()
    }

    // -----------------------------------------------------------------------
    // Persistence
    // -----------------------------------------------------------------------

    /// Persists the current state to disk if a storage path is configured.
    fn maybe_persist(&self) -> Result<()> {
        if let Some(ref path) = self.storage_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(&self.store)?;
            std::fs::write(path, json)?;
        }
        Ok(())
    }

    /// Loads the store from the configured storage path.
    fn load_from_disk(&mut self) -> Result<()> {
        if let Some(ref path) = self.storage_path {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                self.store = serde_json::from_str(&content)?;
                info!("Loaded {} bookmarks from disk", self.store.entries.len());
            }
        }
        Ok(())
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Escapes special HTML characters in a string.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_retrieve_bookmark() {
        let mut mgr = BookmarkManager::new();
        let entry = mgr
            .add_bookmark("Test", "https://example.com", None, vec!["test".into()])
            .unwrap();
        assert_eq!(entry.title, "Test");
        assert_eq!(entry.url, "https://example.com");
        assert!(mgr.get_bookmark(&entry.id).is_some());
    }

    #[test]
    fn test_folder_operations() {
        let mut mgr = BookmarkManager::new();
        let folder = mgr.create_folder("Dev", None).unwrap();
        let entry = mgr
            .add_bookmark("Rust", "https://rust-lang.org", Some(&folder.id), vec![])
            .unwrap();
        let in_folder = mgr.bookmarks_in_folder(&folder.id).unwrap();
        assert_eq!(in_folder.len(), 1);
        assert_eq!(in_folder[0].id, entry.id);
    }

    #[test]
    fn test_search() {
        let mut mgr = BookmarkManager::new();
        mgr.add_bookmark("Rust", "https://rust-lang.org", None, vec!["lang".into()])
            .unwrap();
        mgr.add_bookmark("Python", "https://python.org", None, vec!["lang".into()])
            .unwrap();
        let results = mgr.search("rust");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_favorites() {
        let mut mgr = BookmarkManager::new();
        let entry = mgr
            .add_bookmark("Favorite", "https://example.com", None, vec![])
            .unwrap();
        mgr.update_bookmark(&entry.id, None, None, None, Some(true))
            .unwrap();
        assert_eq!(mgr.favorites().len(), 1);
    }

    #[test]
    fn test_remove_bookmark() {
        let mut mgr = BookmarkManager::new();
        let entry = mgr
            .add_bookmark("To Remove", "https://example.com", None, vec![])
            .unwrap();
        mgr.remove_bookmark(&entry.id).unwrap();
        assert!(mgr.get_bookmark(&entry.id).is_none());
    }

    #[test]
    fn test_json_roundtrip() {
        let mut mgr = BookmarkManager::new();
        mgr.add_bookmark("Test", "https://example.com", None, vec![])
            .unwrap();
        let tmp = std::env::temp_dir().join("nova_test_bookmarks.json");
        mgr.export(&tmp, ExportFormat::Json).unwrap();

        let mut mgr2 = BookmarkManager::new();
        mgr2.import(&tmp).unwrap();
        assert_eq!(mgr2.bookmark_count(), 1);
        let _ = std::fs::remove_file(&tmp);
    }
}