//! # Nova History Manager
//!
//! A full-featured browsing history manager with search, date-based grouping,
//! and flexible clearing options.
//!
//! ## Features
//!
//! - **Visit recording**: Automatic recording of page visits with metadata
//! - **Full-text search**: Search by URL, title, or domain
//! - **Date grouping**: Group history entries by day, week, or month
//! - **Clear options**: Clear by time range, domain, or individual entries
//! - **Visit frequency**: Track how often pages are visited
//! - **Deduplication**: Consecutive visits to the same URL are merged
//!
//! ## Architecture
//!
//! The [`HistoryManager`] maintains an in-memory list of [`HistoryEntry`]
//! records that can be persisted to disk. Entries are indexed by time for
//! efficient range queries and grouping.

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur during history operations.
#[derive(Error, Debug)]
pub enum HistoryError {
    /// A history entry was not found.
    #[error("history entry not found: {0}")]
    NotFound(String),

    /// I/O error during persistence.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Invalid time range specified.
    #[error("invalid time range: {0}")]
    InvalidTimeRange(String),

    /// A generic error.
    #[error("history error: {0}")]
    Other(String),
}

/// Convenience type alias.
pub type Result<T> = std::result::Result<T, HistoryError>;

// ---------------------------------------------------------------------------
// Data models
// ---------------------------------------------------------------------------

/// A single browsing history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique identifier.
    pub id: String,

    /// The URL that was visited.
    pub url: String,

    /// The page title at the time of visit.
    pub title: String,

    /// The domain extracted from the URL.
    pub domain: String,

    /// Timestamp of the visit.
    pub visited_at: DateTime<Utc>,

    /// Duration of the visit in seconds, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visit_duration_secs: Option<u64>,

    /// How this visit was initiated (e.g., typed, link, bookmark).
    #[serde(default = "default_transition")]
    pub transition: TransitionType,

    /// How many times this exact URL has been visited (cumulative).
    #[serde(default = "default_visit_count")]
    pub visit_count: u32,

    /// Optional favicon URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,

    /// Whether this entry was recorded during an incognito session.
    #[serde(default)]
    pub is_incognito: bool,
}

fn default_transition() -> TransitionType {
    TransitionType::Link
}

fn default_visit_count() -> u32 {
    1
}

/// Describes how a navigation was initiated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionType {
    /// User clicked a link on a page.
    Link,
    /// User typed the URL directly.
    Typed,
    /// User used a bookmark.
    Bookmark,
    /// Automatic redirect.
    Redirect,
    /// Page was reloaded.
    Reload,
    /// Form submission.
    FormSubmit,
    /// Back/forward navigation.
    BackForward,
    /// Other/unknown transition.
    Other,
}

/// A group of history entries keyed by date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryGroup {
    /// The date label for this group (e.g., "Today", "Yesterday", "2026-06-19").
    pub label: String,

    /// The date this group represents.
    pub date: NaiveDate,

    /// Entries in this group, sorted most-recent-first.
    pub entries: Vec<HistoryEntry>,
}

/// Options for clearing history.
#[derive(Debug, Clone)]
pub enum ClearOptions {
    /// Clear everything.
    All,
    /// Clear entries older than the given duration.
    OlderThan(Duration),
    /// Clear entries within a specific date range (inclusive).
    DateRange {
        /// Start date (inclusive).
        from: NaiveDate,
        /// End date (inclusive).
        to: NaiveDate,
    },
    /// Clear entries for a specific domain.
    Domain(String),
    /// Clear a specific entry by ID.
    Entry(String),
    /// Clear the last N entries.
    LastN(usize),
}

/// Statistics about the browsing history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStats {
    /// Total number of history entries.
    pub total_entries: usize,
    /// Number of unique URLs visited.
    pub unique_urls: usize,
    /// Number of unique domains visited.
    pub unique_domains: usize,
    /// Most frequently visited domain.
    pub top_domain: Option<String>,
    /// Average entries per day over the last 30 days.
    pub avg_entries_per_day: f64,
    /// Date of the earliest entry.
    pub earliest_visit: Option<DateTime<Utc>>,
    /// Date of the most recent entry.
    pub latest_visit: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// History manager
