// Nova Browser Settings Module
// Comprehensive settings covering all browser configuration categories:
// 1. Privacy & Security
// 2. Appearance & Experience
// 3. Functionality & Efficiency
// 4. Advanced & Technical
// 5. Mobile-specific

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Tracking protection level for the browser.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrackingProtectionLevel {
    /// Standard protection: blocks known trackers, allows some for compatibility.
    Standard,
    /// Strict protection: blocks all detected trackers, may break some sites.
    Strict,
    /// Custom protection: user-defined blocklist and rules.
    Custom,
}

/// Visual theme mode for the browser chrome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeMode {
    /// Light theme.
    Light,
    /// Dark theme.
    Dark,
    /// Follow the operating system theme.
    System,
}

/// Layout of the new tab page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NewTabLayout {
    /// Speed dial with frequently visited sites.
    #[serde(rename = "speed-dial")]
    SpeedDial,
    /// News feed.
    News,
    /// Blank page.
    Blank,
    /// Custom content.
    Custom,
}

/// What the browser should do on startup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StartupBehavior {
    /// Open a new tab.
    #[serde(rename = "new-tab")]
    NewTab,
    /// Restore the previous session.
    #[serde(rename = "restore-previous")]
    RestorePrevious,
    /// Open a custom set of pages.
    #[serde(rename = "custom-pages")]
    CustomPages,
}

/// Position of the sidebar relative to the content area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SidebarPosition {
    Left,
    Right,
}

/// Type of proxy server to use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProxyType {
    /// No proxy.
    None,
    /// HTTP proxy.
    Http,
    /// HTTPS proxy.
    Https,
    /// SOCKSv4 proxy.
    Socks4,
    /// SOCKSv5 proxy.
    Socks5,
    /// Proxy auto-config (PAC) script.
    Pac,
}

/// Minimum TLS version to accept for connections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TlsMinVersion {
    /// TLS 1.0 (legacy, insecure).
    #[serde(rename = "tls1.0")]
    Tls1_0,
    /// TLS 1.1 (legacy, insecure).
    #[serde(rename = "tls1.1")]
    Tls1_1,
    /// TLS 1.2 (widely supported).
    #[serde(rename = "tls1.2")]
    Tls1_2,
    /// TLS 1.3 (latest, most secure).
    #[serde(rename = "tls1.3")]
    Tls1_3,
}

/// Autoplay policy for media content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutoplayPolicy {
    /// Allow all autoplay (audio + video).
    #[serde(rename = "allow-all")]
    AllowAll,
    /// Block audio autoplay, allow video autoplay.
    #[serde(rename = "block-audio")]
    BlockAudio,
    /// Block all autoplay.
    #[serde(rename = "block-all")]
    BlockAll,
}

/// Page preloading strategy for performance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreloadPages {
    /// No preloading.
    None,
    /// Standard preloading: preloads pages you are likely to visit.
    Standard,
    /// Extended preloading: preloads more pages for even faster navigation.
    Extended,
}

// ---------------------------------------------------------------------------
// Settings struct
// ---------------------------------------------------------------------------

