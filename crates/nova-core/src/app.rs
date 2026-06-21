// Application lifecycle management

use crate::{
    config::AppConfig,
    NovaState,
};

/// Application entry point configuration
pub struct AppSettings {
    pub config_path: Option<String>,
    pub cef_cache_path: String,
    pub user_agent: String,
    pub enable_dev_tools: bool,
    pub enable_logging: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            config_path: None,
            cef_cache_path: dirs::cache_dir()
                .unwrap_or_default()
                .join("nova-browser")
                .to_string_lossy()
                .to_string(),
            user_agent: String::from("NovaBrowser/1.0"),
            enable_dev_tools: true,
            enable_logging: true,
        }
    }
}

/// Initialize the Nova browser application
pub async fn initialize_app(settings: AppSettings) -> Result<NovaState, Box<dyn std::error::Error>> {
    // Initialize logging
    if settings.enable_logging {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .init();
    }

    log::info!("Starting Nova Browser v{}", env!("CARGO_PKG_VERSION"));

    // Initialize CEF (only when feature is enabled)
    #[cfg(feature = "cef-support")]
    crate::cef_bridge::init_cef(&settings.cef_cache_path, &settings.user_agent)?;

    // Create application state
    let state = NovaState::new();

    // Load saved session if not incognito
    {
        let is_incognito = *state.is_incognito.read().await;
        if !is_incognito {
            load_session(&state).await;
        }
    }

    log::info!("Nova Browser initialized successfully");
    Ok(state)
}

/// Shutdown the application
pub async fn shutdown_app(state: &NovaState) {
    log::info!("Shutting down Nova Browser...");

    // Save session
    let is_incognito = *state.is_incognito.read().await;
    if !is_incognito {
        save_session(state).await;
    }

    // Save config
    let config = state.config.read().await;
    let _ = config.save();

    // Shutdown CEF (only when feature is enabled)
    #[cfg(feature = "cef-support")]
    crate::cef_bridge::shutdown_cef();

    log::info!("Nova Browser shutdown complete");
}

/// Save current session state
async fn save_session(state: &NovaState) {
    let tabs = state.tabs.read().await;
    let session_path = dirs::data_dir()
        .unwrap_or_default()
        .join("nova-browser")
        .join("session.json");

    if let Some(parent) = session_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let json = serde_json::to_string_pretty(&*tabs).unwrap_or_default();
    let _ = std::fs::write(&session_path, json);
    log::info!("Session saved to {:?}", session_path);
}

/// Load previous session state
async fn load_session(state: &NovaState) {
    let session_path = dirs::data_dir()
        .unwrap_or_default()
        .join("nova-browser")
        .join("session.json");

    if session_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&session_path) {
            if let Ok(tabs) = serde_json::from_str(&content) {
                let mut current_tabs = state.tabs.write().await;
                *current_tabs = tabs;
                log::info!("Session restored from {:?}", session_path);
                return;
            }
        }
    }
    log::info!("No previous session found, starting fresh");
}

/// Get the app data directory
pub fn app_data_dir() -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_default()
        .join("nova-browser")
}

/// Get the app config directory
pub fn app_config_dir() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_default()
        .join("nova-browser")
}

/// Ensure all required directories exist
pub fn ensure_directories() -> std::io::Result<()> {
    let dirs = vec![
        app_data_dir(),
        app_config_dir(),
        app_data_dir().join("extensions"),
        app_data_dir().join("bookmarks"),
        app_data_dir().join("history"),
        app_data_dir().join("passwords"),
        app_data_dir().join("downloads"),
        app_data_dir().join("themes"),
        app_data_dir().join("cache"),
    ];

    for dir in dirs {
        std::fs::create_dir_all(&dir)?;
    }

    Ok(())
}