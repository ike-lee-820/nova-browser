// Nova Browser - Core Library
// CEF integration (optional), window management, and browser engine

pub mod app;
#[cfg(feature = "cef-support")]
pub mod cef_bridge;
pub mod config;
pub mod window;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Global application state
pub struct NovaState {
    pub config: Arc<RwLock<config::AppConfig>>,
    pub windows: Arc<RwLock<Vec<window::BrowserWindow>>>,
    pub tabs: Arc<RwLock<Vec<TabInfo>>>,
    pub is_incognito: Arc<RwLock<bool>>,
    pub active_theme: Arc<RwLock<ThemeMode>>,
    pub current_locale: Arc<RwLock<String>>,
}

/// Represents a single browser tab
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TabInfo {
    pub id: uuid::Uuid,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub is_loading: bool,
    pub is_pinned: bool,
    pub is_muted: bool,
    pub zoom_level: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Default for TabInfo {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            title: String::from("New Tab"),
            url: String::from("nova://newtab"),
            favicon: None,
            is_loading: false,
            is_pinned: false,
            is_muted: false,
            zoom_level: 1.0,
            created_at: chrono::Utc::now(),
        }
    }
}

/// Theme mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn is_dark(&self) -> bool {
        match self {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => Self::detect_system_theme(),
        }
    }

    fn detect_system_theme() -> bool {
        // Detect system dark mode preference
        #[cfg(target_os = "macos")]
        {
            // macOS dark mode detection
            std::env::var("AppleInterfaceStyle")
                .map(|s| s == "Dark")
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }
}

/// Browser window type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Normal,
    Incognito,
    Popup,
}

/// Split view direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Nova browser result type
pub type NovaResult<T> = Result<T, NovaError>;

#[derive(Debug, thiserror::Error)]
pub enum NovaError {
    #[error("CEF initialization failed: {0}")]
    CefError(String),

    #[error("Window creation failed: {0}")]
    WindowError(String),

    #[error("Tab operation failed: {0}")]
    TabError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl NovaState {
    pub fn new() -> Self {
        let config = config::AppConfig::load().unwrap_or_default();
        let theme = config.theme;
        let locale = config.language.clone();

        Self {
            config: Arc::new(RwLock::new(config)),
            windows: Arc::new(RwLock::new(Vec::new())),
            tabs: Arc::new(RwLock::new(vec![TabInfo::default()])),
            is_incognito: Arc::new(RwLock::new(false)),
            active_theme: Arc::new(RwLock::new(theme)),
            current_locale: Arc::new(RwLock::new(locale)),
        }
    }

    pub async fn add_tab(&self, url: Option<String>) -> TabInfo {
        let mut tab = TabInfo::default();
        if let Some(u) = url {
            tab.url = u;
        }
        let mut tabs = self.tabs.write().await;
        tabs.push(tab.clone());
        tab
    }

    pub async fn close_tab(&self, tab_id: uuid::Uuid) {
        let mut tabs = self.tabs.write().await;
        tabs.retain(|t| t.id != tab_id);
        if tabs.is_empty() {
            tabs.push(TabInfo::default());
        }
    }

    pub async fn toggle_incognito(&self) -> bool {
        let mut incognito = self.is_incognito.write().await;
        *incognito = !*incognito;
        *incognito
    }

    pub async fn set_theme(&self, mode: ThemeMode) {
        let mut theme = self.active_theme.write().await;
        *theme = mode;
        let mut config = self.config.write().await;
        config.theme = mode;
        let _ = config.save();
    }
}