/// Comprehensive settings for the Nova Browser.
///
/// Covers privacy & security, appearance & experience, functionality &
/// efficiency, advanced & technical, and mobile-specific settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    // =======================================================================
    // 1. Privacy & Security
    // =======================================================================

    // --- Cookie management ---
    /// Whether to allow third-party cookies.
    #[serde(default = "default_true")]
    pub allow_third_party_cookies: bool,
    /// Domains for which cookies are always retained, even after clearing.
    #[serde(default)]
    pub cookie_retention_list: Vec<String>,
    /// Whether to block third-party cookies by default (overrides per-site).
    #[serde(default = "default_true")]
    pub block_third_party_cookies_by_default: bool,

    // --- Tracking protection ---
    /// The level of tracking protection.
    #[serde(default = "default_tracking_protection_level")]
    pub tracking_protection_level: TrackingProtectionLevel,
    /// Whether to block ad trackers.
    #[serde(default = "default_true")]
    pub block_ad_trackers: bool,
    /// Whether to block browser fingerprinting attempts.
    #[serde(default = "default_true")]
    pub block_fingerprinting: bool,
    /// Whether to block social media trackers.
    #[serde(default = "default_true")]
    pub block_social_media_trackers: bool,
    /// Custom list of tracker domains to block.
    #[serde(default)]
    pub custom_tracker_blocklist: Vec<String>,

    // --- HTTPS ---
    /// Whether to force HTTPS for all connections.
    #[serde(default = "default_true")]
    pub force_https: bool,
    /// Whether to warn the user when connecting over insecure HTTP.
    #[serde(default = "default_true")]
    pub warn_insecure_connections: bool,

    // --- Safe browsing ---
    /// Whether Google Safe Browsing is enabled.
    #[serde(default = "default_true")]
    pub safe_browsing_enabled: bool,
    /// Whether to scan downloads for malware.
    #[serde(default = "default_true")]
    pub scan_downloads: bool,
    /// Whether phishing protection is enabled.
    #[serde(default = "default_true")]
    pub phishing_protection: bool,

    // --- Password manager ---
    /// Whether the built-in password generator is enabled.
    #[serde(default = "default_true")]
    pub password_generator_enabled: bool,
    /// Whether to auto-fill saved passwords.
    #[serde(default = "default_true")]
    pub auto_fill_passwords: bool,
    /// Whether to check saved passwords against known data breaches.
    #[serde(default = "default_true")]
    pub leak_detection: bool,
    /// Whether a master password is required to access stored credentials.
    #[serde(default)]
    pub use_master_password: bool,
    /// Whether biometric protection (fingerprint / face) is enabled.
    #[serde(default)]
    pub biometric_protection: bool,

    // --- Permissions (per-site) ---
    /// Per-site camera permission.
    #[serde(default)]
    pub per_site_camera: HashMap<String, bool>,
    /// Per-site microphone permission.
    #[serde(default)]
    pub per_site_microphone: HashMap<String, bool>,
    /// Per-site location permission.
    #[serde(default)]
    pub per_site_location: HashMap<String, bool>,
    /// Per-site notification permission.
    #[serde(default)]
    pub per_site_notifications: HashMap<String, bool>,
    /// Per-site autoplay permission.
    #[serde(default)]
    pub per_site_autoplay: HashMap<String, bool>,
    /// Per-site clipboard permission.
    #[serde(default)]
    pub per_site_clipboard: HashMap<String, bool>,

    // --- Privacy mode ---
    /// Whether to clear all browsing data when exiting incognito mode.
    #[serde(default = "default_true")]
    pub incognito_clear_on_exit: bool,
    /// Whether to block third-party cookies in incognito mode.
    #[serde(default = "default_true")]
    pub block_third_party_in_incognito: bool,

    // --- Do Not Track ---
    /// Whether to send the DNT (Do Not Track) HTTP header.
    #[serde(default)]
    pub send_dnt_header: bool,

    // --- Certificate management ---
    /// Paths to custom CA certificates to trust.
    #[serde(default)]
    pub custom_certificates: Vec<String>,
    /// Path to a custom trust store (e.g., OS or enterprise CA bundle).
    #[serde(default)]
    pub trust_store_path: Option<String>,

    // --- Site isolation ---
    /// Whether site isolation is enabled (each site in its own process).
    #[serde(default = "default_true")]
    pub site_isolation_enabled: bool,
    /// Whether to use strict origin isolation (separate process per origin).
    #[serde(default)]
    pub strict_isolation: bool,

    // =======================================================================
    // 2. Appearance & Experience
    // =======================================================================

    // --- Theme ---
    /// The theme mode.
    #[serde(default = "default_theme_mode")]
    pub theme_mode: ThemeMode,
    /// Custom accent color (CSS color string, e.g., "#ff6600").
    #[serde(default)]
    pub custom_accent_color: Option<String>,
    /// Path to a custom theme file.
    #[serde(default)]
    pub custom_theme_path: Option<String>,

    // --- New tab page ---
    /// Layout of the new tab page.
    #[serde(default = "default_new_tab_layout")]
    pub new_tab_layout: NewTabLayout,
    /// Whether to show shortcuts on the new tab page.
    #[serde(default = "default_true")]
    pub show_shortcuts: bool,
    /// URL of a custom background image for the new tab page.
    #[serde(default)]
    pub new_tab_background_url: Option<String>,
    /// Whether to show the search box on the new tab page.
    #[serde(default = "default_true")]
    pub show_search_box: bool,

    // --- Startup ---
    /// What to do when the browser starts.
    #[serde(default = "default_startup_behavior")]
    pub startup_behavior: StartupBehavior,
    /// Custom pages to open on startup (when `startup_behavior` is CustomPages).
    #[serde(default)]
    pub custom_startup_pages: Vec<String>,

    // --- Bookmarks ---
    /// Whether the bookmarks bar is visible.
    #[serde(default = "default_true")]
    pub bookmarks_bar_visible: bool,
    /// Whether to automatically sort bookmarks alphabetically.
    #[serde(default)]
    pub auto_sort_bookmarks: bool,
    /// Whether smart folders (auto-categorized bookmarks) are enabled.
    #[serde(default = "default_true")]
    pub smart_folders_enabled: bool,

    // --- Downloads ---
    /// Default download directory path.
    #[serde(default = "default_download_path")]
    pub download_path: String,
    /// Whether to ask where to save each file before downloading.
    #[serde(default = "default_true")]
    pub ask_before_download: bool,
    /// Whether to show a notification when a download completes.
    #[serde(default = "default_true")]
    pub show_download_notifications: bool,
    /// Whether to warn before downloading dangerous file types.
    #[serde(default = "default_true")]
    pub warn_dangerous_files: bool,

    // --- Fonts ---
    /// Standard proportional font family.
    #[serde(default = "default_standard_font")]
    pub standard_font: String,
    /// Serif font family.
    #[serde(default = "default_serif_font")]
    pub serif_font: String,
    /// Sans-serif font family.
    #[serde(default = "default_sans_serif_font")]
    pub sans_serif_font: String,
    /// Fixed-width / monospace font family.
    #[serde(default = "default_fixed_width_font")]
    pub fixed_width_font: String,
    /// Minimum font size in pixels.
    #[serde(default = "default_minimum_font_size")]
    pub minimum_font_size: u32,
    /// Default page zoom level (1.0 = 100%).
    #[serde(default = "default_page_zoom")]
    pub default_page_zoom: f32,
    /// Whether to force the browser's font choices over website fonts.
    #[serde(default)]
    pub force_fonts: bool,

    // --- Language ---
    /// UI language code (e.g., "en", "zh-CN").
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    /// Whether to offer to translate pages in foreign languages.
    #[serde(default = "default_true")]
    pub translate_pages: bool,
    /// Whether spell check is enabled.
    #[serde(default = "default_true")]
    pub spell_check_enabled: bool,
    /// Per-site language overrides.
    #[serde(default)]
    pub per_site_language: HashMap<String, String>,

    // --- Reading mode ---
    /// Line width in reading mode (in em units).
    #[serde(default = "default_reading_mode_line_width")]
    pub reading_mode_line_width: f32,
    /// Background color for reading mode (CSS color string).
    #[serde(default = "default_reading_mode_bg_color")]
    pub reading_mode_bg_color: String,
    /// Font size in reading mode (in px).
    #[serde(default = "default_reading_mode_font_size")]
    pub reading_mode_font_size: f32,

    // --- Sidebar ---
    /// Whether the sidebar is visible.
    #[serde(default = "default_true")]
    pub sidebar_visible: bool,
    /// Which side of the window the sidebar appears on.
    #[serde(default = "default_sidebar_position")]
    pub sidebar_position: SidebarPosition,
    /// List of panel IDs currently shown in the sidebar.
    #[serde(default = "default_sidebar_panels")]
    pub sidebar_panels: Vec<String>,

    // =======================================================================
    // 3. Functionality & Efficiency
    // =======================================================================

    // --- Search ---
    /// Default search engine name (matches a key in `search_engines`).
    #[serde(default = "default_search_engine")]
    pub default_search_engine: String,
    /// Search engines map: keyword -> search URL template.
    #[serde(default = "default_search_engines")]
    pub search_engines: HashMap<String, String>,
    /// Whether to show suggestions from the address bar.
    #[serde(default = "default_true")]
    pub address_bar_suggestions: bool,
    /// Whether to include bookmark suggestions in the address bar.
    #[serde(default = "default_true")]
    pub bookmark_suggestions: bool,
    /// Whether to include history suggestions in the address bar.
    #[serde(default = "default_true")]
    pub history_suggestions: bool,
    /// Whether auto-complete is enabled in the address bar.
    #[serde(default = "default_true")]
    pub auto_complete_enabled: bool,
    /// Whether to allow paste-and-search in the address bar.
    #[serde(default = "default_true")]
    pub paste_and_search: bool,

    // --- Tabs ---
    /// Whether tab groups are enabled.
    #[serde(default = "default_true")]
    pub tab_groups_enabled: bool,
    /// Whether to use vertical tabs instead of horizontal.
    #[serde(default)]
    pub vertical_tabs: bool,
    /// Whether to hibernate inactive tabs to save memory.
    #[serde(default = "default_true")]
    pub hibernate_inactive_tabs: bool,
    /// Number of minutes of inactivity before a tab is hibernated.
    #[serde(default = "default_hibernate_after_minutes")]
    pub hibernate_after_minutes: u32,
    /// Whether to show a preview of the tab content on hover.
    #[serde(default = "default_true")]
    pub tab_preview_on_hover: bool,
    /// Whether multi-select (Ctrl/Cmd-click) for tabs is enabled.
    #[serde(default = "default_true")]
    pub multi_select_tabs: bool,
    /// Whether to confirm before closing multiple tabs at once.
    #[serde(default = "default_true")]
    pub confirm_before_closing_multiple: bool,

    // --- History ---
    /// Whether history sync is enabled.
    #[serde(default = "default_true")]
    pub history_sync_enabled: bool,
    /// Whether to automatically clear browsing history.
    #[serde(default)]
    pub auto_clear_history: bool,
    /// Number of days after which history is automatically cleared.
    #[serde(default = "default_auto_clear_after_days")]
    pub auto_clear_after_days: u32,

    // --- Auto-fill ---
    /// Whether to auto-fill addresses.
    #[serde(default = "default_true")]
    pub auto_fill_addresses: bool,
    /// Whether to auto-fill credit card information.
    #[serde(default = "default_true")]
    pub auto_fill_credit_cards: bool,
    /// Whether to auto-fill form history (previously entered values).
    #[serde(default = "default_true")]
    pub auto_fill_form_history: bool,

    // --- Sync ---
    /// Whether Nova Sync is enabled.
    #[serde(default)]
    pub sync_enabled: bool,
    /// Whether to sync bookmarks.
    #[serde(default = "default_true")]
    pub sync_bookmarks: bool,
    /// Whether to sync browsing history.
    #[serde(default = "default_true")]
    pub sync_history: bool,
    /// Whether to sync saved passwords.
    #[serde(default = "default_true")]
    pub sync_passwords: bool,
    /// Whether to sync installed extensions.
    #[serde(default = "default_true")]
    pub sync_extensions: bool,
    /// Whether to sync settings across devices.
    #[serde(default = "default_true")]
    pub sync_settings: bool,
    /// Whether to sync open tabs across devices.
    #[serde(default = "default_true")]
    pub sync_open_tabs: bool,
    /// The email address associated with the sync account.
    #[serde(default)]
    pub sync_account_email: Option<String>,

    // --- Extensions ---
    /// Whether extensions are enabled globally.
    #[serde(default = "default_true")]
    pub extensions_enabled: bool,
    /// Whether developer mode for extensions is enabled.
    #[serde(default)]
    pub developer_mode: bool,
    /// Whether to allow loading unpacked (unpackaged) extensions.
    #[serde(default)]
    pub allow_unpacked_extensions: bool,
    /// Whether to automatically update extensions.
    #[serde(default = "default_true")]
    pub auto_update_extensions: bool,

    // --- PWA (Progressive Web Apps) ---
    /// Whether PWA support is enabled.
    #[serde(default = "default_true")]
    pub pwa_support_enabled: bool,
    /// Whether to show the install prompt for PWAs.
    #[serde(default = "default_true")]
    pub install_prompt_enabled: bool,

    // --- Screenshot ---
    /// Whether full-page screenshot capture is enabled.
    #[serde(default = "default_true")]
    pub screenshot_full_page: bool,
    /// Whether region/crop screenshot is enabled.
    #[serde(default = "default_true")]
    pub screenshot_region: bool,
    /// Whether scrolling screenshot capture is enabled.
    #[serde(default = "default_true")]
    pub screenshot_scrolling: bool,
    /// Whether screenshot annotation tools are enabled.
    #[serde(default = "default_true")]
    pub screenshot_annotations: bool,

    // --- Workspace ---
    /// Whether the workspace feature is enabled.
    #[serde(default = "default_true")]
    pub workspace_enabled: bool,
    /// List of workspace names.
    #[serde(default = "default_workspaces")]
    pub workspaces: Vec<String>,
    /// Whether split view (side-by-side tabs) is enabled.
    #[serde(default = "default_true")]
    pub split_view_enabled: bool,

    // =======================================================================
    // 4. Advanced & Technical
    // =======================================================================

    // --- Proxy ---
    /// The type of proxy to use.
    #[serde(default = "default_proxy_type")]
    pub proxy_type: ProxyType,
    /// Proxy server hostname or IP address.
    #[serde(default)]
    pub proxy_server: Option<String>,
    /// Proxy server port.
    #[serde(default)]
    pub proxy_port: Option<u16>,
    /// Proxy authentication username.
    #[serde(default)]
    pub proxy_username: Option<String>,
    /// Proxy authentication password.
    #[serde(default)]
    pub proxy_password: Option<String>,
    /// URL of the PAC (Proxy Auto-Config) script.
    #[serde(default)]
    pub pac_url: Option<String>,
    /// Whether to bypass the proxy for local addresses.
    #[serde(default = "default_true")]
    pub bypass_local: bool,

    // --- DNS ---
    /// Whether DNS-over-HTTPS (DoH) is enabled.
    #[serde(default)]
    pub dns_over_https_enabled: bool,
    /// DoH provider URL (e.g., "https://cloudflare-dns.com/dns-query").
    #[serde(default)]
    pub doh_provider: Option<String>,
    /// Whether DNS-over-TLS (DoT) is enabled.
    #[serde(default)]
    pub dns_over_tls_enabled: bool,

    // --- Network ---
    /// Whether IPv6 is enabled.
    #[serde(default = "default_true")]
    pub ipv6_enabled: bool,
    /// Whether the QUIC protocol is enabled.
    #[serde(default = "default_true")]
    pub quic_enabled: bool,
    /// Whether HTTP/3 is enabled.
    #[serde(default = "default_true")]
    pub http3_enabled: bool,
    /// Minimum TLS version accepted for connections.
    #[serde(default = "default_tls_min_version")]
    pub tls_min_version: TlsMinVersion,

    // --- Hardware acceleration ---
    /// Whether GPU hardware acceleration is enabled.
    #[serde(default = "default_true")]
    pub gpu_acceleration_enabled: bool,
    /// Whether WebGL is enabled.
    #[serde(default = "default_true")]
    pub webgl_enabled: bool,
    /// Whether canvas 2D acceleration is enabled.
    #[serde(default = "default_true")]
    pub canvas_acceleration: bool,
    /// Whether hardware-accelerated video decoding is enabled.
    #[serde(default = "default_true")]
    pub video_decode_acceleration: bool,

    // --- Content ---
    /// Whether JavaScript execution is enabled.
    #[serde(default = "default_true")]
    pub javascript_enabled: bool,
    /// Whether images are loaded and displayed.
    #[serde(default = "default_true")]
    pub images_enabled: bool,
    /// Whether the popup blocker is enabled.
    #[serde(default = "default_true")]
    pub popup_blocker_enabled: bool,
    /// Whether the built-in PDF viewer is enabled.
    #[serde(default = "default_true")]
    pub pdf_viewer_enabled: bool,
    /// Whether Flash is enabled (legacy).
    #[serde(default)]
    pub flash_enabled: bool,
    /// Autoplay policy for media.
    #[serde(default = "default_autoplay_policy")]
    pub autoplay_policy: AutoplayPolicy,

    // --- User agent ---
    /// Custom user agent string to override the default.
    #[serde(default)]
    pub custom_user_agent: Option<String>,
    /// Per-site user agent overrides.
    #[serde(default)]
    pub per_site_user_agents: HashMap<String, String>,

    // --- Experimental flags ---
    /// Experimental feature flags (name -> enabled).
    #[serde(default)]
    pub experimental_flags: HashMap<String, bool>,

    // --- Developer tools ---
    /// Whether developer tools are enabled.
    #[serde(default = "default_true")]
    pub dev_tools_enabled: bool,
    /// Whether remote debugging is enabled.
    #[serde(default)]
    pub remote_debugging_enabled: bool,
    /// Port for remote debugging connections.
    #[serde(default = "default_remote_debugging_port")]
    pub remote_debugging_port: u16,
    /// Whether Lighthouse audit integration is enabled.
    #[serde(default = "default_true")]
    pub lighthouse_audit_enabled: bool,

    // --- Performance ---
    /// Memory limit in megabytes for the browser process.
    #[serde(default = "default_memory_limit_mb")]
    pub memory_limit_mb: u32,
    /// Maximum number of renderer processes.
    #[serde(default = "default_max_process_count")]
    pub max_process_count: u32,
    /// Page preloading strategy.
    #[serde(default = "default_preload_pages")]
    pub preload_pages: PreloadPages,
    /// Whether energy-saving mode is enabled.
    #[serde(default)]
    pub energy_saving_mode: bool,

    // --- Accessibility ---
    /// Whether screen reader support is enabled.
    #[serde(default)]
    pub screen_reader_support: bool,
    /// Whether high contrast mode is enabled.
    #[serde(default)]
    pub high_contrast_mode: bool,
    /// Whether to reduce animations.
    #[serde(default)]
    pub reduced_animations: bool,
    /// Whether keyboard navigation is enabled.
    #[serde(default = "default_true")]
    pub keyboard_navigation: bool,
    /// Whether caret browsing is enabled.
    #[serde(default)]
    pub caret_browsing: bool,
    /// Whether the focus ring is visible around focused elements.
    #[serde(default = "default_true")]
    pub focus_ring_visible: bool,

    // =======================================================================
    // 5. Mobile-specific
    // =======================================================================

    // --- Gestures ---
    /// Whether swipe navigation (back/forward) is enabled.
    #[serde(default = "default_true")]
    pub swipe_navigation: bool,
    /// Whether pull-to-refresh is enabled on touch devices.
    #[serde(default = "default_true")]
    pub pull_to_refresh: bool,
    /// Whether long-press context menu is enabled.
    #[serde(default = "default_true")]
    pub long_press_menu: bool,
    /// Whether pinch-to-zoom is enabled.
    #[serde(default = "default_true")]
    pub pinch_to_zoom: bool,

    // --- Data saving ---
    /// Whether data saver mode is enabled.
    #[serde(default)]
    pub data_saver_enabled: bool,
    /// Whether to compress images in data saver mode.
    #[serde(default = "default_true")]
    pub compress_images: bool,
    /// Whether to lazy-load images (load only when visible).
    #[serde(default = "default_true")]
    pub lazy_load_images: bool,
    /// Whether to restrict background data usage.
    #[serde(default)]
    pub restrict_background_data: bool,

    // --- Desktop site ---
    /// Whether to force desktop site mode globally.
    #[serde(default)]
    pub force_desktop_site: bool,
    /// Custom mobile user agent string.
    #[serde(default)]
    pub custom_mobile_ua: Option<String>,

    // --- Sharing ---
    /// Whether the native share menu is enabled.
    #[serde(default = "default_true")]
    pub native_share_menu: bool,
    /// Whether the QR code generator is enabled.
    #[serde(default = "default_true")]
    pub qr_code_generator: bool,
}

