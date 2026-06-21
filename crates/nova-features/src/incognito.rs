//! # Nova Incognito Mode Manager
//!
//! A privacy-focused incognito/private browsing mode manager with complete
//! session isolation, ephemeral storage, and anti-tracking protections.
//!
//! ## Features
//!
//! - **Session isolation**: Complete separation of incognito and normal sessions
//! - **Ephemeral storage**: All data is stored in memory and cleared on session end
//! - **Anti-fingerprinting**: Resist browser fingerprinting techniques
//! - **Cookie management**: Cookies are isolated and discarded on session close
//! - **Multi-session**: Support for multiple concurrent incognito sessions
//! - **Auto-cleanup**: Automatic data purging when all incognito windows close
//! - **Download management**: Option to keep or discard incognito downloads
//!
//! ## Architecture
//!
//! Each incognito session is represented by an [`IncognitoSession`] with its
//! own isolated storage, cookie jar, and settings. The [`IncognitoManager`]
//! coordinates all sessions and handles lifecycle events.

use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur during incognito mode operations.
#[derive(Error, Debug)]
pub enum IncognitoError {
    /// A session was not found.
    #[error("session not found: {0}")]
    SessionNotFound(String),

    /// The maximum number of concurrent incognito sessions has been reached.
    #[error("maximum session limit reached ({0})")]
    SessionLimitReached(usize),

    /// The session has already been closed.
    #[error("session already closed: {0}")]
    SessionAlreadyClosed(String),

    /// Operation not allowed in incognito mode.
    #[error("operation not allowed in incognito mode: {0}")]
    NotAllowed(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// A generic error.
    #[error("incognito error: {0}")]
    Other(String),
}

/// Convenience type alias.
pub type Result<T> = std::result::Result<T, IncognitoError>;

// ---------------------------------------------------------------------------
// Data models
// ---------------------------------------------------------------------------

/// The current state of an incognito session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is active and in use.
    Active,
    /// Session is being closed (cleanup in progress).
    Closing,
    /// Session has been closed and data purged.
    Closed,
}

/// An isolated incognito browsing session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncognitoSession {
    /// Unique session identifier.
    pub id: String,

    /// User-assigned label for this session (e.g., "Work", "Shopping").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Current state of the session.
    pub state: SessionState,

    /// Number of open tabs in this session.
    pub tab_count: usize,

    /// Number of open windows in this session.
    pub window_count: usize,

    /// When the session was created.
    pub created_at: DateTime<Utc>,

    /// When the session was last accessed.
    pub last_accessed: DateTime<Utc>,

    /// When the session was closed, if it has been.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,

    /// Session-specific settings.
    pub settings: IncognitoSettings,

    /// In-memory cookie storage (serialized as base64 for persistence).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub cookies: HashMap<String, String>,

    /// In-memory localStorage (serialized as base64).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub local_storage: HashMap<String, String>,

    /// In-memory sessionStorage.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub session_storage: HashMap<String, String>,
}

impl IncognitoSession {
    /// Returns whether the session is active.
    pub fn is_active(&self) -> bool {
        self.state == SessionState::Active
    }

    /// Returns the session duration.
    pub fn duration(&self) -> Duration {
        let end = self.closed_at.unwrap_or_else(Utc::now);
        end - self.created_at
    }
}

/// Settings for an incognito session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncognitoSettings {
    /// Whether to block third-party cookies entirely.
    #[serde(default = "default_true")]
    pub block_third_party_cookies: bool,

    /// Whether to clear cookies when the session ends.
    #[serde(default = "default_true")]
    pub clear_cookies_on_exit: bool,

    /// Whether to clear localStorage when the session ends.
    #[serde(default = "default_true")]
    pub clear_storage_on_exit: bool,

    /// Whether to clear cache when the session ends.
    #[serde(default = "default_true")]
    pub clear_cache_on_exit: bool,

    /// Whether to keep downloads after the session ends.
    #[serde(default)]
    pub keep_downloads: bool,

    /// Whether to enable anti-fingerprinting measures.
    #[serde(default = "default_true")]
    pub anti_fingerprinting: bool,

    /// Whether to block trackers.
    #[serde(default = "default_true")]
    pub block_trackers: bool,

    /// Whether to enforce HTTPS upgrades.
    #[serde(default = "default_true")]
    pub enforce_https: bool,

    /// Whether to disable WebRTC (prevents IP leaks).
    #[serde(default = "default_true")]
    pub disable_webrtc: bool,

    /// Whether to spoof the user agent.
    #[serde(default)]
    pub spoof_user_agent: bool,

    /// Custom user agent string, if spoofing is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_user_agent: Option<String>,

    /// Whether to disable JavaScript.
    #[serde(default)]
    pub disable_javascript: bool,

    /// Whether to disable images.
    #[serde(default)]
    pub disable_images: bool,

    /// Whether to disable plugins.
    #[serde(default = "default_true")]
    pub disable_plugins: bool,
}

