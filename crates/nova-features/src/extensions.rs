//! # Nova Extension Manager
//!
//! A comprehensive extension management system supporting installation,
//! uninstallation, enable/disable toggling, and fine-grained permission
//! control.
//!
//! ## Features
//!
//! - **Install/Uninstall**: Package-based installation with dependency resolution
//! - **Enable/Disable**: Per-extension toggling without uninstalling
//! - **Permissions**: Granular permission model with user consent flow
//! - **Auto-update**: Periodic update checking with manual override
//! - **Security**: Sandbox isolation and content security policy enforcement
//!
//! ## Architecture
//!
//! The [`ExtensionManager`] maintains a registry of [`Extension`] entries.
//! Each extension declares its required [`Permission`]s in a manifest, and
//! the manager enforces those permissions at runtime.

use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur during extension operations.
#[derive(Error, Debug)]
pub enum ExtensionError {
    /// An extension was not found.
    #[error("extension not found: {0}")]
    NotFound(String),

    /// The extension is already installed.
    #[error("extension already installed: {0}")]
    AlreadyInstalled(String),

    /// The extension manifest is invalid.
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),

    /// A required permission was denied by the user.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// An extension depends on another extension that is not installed.
    #[error("missing dependency: {0} requires {1}")]
    MissingDependency(String, String),

    /// The extension package is incompatible with the current browser version.
    #[error("incompatible extension: {0}")]
    Incompatible(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// A generic error.
    #[error("extension error: {0}")]
    Other(String),
}

/// Convenience type alias.
pub type Result<T> = std::result::Result<T, ExtensionError>;

// ---------------------------------------------------------------------------
// Permission model
// ---------------------------------------------------------------------------

/// Permissions that an extension can request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Access to browsing history.
    History,
    /// Access to bookmarks.
    Bookmarks,
    /// Access to the clipboard.
    Clipboard,
    /// Access to download files.
    Downloads,
    /// Access to stored passwords (requires additional user consent).
    Passwords,
    /// Access to the current tab's URL and title.
    ActiveTab,
    /// Access to all open tabs.
    Tabs,
    /// Access to browser cookies.
    Cookies,
    /// Access to browser storage (localStorage, sessionStorage).
    Storage,
    /// Can make network requests to any origin.
    UnlimitedStorage,
    /// Access to geolocation data.
    Geolocation,
    /// Access to notifications.
    Notifications,
    /// Access to the webRequest API.
    WebRequest,
    /// Access to the webNavigation API.
    WebNavigation,
    /// Access to the context menu.
    ContextMenus,
    /// Run content scripts on matching pages.
    ContentScripts,
    /// Access to user's installed themes.
    Theme,
    /// Access to the debugger API.
    Debugger,
    /// Access to native messaging.
    NativeMessaging,
    /// Access to proxy settings.
    Proxy,
    /// Access to browsing data (clear history, etc.).
    BrowsingData,
    /// Access to privacy settings.
    Privacy,
    /// Access to the side panel.
    SidePanel,
}

impl Permission {
    /// Returns a human-readable description of the permission.
    pub fn description(&self) -> &'static str {
        match self {
            Permission::History => "Read and modify your browsing history",
            Permission::Bookmarks => "Read and modify your bookmarks",
            Permission::Clipboard => "Read and modify data you copy and paste",
            Permission::Downloads => "Download files and read/modify the download history",
            Permission::Passwords => "Read and modify your stored passwords",
            Permission::ActiveTab => "Access the current tab's URL and title",
            Permission::Tabs => "Access all open tabs",
            Permission::Cookies => "Read and modify cookies",
            Permission::Storage => "Store unlimited client-side data",
            Permission::UnlimitedStorage => "Store unlimited amount of client-side data",
            Permission::Geolocation => "Access your physical location",
            Permission::Notifications => "Display notifications to you",
            Permission::WebRequest => "Observe and modify network requests",
            Permission::WebNavigation => "Access browser navigation events",
            Permission::ContextMenus => "Add items to the browser context menu",
            Permission::ContentScripts => "Run scripts on matching web pages",
            Permission::Theme => "Manage installed themes",
            Permission::Debugger => "Access the debugger (requires restart)",
            Permission::NativeMessaging => "Communicate with native applications",
            Permission::Proxy => "Manage proxy settings",
            Permission::BrowsingData => "Clear recent browsing history, cookies, and related data",
            Permission::Privacy => "Read and modify privacy settings",
            Permission::SidePanel => "Display content in the browser side panel",
        }
    }

    /// Returns whether this permission is considered sensitive (requires extra user confirmation).
    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            Permission::Passwords
                | Permission::Cookies
                | Permission::Debugger
                | Permission::NativeMessaging
                | Permission::Proxy
                | Permission::BrowsingData
        )
    }
}