// ---------------------------------------------------------------------------
// Default implementation
// ---------------------------------------------------------------------------

impl Default for Settings {
    fn default() -> Self {
        Self {
            // -- 1. Privacy & Security --
            allow_third_party_cookies: default_true(),
            cookie_retention_list: Vec::new(),
            block_third_party_cookies_by_default: default_true(),
            tracking_protection_level: default_tracking_protection_level(),
            block_ad_trackers: default_true(),
            block_fingerprinting: default_true(),
            block_social_media_trackers: default_true(),
            custom_tracker_blocklist: Vec::new(),
            force_https: default_true(),
            warn_insecure_connections: default_true(),
            safe_browsing_enabled: default_true(),
            scan_downloads: default_true(),
            phishing_protection: default_true(),
            password_generator_enabled: default_true(),
            auto_fill_passwords: default_true(),
            leak_detection: default_true(),
            use_master_password: false,
            biometric_protection: false,
            per_site_camera: HashMap::new(),
            per_site_microphone: HashMap::new(),
            per_site_location: HashMap::new(),
            per_site_notifications: HashMap::new(),
            per_site_autoplay: HashMap::new(),
            per_site_clipboard: HashMap::new(),
            incognito_clear_on_exit: default_true(),
            block_third_party_in_incognito: default_true(),
            send_dnt_header: false,
            custom_certificates: Vec::new(),
            trust_store_path: None,
            site_isolation_enabled: default_true(),
            strict_isolation: false,

            // -- 2. Appearance & Experience --
            theme_mode: default_theme_mode(),
            custom_accent_color: None,
            custom_theme_path: None,
            new_tab_layout: default_new_tab_layout(),
            show_shortcuts: default_true(),
            new_tab_background_url: None,
            show_search_box: default_true(),
            startup_behavior: default_startup_behavior(),
            custom_startup_pages: Vec::new(),
            bookmarks_bar_visible: default_true(),
            auto_sort_bookmarks: false,
            smart_folders_enabled: default_true(),
            download_path: default_download_path(),
            ask_before_download: default_true(),
            show_download_notifications: default_true(),
            warn_dangerous_files: default_true(),
            standard_font: default_standard_font(),
            serif_font: default_serif_font(),
            sans_serif_font: default_sans_serif_font(),
            fixed_width_font: default_fixed_width_font(),
            minimum_font_size: default_minimum_font_size(),
            default_page_zoom: default_page_zoom(),
            force_fonts: false,
            ui_language: default_ui_language(),
            translate_pages: default_true(),
            spell_check_enabled: default_true(),
            per_site_language: HashMap::new(),
            reading_mode_line_width: default_reading_mode_line_width(),
            reading_mode_bg_color: default_reading_mode_bg_color(),
            reading_mode_font_size: default_reading_mode_font_size(),
            sidebar_visible: default_true(),
            sidebar_position: default_sidebar_position(),
            sidebar_panels: default_sidebar_panels(),

            // -- 3. Functionality & Efficiency --
            default_search_engine: default_search_engine(),
            search_engines: default_search_engines(),
            address_bar_suggestions: default_true(),
            bookmark_suggestions: default_true(),
            history_suggestions: default_true(),
            auto_complete_enabled: default_true(),
            paste_and_search: default_true(),
            tab_groups_enabled: default_true(),
            vertical_tabs: false,
            hibernate_inactive_tabs: default_true(),
            hibernate_after_minutes: default_hibernate_after_minutes(),
            tab_preview_on_hover: default_true(),
            multi_select_tabs: default_true(),
            confirm_before_closing_multiple: default_true(),
            history_sync_enabled: default_true(),
            auto_clear_history: false,
            auto_clear_after_days: default_auto_clear_after_days(),
            auto_fill_addresses: default_true(),
            auto_fill_credit_cards: default_true(),
            auto_fill_form_history: default_true(),
            sync_enabled: false,
            sync_bookmarks: default_true(),
            sync_history: default_true(),
            sync_passwords: default_true(),
            sync_extensions: default_true(),
            sync_settings: default_true(),
            sync_open_tabs: default_true(),
            sync_account_email: None,
            extensions_enabled: default_true(),
            developer_mode: false,
            allow_unpacked_extensions: false,
            auto_update_extensions: default_true(),
            pwa_support_enabled: default_true(),
            install_prompt_enabled: default_true(),
            screenshot_full_page: default_true(),
            screenshot_region: default_true(),
            screenshot_scrolling: default_true(),
            screenshot_annotations: default_true(),
            workspace_enabled: default_true(),
            workspaces: default_workspaces(),
            split_view_enabled: default_true(),

            // -- 4. Advanced & Technical --
            proxy_type: default_proxy_type(),
            proxy_server: None,
            proxy_port: None,
            proxy_username: None,
            proxy_password: None,
            pac_url: None,
            bypass_local: default_true(),
            dns_over_https_enabled: false,
            doh_provider: None,
            dns_over_tls_enabled: false,
            ipv6_enabled: default_true(),
            quic_enabled: default_true(),
            http3_enabled: default_true(),
            tls_min_version: default_tls_min_version(),
            gpu_acceleration_enabled: default_true(),
            webgl_enabled: default_true(),
            canvas_acceleration: default_true(),
            video_decode_acceleration: default_true(),
            javascript_enabled: default_true(),
            images_enabled: default_true(),
            popup_blocker_enabled: default_true(),
            pdf_viewer_enabled: default_true(),
            flash_enabled: false,
            autoplay_policy: default_autoplay_policy(),
            custom_user_agent: None,
            per_site_user_agents: HashMap::new(),
            experimental_flags: HashMap::new(),
            dev_tools_enabled: default_true(),
            remote_debugging_enabled: false,
            remote_debugging_port: default_remote_debugging_port(),
            lighthouse_audit_enabled: default_true(),
            memory_limit_mb: default_memory_limit_mb(),
            max_process_count: default_max_process_count(),
            preload_pages: default_preload_pages(),
            energy_saving_mode: false,
            screen_reader_support: false,
            high_contrast_mode: false,
            reduced_animations: false,
            keyboard_navigation: default_true(),
            caret_browsing: false,
            focus_ring_visible: default_true(),

            // -- 5. Mobile-specific --
            swipe_navigation: default_true(),
            pull_to_refresh: default_true(),
            long_press_menu: default_true(),
            pinch_to_zoom: default_true(),
            data_saver_enabled: false,
            compress_images: default_true(),
            lazy_load_images: default_true(),
            restrict_background_data: false,
            force_desktop_site: false,
            custom_mobile_ua: None,
            native_share_menu: default_true(),
            qr_code_generator: default_true(),
        }
    }
}