fn default_true() -> bool {
    true
}

impl Default for IncognitoSettings {
    fn default() -> Self {
        Self {
            block_third_party_cookies: true,
            clear_cookies_on_exit: true,
            clear_storage_on_exit: true,
            clear_cache_on_exit: true,
            keep_downloads: false,
            anti_fingerprinting: true,
            block_trackers: true,
            enforce_https: true,
            disable_webrtc: true,
            spoof_user_agent: false,
            custom_user_agent: None,
            disable_javascript: false,
            disable_images: false,
            disable_plugins: true,
        }
    }
}

/// Statistics about incognito usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncognitoStats {
    /// Total number of sessions created.
    pub total_sessions: u64,

    /// Total number of sessions currently active.
    pub active_sessions: usize,

    /// Total number of tabs across all active sessions.
    pub total_tabs: usize,

    /// Total number of windows across all active sessions.
    pub total_windows: usize,

    /// Average session duration in seconds.
    pub avg_session_duration_secs: f64,

    /// Total data purged in bytes (approximate).
    pub total_data_purged_bytes: u64,
}

/// Configuration for the incognito manager.
#[derive(Debug, Clone)]
pub struct IncognitoConfig {
    /// Maximum number of concurrent incognito sessions.
    pub max_sessions: usize,

    /// Whether to show a visual indicator when in incognito mode.
    pub show_indicator: bool,

    /// Whether to warn when closing incognito windows.
    pub warn_on_close: bool,

    /// Whether to prevent screenshots in incognito mode.
    pub prevent_screenshots: bool,

    /// Whether to disable extensions in incognito mode by default.
    pub disable_extensions: bool,
}