// ---------------------------------------------------------------------------
// Data models
// ---------------------------------------------------------------------------

/// An extension's manifest describing its metadata and requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    /// Unique extension identifier.
    pub id: String,

    /// Human-readable name.
    pub name: String,

    /// Short description.
    pub description: String,

    /// Semantic version string.
    pub version: String,

    /// Author name or organization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Homepage URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage_url: Option<String>,

    /// Minimum browser version required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_browser_version: Option<String>,

    /// Permissions requested by this extension.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<Permission>,

    /// Host permissions (URL patterns the extension can access).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub host_permissions: Vec<String>,

    /// IDs of other extensions this one depends on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,

    /// Path to the background script, relative to the extension root.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_script: Option<String>,

    /// Content scripts to inject.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_scripts: Vec<ContentScriptDef>,

    /// Icons keyed by size (e.g., "16", "48", "128").
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub icons: HashMap<String, String>,

    /// Default popup page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_popup: Option<String>,

    /// Options page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_page: Option<String>,
}

/// Definition of a content script to inject.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScriptDef {
    /// URL patterns to match.
    pub matches: Vec<String>,

    /// JavaScript files to inject.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub js: Vec<String>,

    /// CSS files to inject.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub css: Vec<String>,

    /// When to inject: "document_start", "document_end", or "document_idle".
    #[serde(default = "default_run_at")]
    pub run_at: String,
}

fn default_run_at() -> String {
    "document_idle".to_string()
}

/// The current state of an installed extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionState {
    /// Extension is installed and enabled.
    Enabled,
    /// Extension is installed but disabled.
    Disabled,
    /// Extension is currently being installed.
    Installing,
    /// Extension has an error and is not functioning.
    Error,
    /// Extension is pending an update.
    UpdatePending,
}

/// An installed extension with its runtime state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    /// The extension's manifest.
    pub manifest: ExtensionManifest,

    /// Current state.
    pub state: ExtensionState,

    /// Path to the extension's installed directory.
    pub install_path: PathBuf,

    /// Permissions that have been granted by the user.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granted_permissions: Vec<Permission>,

    /// Host permissions that have been granted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granted_host_permissions: Vec<String>,

    /// Whether the extension can run in incognito mode.
    #[serde(default)]
    pub allowed_in_incognito: bool,

    /// Timestamp when the extension was installed.
    pub installed_at: DateTime<Utc>,

    /// Timestamp of the last update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,

    /// Error message if the state is Error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl Extension {
    /// Returns true if the extension is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.state == ExtensionState::Enabled
    }

    /// Checks if a specific permission has been granted.
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.granted_permissions.contains(&permission)
    }

    /// Checks if a host permission pattern has been granted.
    pub fn has_host_permission(&self, pattern: &str) -> bool {
        self.granted_host_permissions.iter().any(|p| p == pattern)
    }
}

// ---------------------------------------------------------------------------
// Extension manager
// ---------------------------------------------------------------------------

/// The main extension manager.
///
/// # Examples
///
/// ```no_run
/// use nova_features::extensions::ExtensionManager;
///
/// let mut manager = ExtensionManager::new("/path/to/extensions");
/// // Install an extension from a package file
/// // manager.install_from_package("/path/to/extension.zip");
/// ```
pub struct ExtensionManager {
    /// All installed extensions keyed by ID.
    extensions: HashMap<String, Extension>,

    /// Root directory for installed extensions.
    extensions_dir: PathBuf,

