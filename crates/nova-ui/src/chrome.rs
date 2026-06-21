// Main chrome layout - the browser shell UI

use egui::{Align, Color32, Context, Frame, Layout, Margin, Rect, Rounding, Stroke, Ui, Vec2};
use nova_core::ThemeMode;

use crate::{
    address_bar::AddressBar,
    sidebar::Sidebar,
    tab_bar::TabBar,
    theme::{get_palette, NovaPalette},
    DeviceType, Notification, NotificationType, UiState,
};

/// Renders the complete browser chrome
pub struct Chrome;

impl Chrome {
    /// Render the entire browser chrome UI
    pub fn render(
        ctx: &Context,
        ui_state: &mut UiState,
        tabs: &[nova_core::TabInfo],
        active_tab_id: Option<uuid::Uuid>,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        let palette = get_palette(ui_state.theme_mode.is_dark());
        let device = ui_state.device_type;

        match device {
            DeviceType::Desktop => Self::render_desktop(ctx, ui_state, tabs, active_tab_id, palette, on_action),
            DeviceType::Tablet => Self::render_tablet(ctx, ui_state, tabs, active_tab_id, palette, on_action),
            DeviceType::Mobile => Self::render_mobile(ctx, ui_state, tabs, active_tab_id, palette, on_action),
        }
    }

    /// Desktop layout: top toolbar + side panel + main content
    fn render_desktop(
        ctx: &Context,
        ui_state: &mut UiState,
        tabs: &[nova_core::TabInfo],
        active_tab_id: Option<uuid::Uuid>,
        palette: &'static NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        // Top toolbar area
        egui::TopBottomPanel::top("nova_toolbar")
            .frame(Frame {
                fill: palette.toolbar_bg,
                inner_margin: Margin::symmetric(0.0, 0.0),
                outer_margin: Margin::default(),
                rounding: Rounding::default(),
                shadow: egui::epaint::Shadow {
                    offset: [0.0, 1.0].into(),
                    blur: 3.0,
                    spread: 0.0,
                    color: palette.shadow_color,
                },
                stroke: Stroke::new(1.0, palette.border),
            })
            .show(ctx, |ui| {
                Self::render_toolbar(ui, ui_state, tabs, active_tab_id, palette, on_action);
            });

        // Sidebar
        if ui_state.sidebar_visible {
            egui::SidePanel::left("nova_sidebar")
                .frame(Frame {
                    fill: palette.sidebar_bg,
                    inner_margin: Margin::same(0.0),
                    outer_margin: Margin::default(),
                    rounding: Rounding::default(),
                    shadow: egui::epaint::Shadow {
                        offset: [1.0, 0.0].into(),
                        blur: 3.0,
                        spread: 0.0,
                        color: palette.shadow_color,
                    },
                    stroke: Stroke::new(1.0, palette.border),
                })
                .min_width(240.0)
                .max_width(360.0)
                .resizable(true)
                .show(ctx, |ui| {
                    Sidebar::render(ui, ui_state, palette, on_action);
                });
        }

        // Main content area
        egui::CentralPanel::default()
            .frame(Frame {
                fill: palette.bg_primary,
                ..Default::default()
            })
            .show(ctx, |ui| {
                Self::render_content_area(ui, ui_state, palette, on_action);
            });

        // Notification overlay
        Self::render_notification(ctx, ui_state, palette);

        // Status bar at bottom
        egui::TopBottomPanel::bottom("nova_statusbar")
            .frame(Frame {
                fill: palette.toolbar_bg,
                inner_margin: Margin::symmetric(12.0, 4.0),
                outer_margin: Margin::default(),
                rounding: Rounding::default(),
                stroke: Stroke::new(1.0, palette.border),
                ..Default::default()
            })
            .min_height(28.0)
            .show(ctx, |ui| {
                Self::render_status_bar(ui, palette);
            });
    }

    /// Tablet layout: similar to desktop but more compact
    fn render_tablet(
        ctx: &Context,
        ui_state: &mut UiState,
        tabs: &[nova_core::TabInfo],
        active_tab_id: Option<uuid::Uuid>,
        palette: &'static NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        // Similar to desktop but with larger touch targets
        // and collapsible sidebar
        Self::render_desktop(ctx, ui_state, tabs, active_tab_id, palette, on_action);
    }