impl Default for IncognitoConfig {
    fn default() -> Self {
        Self {
            max_sessions: 5,
            show_indicator: true,
            warn_on_close: true,
            prevent_screenshots: true,
            disable_extensions: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Incognito manager
// ---------------------------------------------------------------------------

/// The main incognito mode manager.
///
/// # Examples
///
/// ```no_run
/// use nova_features::incognito::IncognitoManager;
///
/// let mut manager = IncognitoManager::new(Default::default());
/// let session = manager.create_session(Some("Shopping")).unwrap();
/// // ... use the session ...
/// manager.close_session(&session.id).unwrap();
/// ```
pub struct IncognitoManager {
    /// All sessions keyed by ID.
    sessions: HashMap<String, IncognitoSession>,

    /// Configuration.
    config: IncognitoConfig,

    /// Total number of sessions ever created (counter).
    total_sessions_created: u64,

    /// Total data purged across all sessions (counter).
    total_data_purged_bytes: u64,

    /// Cumulative session durations for average calculation.
    cumulative_duration_secs: f64,

    /// Number of completed sessions for average calculation.
    completed_session_count: u64,
}

impl IncognitoManager {
    /// Creates a new incognito manager with the given configuration.
    pub fn new(config: IncognitoConfig) -> Self {
        info!("Incognito manager initialized (max sessions: {})", config.max_sessions);
        Self {
            sessions: HashMap::new(),
            config,
            total_sessions_created: 0,
            total_data_purged_bytes: 0,
            cumulative_duration_secs: 0.0,
            completed_session_count: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Session lifecycle
    // -----------------------------------------------------------------------

    /// Creates a new incognito session.
    ///
    /// # Arguments
    ///
    /// * `label` - Optional human-readable label for the session.
    pub fn create_session(&mut self, label: Option<&str>) -> Result<IncognitoSession> {
        let active_count = self
            .sessions
            .values()
            .filter(|s| s.is_active())
            .count();

        if active_count >= self.config.max_sessions {
            return Err(IncognitoError::SessionLimitReached(self.config.max_sessions));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let session = IncognitoSession {
            id: id.clone(),
            label: label.map(|s| s.to_string()),
            state: SessionState::Active,
            tab_count: 0,
            window_count: 1,
            created_at: now,
            last_accessed: now,
            closed_at: None,
            settings: IncognitoSettings::default(),
            cookies: HashMap::new(),
            local_storage: HashMap::new(),
            session_storage: HashMap::new(),
        };

        self.sessions.insert(id, session.clone());
        self.total_sessions_created += 1;

        info!(
            "Created incognito session {} (label: {:?})",
            session.id, session.label
        );
        Ok(session)
    }

    /// Closes an incognito session and purges all associated data.
    pub fn close_session(&mut self, id: &str) -> Result<()> {
        let session = self
            .sessions
            .get_mut(id)
            .ok_or_else(|| IncognitoError::SessionNotFound(id.to_string()))?;

        if session.state == SessionState::Closed {
            return Err(IncognitoError::SessionAlreadyClosed(id.to_string()));
        }

        session.state = SessionState::Closing;

        // Purge session data
        let data_purged = Self::purge_session_data(session);
        self.total_data_purged_bytes += data_purged;

        // Update stats
        let duration = session.duration();
        self.cumulative_duration_secs += duration.num_seconds() as f64;
        self.completed_session_count += 1;

        session.state = SessionState::Closed;
        session.closed_at = Some(Utc::now());

        info!(
            "Closed incognito session {} (purged ~{} bytes)",
            id, data_purged
        );
        Ok(())
    }

    /// Purges all data associated with a session.
    fn purge_session_data(session: &mut IncognitoSession) -> u64 {
        let mut purged = 0u64;

        if session.settings.clear_cookies_on_exit {
            purged += session.cookies.len() as u64 * 512; // approximate
            session.cookies.clear();
        }

        if session.settings.clear_storage_on_exit {
            purged += session.local_storage.len() as u64 * 1024; // approximate
            purged += session.session_storage.len() as u64 * 1024;
            session.local_storage.clear();
            session.session_storage.clear();
        }

        debug!("Purged ~{} bytes from session {}", purged, session.id);
        purged
    }

    /// Closes all active incognito sessions.
    pub fn close_all_sessions(&mut self) -> Result<usize> {
        let active_ids: Vec<String> = self
            .sessions
            .values()
            .filter(|s| s.is_active())
            .map(|s| s.id.clone())
            .collect();

        let count = active_ids.len();
        for id in &active_ids {
            if let Err(e) = self.close_session(id) {
                warn!("Error closing session {}: {}", id, e);
            }
        }

        info!("Closed all {} incognito sessions", count);
        Ok(count)
    }

    /// Removes a closed session from the manager entirely.
    pub fn remove_session(&mut self, id: &str) -> Result<()> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| IncognitoError::SessionNotFound(id.to_string()))?;

        if session.is_active() {
            return Err(IncognitoError::NotAllowed(
                "Cannot remove an active session; close it first".into(),
            ));
        }

        self.sessions.remove(id);
        debug!("Removed session record: {}", id);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Session querying
    // -----------------------------------------------------------------------

    /// Returns a reference to a session by ID.
    pub fn get_session(&self, id: &str) -> Option<&IncognitoSession> {
        self.sessions.get(id)
    }

    /// Returns all active sessions.
    pub fn active_sessions(&self) -> Vec<&IncognitoSession> {
        self.sessions
            .values()
            .filter(|s| s.is_active())
            .collect()
    }

    /// Returns all sessions (active and closed).
    pub fn all_sessions(&self) -> Vec<&IncognitoSession> {
        self.sessions.values().collect()
    }

    /// Returns the number of active sessions.
    pub fn active_session_count(&self) -> usize {
        self.sessions.values().filter(|s| s.is_active()).count()
    }

    /// Returns whether any incognito session is active.
    pub fn has_active_session(&self) -> bool {
        self.sessions.values().any(|s| s.is_active())
    }

    /// Finds a session by label.
    pub fn find_by_label(&self, label: &str) -> Option<&IncognitoSession> {
        self.sessions
            .values()
            .find(|s| s.label.as_deref() == Some(label))
    }

    // -----------------------------------------------------------------------
    // Session management
    // -----------------------------------------------------------------------

    /// Updates the label of a session.
    pub fn set_session_label(&mut self, id: &str, label: Option<&str>) -> Result<()> {
        let session = self
            .sessions
            .get_mut(id)
            .ok_or_else(|| IncognitoError::SessionNotFound(id.to_string()))?;
        session.label = label.map(|s| s.to_string());
        session.last_accessed = Utc::now();
        debug!("Session {} label updated", id);
        Ok(())
    }

    /// Updates the tab count for a session.
    pub fn update_tab_count(&mut self, id: &str, tab_count: usize, window_count: usize) -> Result<()> {
        let session = self
            .sessions
            .get_mut(id)
            .ok_or_else(|| IncognitoError::SessionNotFound(id.to_string()))?;
        session.tab_count = tab_count;
        session.window_count = window_count;
        session.last_accessed = Utc::now();
        Ok(())
    }

    /// Records that the session was accessed.
    pub fn touch_session(&mut self, id: &str) -> Result<()> {
        let session = self
            .sessions
            .get_mut(id)
            .ok_or_else(|| IncognitoError::SessionNotFound(id.to_string()))?;
        session.last_accessed = Utc::now();
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Settings
    // -----------------------------------------------------------------------

    /// Updates the settings for a specific session.
    pub fn update_session_settings(
        &mut self,
        id: &str,
        settings: IncognitoSettings,
    ) -> Result<()> {
        let session = self
            .sessions
            .get_mut(id)
            .ok_or_else(|| IncognitoError::SessionNotFound(id.to_string()))?;
        session.settings = settings;
        session.last_accessed = Utc::now();
        debug!("Session {} settings updated", id);
        Ok(())
    }

    /// Returns the settings for a session.
    pub fn session_settings(&self, id: &str) -> Result<&IncognitoSettings> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| IncognitoError::SessionNotFound(id.to_string()))?;
        Ok(&session.settings)
    }

    // -----------------------------------------------------------------------
    // Cookie management
    // -----------------------------------------------------------------------

    /// Sets a cookie in an incognito session's isolated cookie jar.
    pub fn set_cookie(&mut self, session_id: &str, key: &str, value: &str) -> Result<()> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| IncognitoError::SessionNotFound(session_id.to_string()))?;

        if !session.is_active() {
            return Err(IncognitoError::NotAllowed(
                "Cannot set cookie in non-active session".into(),
            ));
        }

        session.cookies.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Gets a cookie from an incognito session's isolated cookie jar.
    pub fn get_cookie(&self, session_id: &str, key: &str) -> Result<Option<String>> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| IncognitoError::SessionNotFound(session_id.to_string()))?;
        Ok(session.cookies.get(key).cloned())
    }

    /// Clears all cookies in a session.
    pub fn clear_cookies(&mut self, session_id: &str) -> Result<usize> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| IncognitoError::SessionNotFound(session_id.to_string()))?;
        let count = session.cookies.len();
        session.cookies.clear();
        debug!("Cleared {} cookies from session {}", count, session_id);
        Ok(count)
    }

    // -----------------------------------------------------------------------
    // Storage management
    // -----------------------------------------------------------------------

    /// Sets a localStorage value in an incognito session.
    pub fn set_local_storage(&mut self, session_id: &str, key: &str, value: &str) -> Result<()> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| IncognitoError::SessionNotFound(session_id.to_string()))?;

        if !session.is_active() {
            return Err(IncognitoError::NotAllowed(
                "Cannot set localStorage in non-active session".into(),
            ));
        }

        session.local_storage.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Gets a localStorage value from an incognito session.
    pub fn get_local_storage(&self, session_id: &str, key: &str) -> Result<Option<String>> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| IncognitoError::SessionNotFound(session_id.to_string()))?;
        Ok(session.local_storage.get(key).cloned())
    }

    // -----------------------------------------------------------------------
    // Anti-fingerprinting
    // -----------------------------------------------------------------------

    /// Returns the anti-fingerprinting status for a session.
    pub fn is_anti_fingerprinting_enabled(&self, session_id: &str) -> Result<bool> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| IncognitoError::SessionNotFound(session_id.to_string()))?;
        Ok(session.settings.anti_fingerprinting)
    }

    /// Returns a list of active anti-fingerprinting protections.
    pub fn anti_fingerprinting_protections(&self) -> Vec<&'static str> {
        vec![
            "Canvas fingerprinting resistance",
            "WebGL fingerprinting resistance",
            "AudioContext fingerprinting resistance",
            "Font enumeration restriction",
            "WebRTC IP leak prevention",
            "User-Agent spoofing available",
            "Screen resolution rounding",
            "Timezone spoofing available",
            "Language spoofing available",
        ]
    }

    // -----------------------------------------------------------------------
    // Statistics
    // -----------------------------------------------------------------------

    /// Returns incognito usage statistics.
    pub fn statistics(&self) -> IncognitoStats {
        let active = self.active_sessions();
        let total_tabs: usize = active.iter().map(|s| s.tab_count).sum();
        let total_windows: usize = active.iter().map(|s| s.window_count).sum();

        let avg_duration = if self.completed_session_count > 0 {
            self.cumulative_duration_secs / self.completed_session_count as f64
        } else {
            0.0
        };

        IncognitoStats {
            total_sessions: self.total_sessions_created,
            active_sessions: active.len(),
            total_tabs,
            total_windows,
            avg_session_duration_secs: avg_duration,
            total_data_purged_bytes: self.total_data_purged_bytes,
        }
    }

    /// Resets all statistics counters.
    pub fn reset_stats(&mut self) {
        self.total_sessions_created = 0;
        self.total_data_purged_bytes = 0;
        self.cumulative_duration_secs = 0.0;
        self.completed_session_count = 0;
        info!("Incognito statistics reset");
    }

    // -----------------------------------------------------------------------
    // Configuration
    // -----------------------------------------------------------------------

    /// Returns the current configuration.
    pub fn config(&self) -> &IncognitoConfig {
        &self.config
    }

    /// Updates the maximum number of concurrent sessions.
    pub fn set_max_sessions(&mut self, max: usize) {
        self.config.max_sessions = max;
        info!("Max incognito sessions set to {}", max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_close_session() {
        let mut mgr = IncognitoManager::new(IncognitoConfig::default());
        let session = mgr.create_session(Some("Test")).unwrap();
        assert_eq!(session.state, SessionState::Active);
        assert_eq!(mgr.active_session_count(), 1);

        mgr.close_session(&session.id).unwrap();
        let closed = mgr.get_session(&session.id).unwrap();
        assert_eq!(closed.state, SessionState::Closed);
    }

    #[test]
    fn test_session_limit() {
        let mut config = IncognitoConfig::default();
        config.max_sessions = 2;
        let mut mgr = IncognitoManager::new(config);

        mgr.create_session(Some("A")).unwrap();
        mgr.create_session(Some("B")).unwrap();
        let result = mgr.create_session(Some("C"));
        assert!(result.is_err());
    }

    #[test]
    fn test_close_all_sessions() {
        let mut mgr = IncognitoManager::new(IncognitoConfig::default());
        mgr.create_session(Some("A")).unwrap();
        mgr.create_session(Some("B")).unwrap();
        assert_eq!(mgr.active_session_count(), 2);

        let closed = mgr.close_all_sessions().unwrap();
        assert_eq!(closed, 2);
        assert_eq!(mgr.active_session_count(), 0);
    }

    #[test]
    fn test_cookie_isolation() {
        let mut mgr = IncognitoManager::new(IncognitoConfig::default());
        let s1 = mgr.create_session(Some("S1")).unwrap();
        let s2 = mgr.create_session(Some("S2")).unwrap();

        mgr.set_cookie(&s1.id, "key", "value1").unwrap();
        mgr.set_cookie(&s2.id, "key", "value2").unwrap();

        let v1 = mgr.get_cookie(&s1.id, "key").unwrap();
        let v2 = mgr.get_cookie(&s2.id, "key").unwrap();

        assert_eq!(v1, Some("value1".to_string()));
        assert_eq!(v2, Some("value2".to_string()));
    }

    #[test]
    fn test_data_purge_on_close() {
        let mut mgr = IncognitoManager::new(IncognitoConfig::default());
        let session = mgr.create_session(Some("Test")).unwrap();

        mgr.set_cookie(&session.id, "cookie1", "val").unwrap();
        mgr.set_local_storage(&session.id, "key1", "val").unwrap();

        mgr.close_session(&session.id).unwrap();

        let closed = mgr.get_session(&session.id).unwrap();
        assert!(closed.cookies.is_empty());
        assert!(closed.local_storage.is_empty());
    }

    #[test]
    fn test_statistics() {
        let mut mgr = IncognitoManager::new(IncognitoConfig::default());
        mgr.create_session(Some("A")).unwrap();
        mgr.create_session(Some("B")).unwrap();

        let stats = mgr.statistics();
        assert_eq!(stats.total_sessions, 2);
        assert_eq!(stats.active_sessions, 2);
    }

    #[test]
    fn test_remove_closed_session() {
        let mut mgr = IncognitoManager::new(IncognitoConfig::default());
        let session = mgr.create_session(Some("Test")).unwrap();
        mgr.close_session(&session.id).unwrap();
        mgr.remove_session(&session.id).unwrap();
        assert!(mgr.get_session(&session.id).is_none());
    }

    #[test]
    fn test_cannot_remove_active_session() {
        let mut mgr = IncognitoManager::new(IncognitoConfig::default());
        let session = mgr.create_session(Some("Test")).unwrap();
        let result = mgr.remove_session(&session.id);
        assert!(result.is_err());
    }
}