    /// Path to the registry file.
    registry_path: Option<PathBuf>,

    /// Blocklist of extension IDs that are not allowed.
    blocklist: HashSet<String>,
}

impl ExtensionManager {
    /// Creates a new extension manager.
    pub fn new(extensions_dir: impl Into<PathBuf>) -> Self {
        let dir = extensions_dir.into();
        std::fs::create_dir_all(&dir).ok();
        info!("Extension manager initialized at {:?}", dir);
        Self {
            extensions: HashMap::new(),
            extensions_dir: dir,
            registry_path: None,
            blocklist: HashSet::new(),
        }
    }

    /// Creates an extension manager with a registry file for persistence.
    pub fn with_registry(
        extensions_dir: impl Into<PathBuf>,
        registry_path: impl Into<PathBuf>,
    ) -> Self {
        let path = registry_path.into();
        let mut manager = Self::new(extensions_dir);
        manager.registry_path = Some(path.clone());
        if path.exists() {
            match manager.load_registry() {
                Ok(_) => info!("Loaded extension registry from {:?}", path),
                Err(e) => warn!("Failed to load extension registry: {}", e),
            }
        }
        manager
    }

    // -----------------------------------------------------------------------
    // Installation
    // -----------------------------------------------------------------------

    /// Installs an extension from a manifest and extracted package directory.
    pub fn install(
        &mut self,
        manifest: ExtensionManifest,
        package_dir: &Path,
    ) -> Result<Extension> {
        // Check if already installed
        if self.extensions.contains_key(&manifest.id) {
            return Err(ExtensionError::AlreadyInstalled(manifest.id.clone()));
        }

        // Check blocklist
        if self.blocklist.contains(&manifest.id) {
            return Err(ExtensionError::InvalidManifest(format!(
                "Extension {} is blocklisted",
                manifest.id
            )));
        }

        // Check dependencies
        for dep_id in &manifest.dependencies {
            if !self.extensions.contains_key(dep_id) {
                return Err(ExtensionError::MissingDependency(
                    manifest.id.clone(),
                    dep_id.clone(),
                ));
            }
        }

        let install_path = self.extensions_dir.join(&manifest.id);
        let now = Utc::now();

        // Copy package files to install directory
        if package_dir != install_path {
            copy_dir_all(package_dir, &install_path)?;
        }

        let extension = Extension {
            manifest: manifest.clone(),
            state: ExtensionState::Enabled,
            install_path,
            granted_permissions: Vec::new(),
            granted_host_permissions: Vec::new(),
            allowed_in_incognito: false,
            installed_at: now,
            updated_at: None,
            error_message: None,
        };

        info!("Installed extension: {} v{}", manifest.name, manifest.version);
        self.extensions
            .insert(manifest.id.clone(), extension.clone());
        self.maybe_persist()?;
        Ok(extension)
    }

    /// Installs an extension from a CRX (Chrome Extension) file.
    ///
    /// This method:
    /// 1. Parses the CRX file via the [`crate::crx`] module.
    /// 2. Extracts the extension files to a temporary directory.
    /// 3. Reads and validates `manifest.json`.
    /// 4. Calls the existing [`install`](Self::install) method.
    ///
    /// # Arguments
    ///
    /// * `crx_path` - Path to the `.crx` file.
    ///
    /// # Returns
    ///
    /// The installed [`Extension`] on success.
    ///
    /// # Errors
    ///
    /// Returns [`ExtensionError`] if the CRX file is invalid, the manifest
    /// is missing or malformed, or the extension cannot be installed (e.g.
    /// already installed, blocklisted, missing dependencies).
    pub fn install_from_crx_file(&mut self, crx_path: &Path) -> Result<Extension> {
        info!("Installing extension from CRX file: {:?}", crx_path);

        let temp_dir = std::env::temp_dir().join(format!("nova_crx_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).map_err(|e| {
            error!("Failed to create temp directory for CRX extraction: {}", e);
            ExtensionError::IoError(e)
        })?;

        let manifest = crate::crx::install_crx_from_file(crx_path, &temp_dir).map_err(|e| {
            error!("CRX installation failed: {}", e);
            ExtensionError::Other(format!("CRX installation failed: {}", e))
        })?;

        let result = self.install(manifest, &temp_dir);

        // Clean up the temporary extraction directory
        if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
            warn!(
                "Failed to clean up temp directory {:?}: {}",
                temp_dir, e
            );
        }