// ---------------------------------------------------------------------------
// Default value helper functions
// ---------------------------------------------------------------------------

fn default_true() -> bool {
    true
}

fn default_tracking_protection_level() -> TrackingProtectionLevel {
    TrackingProtectionLevel::Standard
}

fn default_theme_mode() -> ThemeMode {
    ThemeMode::System
}

fn default_new_tab_layout() -> NewTabLayout {
    NewTabLayout::SpeedDial
}

fn default_startup_behavior() -> StartupBehavior {
    StartupBehavior::NewTab
}

fn default_sidebar_position() -> SidebarPosition {
    SidebarPosition::Left
}

fn default_proxy_type() -> ProxyType {
    ProxyType::None
}

fn default_tls_min_version() -> TlsMinVersion {
    TlsMinVersion::Tls1_2
}

fn default_autoplay_policy() -> AutoplayPolicy {
    AutoplayPolicy::BlockAudio
}

fn default_preload_pages() -> PreloadPages {
    PreloadPages::Standard
}

fn default_download_path() -> String {
    dirs::download_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "./downloads".to_string())
}

fn default_standard_font() -> String {
    "Times New Roman".to_string()
}

fn default_serif_font() -> String {
    "Times New Roman".to_string()
}

fn default_sans_serif_font() -> String {
    "Arial".to_string()
}