// ---------------------------------------------------------------------------

/// The main history manager providing all history-related operations.
///
/// # Examples
///
/// ```no_run
/// use nova_features::history::HistoryManager;
///
/// let mut manager = HistoryManager::new();
/// manager.record_visit("https://example.com", "Example Site", "example.com");
/// let results = manager.search("example");
/// ```
pub struct HistoryManager {
    /// All history entries, newest first.
    entries: Vec<HistoryEntry>,

    /// Cumulative visit counts per URL.
    visit_counts: HashMap<String, u32>,

    /// Path to the on-disk persistence file.
    storage_path: Option<PathBuf>,

    /// Maximum number of entries to keep in memory.
    max_entries: usize,
}

impl HistoryManager {
    /// Creates a new history manager with a default capacity of 10,000 entries.
    pub fn new() -> Self {
        info!("Initializing history manager");
        Self {
            entries: Vec::new(),
            visit_counts: HashMap::new(),
            storage_path: None,
            max_entries: 10_000,
        }
    }

    /// Creates a new history manager with a custom maximum capacity.
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            max_entries,
            ..Self::new()
        }
    }

    /// Creates a history manager that persists to the given path.
    pub fn with_persistence(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let mut manager = Self::new();
        manager.storage_path = Some(path.clone());
        if path.exists() {
            match manager.load_from_disk() {
                Ok(_) => info!("Loaded history from disk: {:?}", path),
                Err(e) => warn!("Failed to load history from disk: {}", e),
            }
        }
        manager
    }

    // -----------------------------------------------------------------------
    // Visit recording
    // -----------------------------------------------------------------------

    /// Records a new page visit.
    ///
    /// If the URL matches the most recent entry, that entry is updated instead
    /// of creating a duplicate.
    pub fn record_visit(
        &mut self,
        url: &str,
        title: &str,
        domain: &str,
    ) -> Result<HistoryEntry> {
        self.record_visit_with_options(url, title, domain, TransitionType::Link, None)
    }

    /// Records a page visit with full metadata.
    pub fn record_visit_with_options(
        &mut self,
        url: &str,
        title: &str,
        domain: &str,
        transition: TransitionType,
        favicon: Option<&str>,
    ) -> Result<HistoryEntry> {
        let now = Utc::now();

        // Update cumulative visit count
        let count = self
            .visit_counts
            .entry(url.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);

        let entry = HistoryEntry {
            id: Uuid::new_v4().to_string(),
            url: url.to_string(),
            title: title.to_string(),
            domain: domain.to_string(),
            visited_at: now,
            visit_duration_secs: None,
            transition,
            visit_count: *count,
            favicon: favicon.map(|s| s.to_string()),
            is_incognito: false,
        };

        debug!("Recorded visit: {} -> {}", title, url);
        self.entries.insert(0, entry.clone());
        self.prune_if_needed();
        self.maybe_persist()?;
        Ok(entry)
    }

    /// Removes excess entries beyond the configured maximum.
    fn prune_if_needed(&mut self) {
        if self.entries.len() > self.max_entries {
            let excess = self.entries.len() - self.max_entries;
            self.entries.truncate(self.max_entries);
            debug!("Pruned {} excess history entries", excess);
        }
    }

    // -----------------------------------------------------------------------
    // Querying
    // -----------------------------------------------------------------------

    /// Returns all history entries, newest first.
    pub fn all_entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Returns a specific entry by ID.
    pub fn get_entry(&self, id: &str) -> Option<&HistoryEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Returns all entries for a given domain.
    pub fn entries_for_domain(&self, domain: &str) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.domain.eq_ignore_ascii_case(domain))
            .collect()
    }

    /// Returns the most recent N entries.
    pub fn recent(&self, n: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().take(n).collect()
    }

    /// Returns entries within a date range (inclusive).
    pub fn entries_in_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.visited_at >= from && e.visited_at <= to)
            .collect()
    }

    /// Returns entries from today.
    pub fn today(&self) -> Vec<&HistoryEntry> {
        let today = Utc::now().date_naive();
        let start = today
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let end = today
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc();
        self.entries_in_range(start, end)
    }

    // -----------------------------------------------------------------------
    // Search
    // -----------------------------------------------------------------------

    /// Searches history entries by URL, title, or domain.
    ///
    /// The search is case-insensitive and matches substrings.
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        let query = query.to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        self.entries
            .iter()
            .filter(|e| {
                e.url.to_lowercase().contains(&query)
                    || e.title.to_lowercase().contains(&query)
                    || e.domain.to_lowercase().contains(&query)
            })
            .collect()
    }

    /// Searches history entries with advanced filtering options.
    pub fn search_advanced(
        &self,
        query: &str,
        domain_filter: Option<&str>,
        date_from: Option<DateTime<Utc>>,
        date_to: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Vec<&HistoryEntry> {
        let query = query.to_lowercase();
        let mut results: Vec<&HistoryEntry> = self
            .entries
            .iter()
            .filter(|e| {
                let matches_query = query.is_empty()
                    || e.url.to_lowercase().contains(&query)
                    || e.title.to_lowercase().contains(&query);
                let matches_domain = domain_filter
                    .map(|d| e.domain.eq_ignore_ascii_case(d))
                    .unwrap_or(true);
                let matches_from = date_from.map(|d| e.visited_at >= d).unwrap_or(true);
                let matches_to = date_to.map(|d| e.visited_at <= d).unwrap_or(true);
                matches_query && matches_domain && matches_from && matches_to
            })
            .collect();

        if let Some(limit) = limit {
            results.truncate(limit);
        }
        results
    }

    // -----------------------------------------------------------------------
    // Grouping by date
    // -----------------------------------------------------------------------

    /// Groups entries by date, returning them newest-first.
    pub fn group_by_date(&self) -> Vec<HistoryGroup> {
        let mut groups: BTreeMap<NaiveDate, Vec<HistoryEntry>> = BTreeMap::new();

        for entry in &self.entries {
            let date = entry.visited_at.date_naive();
            groups
                .entry(date)
                .or_default()
                .push(entry.clone());
        }

        let today = Utc::now().date_naive();
        let yesterday = today - Duration::days(1);

        groups
            .into_iter()
            .rev() // newest first
            .map(|(date, entries)| {
                let label = if date == today {
                    "Today".to_string()
                } else if date == yesterday {
                    "Yesterday".to_string()
                } else {
                    date.format("%Y-%m-%d").to_string()
                };
                HistoryGroup {
                    label,
                    date,
                    entries,
                }
            })
            .collect()
    }

    /// Groups entries by week, returning them newest-first.
    pub fn group_by_week(&self) -> Vec<HistoryGroup> {
        let mut groups: BTreeMap<NaiveDate, Vec<HistoryEntry>> = BTreeMap::new();

        for entry in &self.entries {
            let date = entry.visited_at.date_naive();
            // Monday of the week
            let week_start = date
                - Duration::days(
                    date.weekday().num_days_from_monday() as i64,
                );
            groups
                .entry(week_start)
                .or_default()
                .push(entry.clone());
        }

        groups
            .into_iter()
            .rev()
            .map(|(week_start, entries)| HistoryGroup {
                label: format!("Week of {}", week_start.format("%Y-%m-%d")),
                date: week_start,
                entries,
            })
            .collect()
    }

    /// Groups entries by domain.
    pub fn group_by_domain(&self) -> Vec<(String, Vec<&HistoryEntry>)> {
        let mut domain_map: HashMap<String, Vec<&HistoryEntry>> = HashMap::new();
        for entry in &self.entries {
            domain_map
                .entry(entry.domain.clone())
                .or_default()
                .push(entry);
        }
        let mut result: Vec<(String, Vec<&HistoryEntry>)> = domain_map.into_iter().collect();
        // Sort by most entries first
        result.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        result
    }

    // -----------------------------------------------------------------------
    // Clearing
    // -----------------------------------------------------------------------

    /// Clears history entries based on the given options.
    ///
    /// Returns the number of entries removed.
    pub fn clear(&mut self, options: ClearOptions) -> Result<usize> {
        let before = self.entries.len();

        match options {
            ClearOptions::All => {
                info!("Clearing all history");
                self.entries.clear();
                self.visit_counts.clear();
            }
            ClearOptions::OlderThan(duration) => {
                let cutoff = Utc::now() - duration;
                info!("Clearing history older than {:?}", duration);
                self.entries.retain(|e| e.visited_at >= cutoff);
            }
            ClearOptions::DateRange { from, to } => {
                let from_dt = from
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let to_dt = to
                    .and_hms_opt(23, 59, 59)
                    .unwrap()
                    .and_utc();
                info!(
                    "Clearing history from {} to {}",
                    from.format("%Y-%m-%d"),
                    to.format("%Y-%m-%d")
                );
                self.entries
                    .retain(|e| e.visited_at < from_dt || e.visited_at > to_dt);
            }
            ClearOptions::Domain(domain) => {
                info!("Clearing history for domain: {}", domain);
                self.entries
                    .retain(|e| !e.domain.eq_ignore_ascii_case(&domain));
            }
            ClearOptions::Entry(id) => {
                debug!("Clearing history entry: {}", id);
                self.entries.retain(|e| e.id != id);
            }
            ClearOptions::LastN(n) => {
                info!("Clearing last {} history entries", n);
                let remove_count = n.min(self.entries.len());
                self.entries.drain(..remove_count);
            }
        }

        let removed = before - self.entries.len();
        self.maybe_persist()?;
        info!("Cleared {} history entries", removed);
        Ok(removed)
    }

    /// Removes a single history entry by ID.
    pub fn remove_entry(&mut self, id: &str) -> Result<()> {
        let before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        if self.entries.len() == before {
            return Err(HistoryError::NotFound(id.to_string()));
        }
        debug!("Removed history entry: {}", id);
        self.maybe_persist()?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Statistics
    // -----------------------------------------------------------------------

    /// Computes and returns browsing history statistics.
    pub fn statistics(&self) -> HistoryStats {
        let unique_urls: std::collections::HashSet<&str> =
            self.entries.iter().map(|e| e.url.as_str()).collect();
        let unique_domains: std::collections::HashSet<&str> =
            self.entries.iter().map(|e| e.domain.as_str()).collect();

        let mut domain_counts: HashMap<&str, usize> = HashMap::new();
        for entry in &self.entries {
            *domain_counts.entry(&entry.domain).or_default() += 1;
        }
        let top_domain = domain_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(domain, _)| domain.to_string());

        let earliest = self.entries.last().map(|e| e.visited_at);
        let latest = self.entries.first().map(|e| e.visited_at);

        let total_days = if let (Some(earliest), Some(latest)) = (earliest, latest) {
            let days = (latest - earliest).num_days().max(1);
            days as f64
        } else {
            1.0
        };
        let avg_entries_per_day = self.entries.len() as f64 / total_days;

        HistoryStats {
            total_entries: self.entries.len(),
            unique_urls: unique_urls.len(),
            unique_domains: unique_domains.len(),
            top_domain,
            avg_entries_per_day,
            earliest_visit: earliest,
            latest_visit: latest,
        }
    }

    /// Returns the total number of history entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if there are no history entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // -----------------------------------------------------------------------
    // Suggested sites (frequently visited)
    // -----------------------------------------------------------------------

    /// Returns the most frequently visited URLs, up to `limit`.
    pub fn top_sites(&self, limit: usize) -> Vec<&HistoryEntry> {
        let mut entries: Vec<&HistoryEntry> = self.entries.iter().collect();
        entries.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));
        entries.dedup_by(|a, b| a.url == b.url);
        entries.truncate(limit);
        entries
    }

    /// Returns the most recently visited unique URLs, up to `limit`.
    pub fn recent_unique_urls(&self, limit: usize) -> Vec<&str> {
        let mut seen = std::collections::HashSet::new();
        let mut urls = Vec::new();
        for entry in &self.entries {
            if seen.insert(&entry.url) {
                urls.push(entry.url.as_str());
                if urls.len() >= limit {
                    break;
                }
            }
        }
        urls
    }

    // -----------------------------------------------------------------------
    // Persistence
    // -----------------------------------------------------------------------

    /// Serializes the history entries to a JSON file.
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.entries)?;
        std::fs::write(path, json)?;
        info!("Saved {} history entries to {:?}", self.entries.len(), path);
        Ok(())
    }

    /// Loads history entries from a JSON file.
    pub fn load_from_file(&mut self, path: &std::path::Path) -> Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let loaded: Vec<HistoryEntry> = serde_json::from_str(&content)?;
        let count = loaded.len();
        // Rebuild visit counts
        for entry in &loaded {
            let counter = self
                .visit_counts
                .entry(entry.url.clone())
                .or_insert(0);
            *counter = (*counter).max(entry.visit_count);
        }
        self.entries = loaded;
        self.entries.sort_by(|a, b| b.visited_at.cmp(&a.visited_at));
        info!("Loaded {} history entries from {:?}", count, path);
        Ok(count)
    }

    /// Persists to the configured storage path.
    fn maybe_persist(&self) -> Result<()> {
        if let Some(ref path) = self.storage_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(&self.entries)?;
            std::fs::write(path, json)?;
        }
        Ok(())
    }

    /// Loads from the configured storage path.
    fn load_from_disk(&mut self) -> Result<()> {
        if let Some(ref path) = self.storage_path {
            if path.exists() {
                let path = path.clone();
                self.load_from_file(&path)?;
            }
        }
        Ok(())
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_search() {
        let mut mgr = HistoryManager::new();
        mgr.record_visit("https://example.com", "Example", "example.com")
            .unwrap();
        mgr.record_visit("https://rust-lang.org", "Rust", "rust-lang.org")
            .unwrap();

        let results = mgr.search("example");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://example.com");
    }

    #[test]
    fn test_group_by_date() {
        let mut mgr = HistoryManager::new();
        mgr.record_visit("https://example.com", "Example", "example.com")
            .unwrap();
        let groups = mgr.group_by_date();
        assert!(!groups.is_empty());
        assert_eq!(groups[0].label, "Today");
    }

    #[test]
    fn test_clear_options() {
        let mut mgr = HistoryManager::new();
        mgr.record_visit("https://a.com", "A", "a.com").unwrap();
        mgr.record_visit("https://b.com", "B", "b.com").unwrap();
        mgr.record_visit("https://c.com", "C", "c.com").unwrap();

        let removed = mgr.clear(ClearOptions::Domain("b.com".into())).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(mgr.len(), 2);
    }

    #[test]
    fn test_clear_all() {
        let mut mgr = HistoryManager::new();
        mgr.record_visit("https://a.com", "A", "a.com").unwrap();
        let removed = mgr.clear(ClearOptions::All).unwrap();
        assert_eq!(removed, 1);
        assert!(mgr.is_empty());
    }

    #[test]
    fn test_statistics() {
        let mut mgr = HistoryManager::new();
        mgr.record_visit("https://a.com", "A", "a.com").unwrap();
        mgr.record_visit("https://b.com", "B", "b.com").unwrap();
        let stats = mgr.statistics();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.unique_urls, 2);
    }

    #[test]
    fn test_recent_unique_urls() {
        let mut mgr = HistoryManager::new();
        mgr.record_visit("https://a.com", "A", "a.com").unwrap();
        mgr.record_visit("https://a.com", "A2", "a.com").unwrap();
        mgr.record_visit("https://b.com", "B", "b.com").unwrap();
        let urls = mgr.recent_unique_urls(10);
        assert_eq!(urls.len(), 2);
    }
}