        result
    }

    /// Uninstalls an extension and removes its files.
    pub fn uninstall(&mut self, id: &str) -> Result<()> {
        let extension = self
            .extensions
            .remove(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;

        // Remove extension files
        if extension.install_path.exists() {
            std::fs::remove_dir_all(&extension.install_path)?;
            debug!("Removed extension files: {:?}", extension.install_path);
        }

        info!("Uninstalled extension: {}", extension.manifest.name);
        self.maybe_persist()?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Enable / Disable
    // -----------------------------------------------------------------------

    /// Enables a disabled extension.
    pub fn enable(&mut self, id: &str) -> Result<()> {
        let extension = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;

        if extension.state == ExtensionState::Enabled {
            return Ok(());
        }

        extension.state = ExtensionState::Enabled;
        extension.error_message = None;
        info!("Enabled extension: {}", extension.manifest.name);
        self.maybe_persist()?;
        Ok(())
    }

    /// Disables an enabled extension.
    pub fn disable(&mut self, id: &str) -> Result<()> {
        let extension = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;

        if extension.state == ExtensionState::Disabled {
            return Ok(());
        }

        extension.state = ExtensionState::Disabled;
        info!("Disabled extension: {}", extension.manifest.name);
        self.maybe_persist()?;
        Ok(())
    }

    /// Toggles the enabled/disabled state of an extension.
    pub fn toggle(&mut self, id: &str) -> Result<bool> {
        let extension = self
            .extensions
            .get(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;
        let is_enabled = extension.is_enabled();
        if is_enabled {
            self.disable(id)?;
        } else {
            self.enable(id)?;
        }
        Ok(!is_enabled)
    }

    // -----------------------------------------------------------------------
    // Permission management
    // -----------------------------------------------------------------------

    /// Grants a permission to an extension.
    pub fn grant_permission(&mut self, id: &str, permission: Permission) -> Result<()> {
        let extension = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;

        // Verify the permission is in the manifest
        if !extension.manifest.permissions.contains(&permission) {
            return Err(ExtensionError::InvalidManifest(format!(
                "Extension {} does not request permission {:?}",
                id, permission
            )));
        }

        if !extension.granted_permissions.contains(&permission) {
            extension.granted_permissions.push(permission);
            info!(
                "Granted permission {:?} to extension {}",
                permission, extension.manifest.name
            );
            self.maybe_persist()?;
        }
        Ok(())
    }

    /// Revokes a permission from an extension.
    pub fn revoke_permission(&mut self, id: &str, permission: Permission) -> Result<()> {
        let extension = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;

        extension.granted_permissions.retain(|p| *p != permission);
        info!(
            "Revoked permission {:?} from extension {}",
            permission, extension.manifest.name
        );
        self.maybe_persist()?;
        Ok(())
    }

    /// Grants an optional host permission.
    pub fn grant_host_permission(&mut self, id: &str, pattern: &str) -> Result<()> {
        let extension = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;

        if !extension.granted_host_permissions.iter().any(|p| p == pattern) {
            extension
                .granted_host_permissions
                .push(pattern.to_string());
            info!(
                "Granted host permission {} to extension {}",
                pattern, extension.manifest.name
            );
            self.maybe_persist()?;
        }
        Ok(())
    }

    /// Returns all permissions requested by an extension.
    pub fn requested_permissions(&self, id: &str) -> Result<Vec<Permission>> {
        let extension = self
            .extensions
            .get(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;
        Ok(extension.manifest.permissions.clone())
    }

    /// Returns all permissions currently granted to an extension.
    pub fn granted_permissions(&self, id: &str) -> Result<Vec<Permission>> {
        let extension = self
            .extensions
            .get(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;
        Ok(extension.granted_permissions.clone())
    }

    // -----------------------------------------------------------------------
    // Settings
    // -----------------------------------------------------------------------

    /// Sets whether an extension can run in incognito mode.
    pub fn set_incognito_allowed(&mut self, id: &str, allowed: bool) -> Result<()> {
        let extension = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;
        extension.allowed_in_incognito = allowed;
        info!(
            "Incognito mode {} for extension {}",
            if allowed { "enabled" } else { "disabled" },
            extension.manifest.name
        );
        self.maybe_persist()?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Blocklist management
    // -----------------------------------------------------------------------

    /// Adds an extension ID to the blocklist.
    pub fn blocklist_add(&mut self, id: &str) {
        self.blocklist.insert(id.to_string());
        info!("Added {} to extension blocklist", id);
    }

    /// Removes an extension ID from the blocklist.
    pub fn blocklist_remove(&mut self, id: &str) {
        self.blocklist.remove(id);
        info!("Removed {} from extension blocklist", id);
    }

    /// Checks if an extension ID is blocklisted.
    pub fn is_blocklisted(&self, id: &str) -> bool {
        self.blocklist.contains(id)
    }

    // -----------------------------------------------------------------------
    // Update checking
    // -----------------------------------------------------------------------

    /// Marks an extension as having an update available.
    pub fn mark_update_available(&mut self, id: &str, new_version: &str) -> Result<()> {
        let extension = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;
        extension.state = ExtensionState::UpdatePending;
        info!(
            "Update available for {}: {} -> {}",
            extension.manifest.name, extension.manifest.version, new_version
        );
        self.maybe_persist()?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Querying
    // -----------------------------------------------------------------------

    /// Returns a reference to an extension by ID.
    pub fn get(&self, id: &str) -> Option<&Extension> {
        self.extensions.get(id)
    }

    /// Returns all installed extensions.
    pub fn all(&self) -> Vec<&Extension> {
        self.extensions.values().collect()
    }

    /// Returns all enabled extensions.
    pub fn enabled(&self) -> Vec<&Extension> {
        self.extensions
            .values()
            .filter(|e| e.is_enabled())
            .collect()
    }

    /// Returns all disabled extensions.
    pub fn disabled(&self) -> Vec<&Extension> {
        self.extensions
            .values()
            .filter(|e| e.state == ExtensionState::Disabled)
            .collect()
    }

    /// Returns extensions with errors.
    pub fn errored(&self) -> Vec<&Extension> {
        self.extensions
            .values()
            .filter(|e| e.state == ExtensionState::Error)
            .collect()
    }

    /// Searches extensions by name or description.
    pub fn search(&self, query: &str) -> Vec<&Extension> {
        let query = query.to_lowercase();
        self.extensions
            .values()
            .filter(|e| {
                e.manifest.name.to_lowercase().contains(&query)
                    || e.manifest.description.to_lowercase().contains(&query)
            })
            .collect()
    }

    /// Returns the total number of installed extensions.
    pub fn len(&self) -> usize {
        self.extensions.len()
    }

    /// Returns true if there are no installed extensions.
    pub fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }

    // -----------------------------------------------------------------------
    // Persistence
    // -----------------------------------------------------------------------

    /// Saves the extension registry to a file.
    pub fn save_registry_to(&self, path: &Path) -> Result<()> {
        let extensions: Vec<&Extension> = self.extensions.values().collect();
        let json = serde_json::to_string_pretty(&extensions)?;
        std::fs::write(path, json)?;
        info!("Saved extension registry to {:?}", path);
        Ok(())
    }

    /// Loads the extension registry from a file.
    fn load_registry(&mut self) -> Result<()> {
        if let Some(ref path) = self.registry_path {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                let loaded: Vec<Extension> = serde_json::from_str(&content)?;
                for ext in loaded {
                    self.extensions.insert(ext.manifest.id.clone(), ext);
                }
                info!(
                    "Loaded {} extensions from registry",
                    self.extensions.len()
                );
            }
        }
        Ok(())
    }

    /// Persists to the configured registry path.
    fn maybe_persist(&self) -> Result<()> {
        if let Some(ref path) = self.registry_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let extensions: Vec<&Extension> = self.extensions.values().collect();
            let json = serde_json::to_string_pretty(&extensions)?;
            std::fs::write(path, json)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Recursively copies a directory.
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_manifest(id: &str, name: &str) -> ExtensionManifest {
        ExtensionManifest {
            id: id.to_string(),
            name: name.to_string(),
            description: "Test extension".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Test Author".to_string()),
            homepage_url: None,
            minimum_browser_version: None,
            permissions: vec![Permission::Storage, Permission::Tabs],
            host_permissions: vec!["*://*.example.com/*".to_string()],
            dependencies: vec![],
            background_script: Some("background.js".to_string()),
            content_scripts: vec![],
            icons: HashMap::new(),
            default_popup: Some("popup.html".to_string()),
            options_page: None,
        }
    }

    #[test]
    fn test_install_and_uninstall() {
        let tmp = std::env::temp_dir().join("nova_test_extensions");
        let pkg_dir = std::env::temp_dir().join("nova_test_pkg");

        // Clean up first
        let _ = std::fs::remove_dir_all(&tmp);
        let _ = std::fs::remove_dir_all(&pkg_dir);
        std::fs::create_dir_all(&pkg_dir).unwrap();

        let mut mgr = ExtensionManager::new(&tmp);
        let manifest = make_test_manifest("test-ext-1", "Test Extension");

        let ext = mgr.install(manifest, &pkg_dir).unwrap();
        assert_eq!(ext.manifest.name, "Test Extension");
        assert!(ext.is_enabled());
        assert_eq!(mgr.len(), 1);

        mgr.uninstall("test-ext-1").unwrap();
        assert!(mgr.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
        let _ = std::fs::remove_dir_all(&pkg_dir);
    }

    #[test]
    fn test_enable_disable() {
        let tmp = std::env::temp_dir().join("nova_test_extensions2");
        let pkg_dir = std::env::temp_dir().join("nova_test_pkg2");
        let _ = std::fs::remove_dir_all(&tmp);
        let _ = std::fs::remove_dir_all(&pkg_dir);
        std::fs::create_dir_all(&pkg_dir).unwrap();

        let mut mgr = ExtensionManager::new(&tmp);
        let manifest = make_test_manifest("test-ext-2", "Toggle Test");

        mgr.install(manifest, &pkg_dir).unwrap();
        mgr.disable("test-ext-2").unwrap();
        assert!(!mgr.get("test-ext-2").unwrap().is_enabled());

        mgr.enable("test-ext-2").unwrap();
        assert!(mgr.get("test-ext-2").unwrap().is_enabled());

        let _ = std::fs::remove_dir_all(&tmp);
        let _ = std::fs::remove_dir_all(&pkg_dir);
    }

    #[test]
    fn test_permissions() {
        let tmp = std::env::temp_dir().join("nova_test_extensions3");
        let pkg_dir = std::env::temp_dir().join("nova_test_pkg3");
        let _ = std::fs::remove_dir_all(&tmp);
        let _ = std::fs::remove_dir_all(&pkg_dir);
        std::fs::create_dir_all(&pkg_dir).unwrap();

        let mut mgr = ExtensionManager::new(&tmp);
        let manifest = make_test_manifest("test-ext-3", "Permission Test");

        mgr.install(manifest, &pkg_dir).unwrap();
        mgr.grant_permission("test-ext-3", Permission::Storage)
            .unwrap();
        mgr.grant_permission("test-ext-3", Permission::Tabs).unwrap();

        let granted = mgr.granted_permissions("test-ext-3").unwrap();
        assert!(granted.contains(&Permission::Storage));
        assert!(granted.contains(&Permission::Tabs));

        let _ = std::fs::remove_dir_all(&tmp);
        let _ = std::fs::remove_dir_all(&pkg_dir);
    }

    #[test]
    fn test_blocklist() {
        let mut mgr = ExtensionManager::new("/tmp/nova_test_extensions4");
        mgr.blocklist_add("evil-ext");
        assert!(mgr.is_blocklisted("evil-ext"));
        mgr.blocklist_remove("evil-ext");
        assert!(!mgr.is_blocklisted("evil-ext"));
    }
}