fn default_fixed_width_font() -> String {
    "Courier New".to_string()
}

fn default_minimum_font_size() -> u32 {
    12
}

fn default_page_zoom() -> f32 {
    1.0
}

fn default_ui_language() -> String {
    "en".to_string()
}

fn default_reading_mode_line_width() -> f32 {
    35.0
}

fn default_reading_mode_bg_color() -> String {
    "#f5f2e8".to_string()
}

fn default_reading_mode_font_size() -> f32 {
    18.0
}

fn default_sidebar_panels() -> Vec<String> {
    vec![
        "bookmarks".to_string(),
        "history".to_string(),
        "downloads".to_string(),
    ]
}

fn default_search_engine() -> String {
    "Google".to_string()
}

fn default_search_engines() -> HashMap<String, String> {
    let mut engines = HashMap::new();
    engines.insert(
        "Google".to_string(),
        "https://www.google.com/search?q=%s".to_string(),
    );
    engines.insert(
        "Bing".to_string(),
        "https://www.bing.com/search?q=%s".to_string(),
    );
    engines.insert(
        "DuckDuckGo".to_string(),
        "https://duckduckgo.com/?q=%s".to_string(),
    );
    engines.insert(
        "Baidu".to_string(),
        "https://www.baidu.com/s?wd=%s".to_string(),
    );
    engines
}

