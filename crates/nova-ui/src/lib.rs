// Nova Browser UI Library
// Chrome UI rendering using egui - modern, clean, cross-platform

pub mod address_bar;
pub mod chrome;
pub mod settings_ui;
pub mod sidebar;
pub mod tab_bar;
pub mod theme;
pub mod widgets;

use egui::Context;
use nova_core::ThemeMode;

/// Global UI state
pub struct UiState {
    /// Current theme mode
    pub theme_mode: ThemeMode,
    /// Whether the sidebar is visible
    pub sidebar_visible: bool,
    /// Which sidebar panel is active (0: bookmarks, 1: history, 2: extensions, 3: downloads, 4: passwords)
    pub sidebar_panel: usize,
    /// Whether the address bar is focused
    pub address_bar_focused: bool,
    /// Search suggestions
    pub search_suggestions: Vec<String>,
    /// Whether reading mode is active
    pub reading_mode: bool,
    /// Reading mode settings
    pub reading_mode_settings: ReadingModeSettings,
    /// Whether split view is active
    pub split_view: bool,
    /// Split view ratio
    pub split_ratio: f32,
    /// Current device type
    pub device_type: DeviceType,
    /// Animation state for transitions
    pub animation_state: f32,
    /// Whether a new tab animation is playing
    pub tab_animation: bool,
    /// Notification message
    pub notification: Option<Notification>,
    /// Current locale
    pub locale: String,
}

#[derive(Debug, Clone)]
pub struct ReadingModeSettings {
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub background_color: [u8; 4],
    pub text_color: [u8; 4],
    pub max_width: f32,
}

impl Default for ReadingModeSettings {
    fn default() -> Self {
        Self {
            font_family: String::from("sans-serif"),
            font_size: 18.0,
            line_height: 1.8,
            background_color: [248, 245, 240, 255],
            text_color: [51, 51, 51, 255],
            max_width: 720.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Desktop,
    Tablet,
    Mobile,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub duration: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::System,
            sidebar_visible: true,
            sidebar_panel: 0,
            address_bar_focused: false,
            search_suggestions: Vec::new(),
            reading_mode: false,
            reading_mode_settings: ReadingModeSettings::default(),
            split_view: false,
            split_ratio: 0.5,
            device_type: DeviceType::Desktop,
            animation_state: 0.0,
            tab_animation: false,
            notification: None,
            locale: String::from("en"),
        }
    }
}

impl UiState {
    pub fn new(theme_mode: ThemeMode, locale: &str) -> Self {
        Self {
            theme_mode,
            locale: locale.to_string(),
            ..Default::default()
        }
    }

    /// Update animation state (smooth transitions)
    pub fn update_animation(&mut self, dt: f32) {
        self.animation_state = (self.animation_state + dt * 4.0).min(1.0);
        if let Some(ref mut notif) = self.notification {
            notif.duration -= dt;
            if notif.duration <= 0.0 {
                self.notification = None;
            }
        }
    }

    pub fn show_notification(&mut self, message: &str, ntype: NotificationType) {
        self.notification = Some(Notification {
            message: message.to_string(),
            notification_type: ntype,
            duration: 3.0,
        });
    }
}

/// Initialize the egui context with the Nova theme
pub fn initialize_egui_context(ctx: &Context, theme_mode: ThemeMode) {
    let is_dark = theme_mode.is_dark();
    theme::apply_theme(ctx, is_dark);
    theme::configure_fonts(ctx);
}

/// Get translated text for a key
pub fn tr(key: &str, locale: &str) -> String {
    // This will delegate to the locale system
    // For now, return the key itself
    key.to_string()
}