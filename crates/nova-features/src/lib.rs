// Nova Browser Features Library
// Advanced browser features: bookmarks, history, extensions,
// password manager, ad blocker, reading mode, dev tools, incognito

pub mod ad_blocker;
pub mod bookmarks;
pub mod crx;
pub mod dev_tools;
pub mod downloads;
pub mod extensions;
pub mod history;
pub mod incognito;
pub mod password_manager;
pub mod reading_mode;
pub mod settings;

use std::path::PathBuf;

/// Feature manager coordinates all browser features.
pub struct FeatureManager {
    pub bookmarks: bookmarks::BookmarkManager,
    pub history: history::HistoryManager,
    pub downloads: downloads::DownloadManager,
    pub extensions: extensions::ExtensionManager,
    pub password_manager: password_manager::PasswordManager,
    pub ad_blocker: ad_blocker::AdBlocker,
    pub reading_mode: reading_mode::ReadingMode,
    pub dev_tools: dev_tools::DevTools,
    pub incognito: incognito::IncognitoManager,
    pub settings: settings::Settings,
}

impl FeatureManager {
    /// Creates a new `FeatureManager` with all features initialized.
    ///
    /// All persistent features are configured to store data under the given
    /// `data_dir`.
    pub fn new(data_dir: PathBuf) -> Self {
        // Ensure data directories exist
        let _ = std::fs::create_dir_all(&data_dir);

        let downloads_config = downloads::DownloadConfig {
            download_dir: data_dir.join("downloads"),
            ..Default::default()
        };

        let ad_blocker_config = ad_blocker::AdBlockerConfig::default();
        let incognito_config = incognito::IncognitoConfig::default();

        Self {
            bookmarks: bookmarks::BookmarkManager::with_persistence(
                data_dir.join("bookmarks.json"),
            ),
            history: history::HistoryManager::with_persistence(
                data_dir.join("history.json"),
            ),
            downloads: downloads::DownloadManager::with_persistence(
                downloads_config,
                data_dir.join("downloads_history.json"),
            ),
            extensions: extensions::ExtensionManager::with_registry(
                data_dir.join("extensions"),
                data_dir.join("extension_registry.json"),
            ),
            password_manager: password_manager::PasswordManager::with_vault_path(
                data_dir.join("passwords.enc"),
            ),
            ad_blocker: ad_blocker::AdBlocker::new(ad_blocker_config),
            reading_mode: reading_mode::ReadingMode::new(),
            dev_tools: dev_tools::DevTools::new(),
            incognito: incognito::IncognitoManager::new(incognito_config),
            settings: settings::Settings::default(),
        }
    }

    // --- Settings ---

    /// Get a reference to the current settings.
    pub fn get_settings(&self) -> &settings::Settings {
        &self.settings
    }

    /// Update the settings.
    pub fn update_settings(&mut self, new_settings: &settings::Settings) {
        self.settings = new_settings.clone();
    }

    // --- Bookmarks ---

    /// Get all bookmarks as a serializable list.
    pub fn get_bookmarks(&self) -> Vec<&bookmarks::BookmarkEntry> {
        self.bookmarks.all_bookmarks()
    }

    /// Add a bookmark with the given title and URL.
    pub fn add_bookmark(&mut self, title: &str, url: &str) {
        if let Err(e) = self.bookmarks.add_bookmark(title, url, None, Vec::new()) {
            log::warn!("Failed to add bookmark: {:?}", e);
        }
    }

    /// Remove a bookmark by URL.
    pub fn remove_bookmark(&mut self, url: &str) {
        let entries = self.bookmarks.all_bookmarks();
        let to_remove: Vec<String> = entries
            .iter()
            .filter(|e| e.url == url)
            .map(|e| e.id.clone())
            .collect();
        for id in to_remove {
            if let Err(e) = self.bookmarks.remove_bookmark(&id) {
                log::warn!("Failed to remove bookmark {}: {:?}", id, e);
            }
        }
    }

    // --- History ---

    /// Get all history entries.
    pub fn get_history(&self) -> Vec<&history::HistoryEntry> {
        self.history.recent(1000)
    }