fn default_hibernate_after_minutes() -> u32 {
    30
}

fn default_auto_clear_after_days() -> u32 {
    90
}

fn default_remote_debugging_port() -> u16 {
    9222
}

fn default_memory_limit_mb() -> u32 {
    4096
}

fn default_max_process_count() -> u32 {
    0 // 0 means use system default
}

fn default_workspaces() -> Vec<String> {
    vec!["Personal".to_string(), "Work".to_string()]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();

        // Privacy & Security
        assert_eq!(settings.tracking_protection_level, TrackingProtectionLevel::Standard);
        assert!(settings.block_ad_trackers);
        assert!(settings.block_fingerprinting);
        assert!(settings.force_https);

        // Appearance & Experience
        assert_eq!(settings.theme_mode, ThemeMode::System);
        assert_eq!(settings.new_tab_layout, NewTabLayout::SpeedDial);
        assert_eq!(settings.default_page_zoom, 1.0);

        // Functionality & Efficiency
        assert_eq!(settings.default_search_engine, "Google");
        assert!(settings.search_engines.contains_key("Google"));
        assert!(settings.tab_groups_enabled);

        // Advanced & Technical
        assert_eq!(settings.proxy_type, ProxyType::None);
        assert_eq!(settings.tls_min_version, TlsMinVersion::Tls1_2);
        assert!(settings.javascript_enabled);

        // Mobile-specific
        assert!(settings.swipe_navigation);
        assert!(settings.pinch_to_zoom);
    }

    #[test]
    fn test_serialize_deserialize_json() {
        let settings = Settings::default();
        let json = serde_json::to_string_pretty(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_toml() {
        let settings = Settings::default();
        let toml_str = toml::to_string_pretty(&settings).unwrap();
        let deserialized: Settings = toml::from_str(&toml_str).unwrap();
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_serialize_enum_variants() {
        // Verify kebab-case serialization
        let settings = Settings {
            tracking_protection_level: TrackingProtectionLevel::Strict,
            tls_min_version: TlsMinVersion::Tls1_3,
            autoplay_policy: AutoplayPolicy::BlockAll,
            ..Default::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"strict\""));
        assert!(json.contains("\"tls1.3\""));
        assert!(json.contains("\"block-all\""));
    }

    #[test]
    fn test_custom_overrides() {
        let settings = Settings {
            force_https: false,
            javascript_enabled: false,
            custom_user_agent: Some("Mozilla/5.0 Custom".to_string()),
            ..Default::default()
        };
        assert!(!settings.force_https);
        assert!(!settings.javascript_enabled);
        assert_eq!(settings.custom_user_agent.as_deref(), Some("Mozilla/5.0 Custom"));
    }
}