    /// Mobile layout: bottom toolbar + compact chrome
    fn render_mobile(
        ctx: &Context,
        ui_state: &mut UiState,
        tabs: &[nova_core::TabInfo],
        active_tab_id: Option<uuid::Uuid>,
        palette: &'static NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        // Top compact toolbar
        egui::TopBottomPanel::top("nova_mobile_top")
            .frame(Frame {
                fill: palette.toolbar_bg,
                inner_margin: Margin::symmetric(8.0, 4.0),
                outer_margin: Margin::default(),
                rounding: Rounding::default(),
                stroke: Stroke::new(1.0, palette.border),
                ..Default::default()
            })
            .show(ctx, |ui| {
                Self::render_mobile_top_bar(ui, ui_state, palette, on_action);
            });

        // Main content
        egui::CentralPanel::default()
            .frame(Frame {
                fill: palette.bg_primary,
                ..Default::default()
            })
            .show(ctx, |ui| {
                Self::render_content_area(ui, ui_state, palette, on_action);
            });

        // Bottom navigation bar
        egui::TopBottomPanel::bottom("nova_mobile_bottom")
            .frame(Frame {
                fill: palette.toolbar_bg,
                inner_margin: Margin::symmetric(12.0, 8.0),
                outer_margin: Margin::default(),
                rounding: Rounding::default(),
                stroke: Stroke::new(1.0, palette.border),
                ..Default::default()
            })
            .show(ctx, |ui| {
                Self::render_mobile_bottom_bar(ui, ui_state, palette, on_action);
            });
    }

    /// Render the main toolbar (tab bar + address bar)
    fn render_toolbar(
        ui: &mut Ui,
        ui_state: &mut UiState,
        tabs: &[nova_core::TabInfo],
        active_tab_id: Option<uuid::Uuid>,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        // Tab bar
        ui.horizontal(|ui| {
            TabBar::render(ui, ui_state, tabs, active_tab_id, palette, on_action);
        });

        // Address bar row
        ui.horizontal(|ui| {
            AddressBar::render(ui, ui_state, palette, on_action);
        });
    }

    /// Render the main content area
    fn render_content_area(
        ui: &mut Ui,
        ui_state: &UiState,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        if ui_state.split_view {
            // Split view - two content panes side by side
            let available = ui.available_size();
            let split_x = available.x * ui_state.split_ratio;

            ui.horizontal(|ui| {
                // Left pane
                ui.allocate_ui(
                    egui::Vec2::new(split_x, available.y),
                    |ui| {
                        Self::render_web_content_placeholder(ui, palette, "Left Pane");
                    },
                );

                // Split divider
                ui.allocate_ui(
                    egui::Vec2::new(4.0, available.y),
                    |ui| {
                        let response = ui.allocate_response(
                            egui::Vec2::new(4.0, available.y),
                            egui::Sense::drag(),
                        );
                        if response.dragged() {
                            on_action(ChromeAction::SplitResize);
                        }
                        ui.painter().rect_filled(
                            response.rect,
                            Rounding::same(2.0),
                            palette.border,
                        );
                    },
                );

                // Right pane
                ui.allocate_ui(
                    egui::Vec2::new(available.x - split_x - 4.0, available.y),
                    |ui| {
                        Self::render_web_content_placeholder(ui, palette, "Right Pane");
                    },
                );
            });
        } else {
            // Single content pane
            Self::render_web_content_placeholder(ui, palette, "Web Content");
        }
    }

    /// Placeholder for web content area
    fn render_web_content_placeholder(ui: &mut Ui, palette: &NovaPalette, label: &str) {
        let rect = ui.available_rect_before_wrap();
        let painter = ui.painter();

        // Draw content area background
        painter.rect_filled(rect, Rounding::same(0.0), palette.bg_primary);

        // Draw a subtle grid pattern for empty state
        let center = rect.center();
        painter.text(
            center - egui::Vec2::new(60.0, 0.0),
            egui::Align2::CENTER_CENTER,
            "Nova Browser",
            egui::FontId::proportional(28.0),
            palette.text_tertiary,
        );

        painter.text(
            center + egui::Vec2::new(0.0, 40.0),
            egui::Align2::CENTER_CENTER,
            "Enter a URL or search to get started",
            egui::FontId::proportional(14.0),
            palette.text_secondary,
        );
    }