    /// Clear all browsing history.
    pub fn clear_history(&mut self) {
        if let Err(e) = self.history.clear(history::ClearOptions::All) {
            log::warn!("Failed to clear history: {:?}", e);
        }
    }

    // --- Passwords ---

    /// Get all saved passwords (requires master password to unlock).
    pub fn get_passwords(&mut self, master_password: &str) -> Vec<serde_json::Value> {
        if self.password_manager.is_locked() {
            if let Err(e) = self.password_manager.unlock(master_password) {
                log::warn!("Failed to unlock password vault: {:?}", e);
                return Vec::new();
            }
        }
        match self.password_manager.all_credentials() {
            Ok(creds) => creds
                .iter()
                .map(|c| serde_json::json!({
                    "id": c.id,
                    "url": c.url,
                    "username": c.username,
                }))
                .collect(),
            Err(e) => {
                log::warn!("Failed to get credentials: {:?}", e);
                Vec::new()
            }
        }
    }

    /// Save a password entry.
    pub fn save_password(&mut self, site: &str, username: &str, password: &str, master_password: &str) {
        if self.password_manager.is_locked() {
            if let Err(e) = self.password_manager.unlock(master_password) {
                log::warn!("Failed to unlock password vault: {:?}", e);
                return;
            }
        }
        if let Err(e) = self.password_manager.add_credential(site, username, password, site) {
            log::warn!("Failed to save password: {:?}", e);
        }
    }

    // --- Ad Blocker ---

    /// Check if a URL should be blocked.
    pub fn should_block_url(&mut self, url: &str) -> bool {
        self.ad_blocker.should_block(url, "", true)
    }

    /// Add a custom ad-block filter rule.
    pub fn add_adblock_rule(&mut self, rule: &str) {
        if let Err(e) = self.ad_blocker.add_custom_rule(rule, "User-defined") {
            log::warn!("Failed to add adblock rule: {:?}", e);
        }
    }

    // --- Downloads ---

    /// Get all download entries.
    pub fn get_downloads(&self) -> Vec<&downloads::Download> {
        self.downloads.all()
    }

    // --- Reading Mode ---

    /// Extract readable content from HTML.
    pub fn extract_reading_content(&self, html: &str) -> String {
        match self.reading_mode.extract(html, "about:blank") {
            Ok(content) => content.html_content,
            Err(e) => {
                log::warn!("Failed to extract content: {:?}", e);
                String::new()
            }
        }
    }

    // --- URL Safety ---

    /// Check URL safety (returns "safe", "phishing", "malware", "unknown").
    pub fn check_url_safety(&self, url: &str) -> String {
        if self.ad_blocker.check_phishing(url) {
            "phishing".to_string()
        } else {
            "safe".to_string()
        }
    }
}

/// Supported locales for internationalization.
pub const SUPPORTED_LOCALES: &[(&str, &str)] = &[
    ("en", "English"),
    ("zh-CN", "\u{7B80}\u{4F53}\u{4E2D}\u{6587}"),
    ("zh-TW", "\u{7E41}\u{9AD4}\u{4E2D}\u{6587}"),
    ("ja", "\u{65E5}\u{672C}\u{8A9E}"),
    ("ko", "\u{D55C}\u{AD6D}\u{C5B4}"),
    ("fr", "Fran\u{E7}ais"),
    ("de", "Deutsch"),
    ("es", "Espa\u{F1}ol"),
    ("pt", "Portugu\u{EA}s"),
    ("ru", "\u{420}\u{443}\u{441}\u{441}\u{43A}\u{438}\u{439}"),
    ("ar", "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64A}\u{629}"),
    ("hi", "\u{939}\u{93F}\u{928}\u{94D}\u{926}\u{940}"),
    ("it", "Italiano"),
    ("nl", "Nederlands"),
    ("tr", "T\u{FC}rk\u{E7}e"),
    ("vi", "Ti\u{1EBF}ng Vi\u{1EC7}t"),
    ("th", "\u{E44}\u{E17}\u{E22}"),
];