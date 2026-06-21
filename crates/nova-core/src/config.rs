use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::ThemeMode;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Theme mode
    #[serde(default)]
    pub theme: ThemeMode,

    /// Language code (e.g., "zh-CN", "en", "ja")
    #[serde(default = "default_language")]
    pub language: String,

    /// Home page URL
    #[serde(default = "default_homepage")]
    pub homepage: String,

    /// Search engine URL template ({searchTerms} placeholder)
    #[serde(default = "default_search_engine")]
    pub search_engine: String,

    /// Enable ad blocking
    #[serde(default = "default_true")]
    pub ad_blocking_enabled: bool,

    /// Enable Do Not Track
    #[serde(default)]
    pub do_not_track: bool,

    /// Download directory
    #[serde(default = "default_download_dir")]
    pub download_dir: PathBuf,

    /// Enable hardware acceleration
    #[serde(default = "default_true")]
    pub hardware_acceleration: bool,

    /// Font size scaling
    #[serde(default = "default_font_size")]
    pub font_size: u32,

    /// Show bookmarks bar
    #[serde(default = "default_true")]
    pub show_bookmarks_bar: bool,

    /// Startup behavior
    #[serde(default)]
    pub startup_behavior: StartupBehavior,

    /// CEF cache path
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,

    /// Custom ad block lists
    #[serde(default)]
    pub custom_filter_lists: Vec<String>,

    /// Custom blocked sites
    #[serde(default)]
    pub blocked_sites: Vec<String>,

    /// Extension directories
    #[serde(default)]
    pub extension_dirs: Vec<PathBuf>,

    /// Password manager master password hash
    #[serde(default)]
    pub master_password_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum StartupBehavior {
    #[default]
    NewTab,
    RestorePrevious,
    CustomPages(Vec<String>),
}

fn default_language() -> String {
    detect_system_language()
}

fn default_homepage() -> String {
    "nova://newtab".to_string()
}

fn default_search_engine() -> String {
    "https://www.google.com/search?q={searchTerms}".to_string()
}

fn default_true() -> bool {
    true
}

fn default_download_dir() -> PathBuf {
    dirs::download_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn default_font_size() -> u32 {
    16
}

fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nova-browser")
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            language: detect_system_language(),
            homepage: default_homepage(),
            search_engine: default_search_engine(),
            ad_blocking_enabled: true,
            do_not_track: false,
            download_dir: default_download_dir(),
            hardware_acceleration: true,
            font_size: 16,
            show_bookmarks_bar: true,
            startup_behavior: StartupBehavior::default(),
            cache_dir: default_cache_dir(),
            custom_filter_lists: Vec::new(),
            blocked_sites: Vec::new(),
            extension_dirs: Vec::new(),
            master_password_hash: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Option<Self> {
        let config_path = config_path();
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => {
                        log::info!("Configuration loaded from {:?}", config_path);
                        return Some(config);
                    }
                    Err(e) => {
                        log::warn!("Failed to parse config: {}", e);
                    }
                },
                Err(e) => {
                    log::warn!("Failed to read config file: {}", e);
                }
            }
        }
        None
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        log::info!("Configuration saved to {:?}", config_path);
        Ok(())
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nova-browser")
        .join("config.toml")
}

/// Detect the system language and map to our supported locales
pub fn detect_system_language() -> String {
    // Try to get locale from environment variables
    let locale = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .unwrap_or_else(|_| String::from("en_US.UTF-8"));

    let lang = locale.split(['-', '_', '.']).next().unwrap_or("en").to_lowercase();

    match lang.as_str() {
        "zh" => {
            if locale.contains("TW") || locale.contains("HK") || locale.contains("Hant") {
                "zh-TW".to_string()
            } else {
                "zh-CN".to_string()
            }
        }
        "ja" => "ja".to_string(),
        "ko" => "ko".to_string(),
        "fr" => "fr".to_string(),
        "de" => "de".to_string(),
        "es" => "es".to_string(),
        "pt" => "pt".to_string(),
        "ru" => "ru".to_string(),
        "ar" => "ar".to_string(),
        "hi" => "hi".to_string(),
        "it" => "it".to_string(),
        "nl" => "nl".to_string(),
        "tr" => "tr".to_string(),
        "vi" => "vi".to_string(),
        "th" => "th".to_string(),
        _ => "en".to_string(),
    }
}

/// Supported locales with their display names
pub const SUPPORTED_LOCALES: &[(&str, &str)] = &[
    ("en", "English"),
    ("zh-CN", "简体中文"),
    ("zh-TW", "繁體中文"),
    ("ja", "日本語"),
    ("ko", "한국어"),
    ("fr", "Français"),
    ("de", "Deutsch"),
    ("es", "Español"),
    ("pt", "Português"),
    ("ru", "Русский"),
    ("ar", "العربية"),
    ("hi", "हिन्दी"),
    ("it", "Italiano"),
    ("nl", "Nederlands"),
    ("tr", "Türkçe"),
    ("vi", "Tiếng Việt"),
    ("th", "ไทย"),
];