    /// Render mobile top bar
    fn render_mobile_top_bar(
        ui: &mut Ui,
        ui_state: &mut UiState,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        ui.horizontal(|ui| {
            // Tab switcher icon
            if ui
                .add(egui::Button::new("≡").fill(Color32::TRANSPARENT))
                .clicked()
            {
                on_action(ChromeAction::ToggleTabOverview);
            }

            // Compact address bar
            AddressBar::render_compact(ui, ui_state, palette, on_action);

            // Menu
            if ui
                .add(egui::Button::new("⋯").fill(Color32::TRANSPARENT))
                .clicked()
            {
                on_action(ChromeAction::ToggleMenu);
            }
        });
    }

    /// Render mobile bottom navigation bar
    fn render_mobile_bottom_bar(
        ui: &mut Ui,
        ui_state: &UiState,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                if ui.button("◀").clicked() {
                    on_action(ChromeAction::GoBack);
                }
            });
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                if ui.button("▶").clicked() {
                    on_action(ChromeAction::GoForward);
                }
            });
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                if ui.button("🏠").clicked() {
                    on_action(ChromeAction::GoHome);
                }
            });
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                if ui.button("📑").clicked() {
                    on_action(ChromeAction::ToggleTabOverview);
                }
            });
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                if ui.button("⚙").clicked() {
                    on_action(ChromeAction::ToggleMenu);
                }
            });
        });
    }

    /// Render the status bar at the bottom
    fn render_status_bar(ui: &mut Ui, palette: &NovaPalette) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Ready")
                    .size(11.0)
                    .color(palette.text_secondary),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("100%")
                        .size(11.0)
                        .color(palette.text_secondary),
                );
            });
        });
    }

    /// Render notification overlay
    fn render_notification(ctx: &Context, ui_state: &UiState, palette: &NovaPalette) {
        if let Some(notif) = &ui_state.notification {
            let color = match notif.notification_type {
                NotificationType::Success => palette.success,
                NotificationType::Warning => palette.warning,
                NotificationType::Error => palette.error,
                NotificationType::Info => palette.info,
            };

            egui::Area::new("notification".into())
                .anchor(egui::Align2::RIGHT_TOP, [-16.0, 16.0])
                .show(ctx, |ui| {
                    egui::Frame::group(ui.style())
                        .fill(palette.surface)
                        .rounding(Rounding::same(8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(color, "●");
                                ui.label(&notif.message);
                            });
                        });
                });
        }
    }
}

/// Actions that the chrome UI can emit
#[derive(Debug, Clone)]
pub enum ChromeAction {
    // Navigation
    Navigate(String),
    GoBack,
    GoForward,
    GoHome,
    Refresh,
    Stop,

    // Tab management
    NewTab,
    NewIncognitoTab,
    CloseTab(uuid::Uuid),
    SelectTab(uuid::Uuid),
    PinTab(uuid::Uuid),
    MuteTab(uuid::Uuid),
    ReorderTabs(usize, usize),
    CloseAllTabs,
    ReopenClosedTab,
    ToggleTabOverview,

    // Sidebar
    ToggleSidebar,
    SelectSidebarPanel(usize),

    // Features
    ToggleBookmark,
    ToggleReadingMode,
    ToggleSplitView,
    ToggleIncognito,
    ToggleDarkMode,
    ToggleMenu,

    // Split view
    SplitResize,
    SplitClose,

    // Extensions
    OpenExtensionManager,
    InstallExtension(String),

    // Dev tools
    OpenDevTools,
    ToggleElementInspector,
    ToggleNetworkMonitor,
    TogglePerformanceProfiler,

    // Downloads
    OpenDownloads,
    ClearDownloads,

    // History
    OpenHistory,
    ClearHistory,

    // Bookmarks
    OpenBookmarks,
    AddBookmark(String, String),
    RemoveBookmark(String),

    // Passwords
    OpenPasswordManager,
    GeneratePassword,
    AutoFillPassword,

    // Settings
    OpenSettings,
    ChangeLanguage(String),
    ChangeTheme(ThemeMode),

    // Search
    Search(String),
    VoiceSearch,
    SearchSuggestionSelected(String),

    // Zoom
    ZoomIn,
    ZoomOut,
    ZoomReset,

    // Other
    Print,
    Find,
    Share,
    None,
}