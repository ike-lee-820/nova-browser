// Sidebar component
// Integrated panels for bookmarks, history, extensions, downloads, and passwords

use egui::{Color32, Frame, Layout, Margin, Rect, Rounding, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};

use crate::{
    chrome::ChromeAction,
    theme::{corner_radius, get_palette, NovaPalette},
    UiState,
};

pub struct Sidebar;

/// Sidebar panel types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarPanel {
    Bookmarks = 0,
    History = 1,
    Extensions = 2,
    Downloads = 3,
    Passwords = 4,
}

impl Sidebar {
    /// Render the sidebar
    pub fn render(
        ui: &mut Ui,
        ui_state: &mut UiState,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        ui.vertical(|ui| {
            // Panel selector tabs
            Self::render_panel_tabs(ui, ui_state, palette, on_action);

            ui.separator();

            // Panel content
            match ui_state.sidebar_panel {
                0 => Self::render_bookmarks(ui, palette, on_action),
                1 => Self::render_history(ui, palette, on_action),
                2 => Self::render_extensions(ui, palette, on_action),
                3 => Self::render_downloads(ui, palette, on_action),
                4 => Self::render_passwords(ui, palette, on_action),
                _ => {}
            }
        });
    }

    /// Render the panel selector tabs at the top of the sidebar
    fn render_panel_tabs(
        ui: &mut Ui,
        ui_state: &mut UiState,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        ui.horizontal(|ui| {
            let tabs = ["📑", "🕐", "🧩", "⬇", "🔑"];
            for (i, icon) in tabs.iter().enumerate() {
                let is_active = ui_state.sidebar_panel == i;
                let bg = if is_active {
                    palette.accent_tertiary
                } else {
                    Color32::TRANSPARENT
                };

                let btn = egui::Button::new(*icon)
                    .fill(bg)
                    .min_size(Vec2::new(40.0, 32.0));

                if ui.add(btn).clicked() {
                    ui_state.sidebar_panel = i;
                    on_action(ChromeAction::SelectSidebarPanel(i));
                }
            }
        });
    }

    /// Render bookmarks panel
    fn render_bookmarks(
        ui: &mut Ui,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        ui.heading(
            egui::RichText::new("Bookmarks")
                .size(16.0)
                .color(palette.text_primary),
        );

        ui.add_space(8.0);

        // Add bookmark bar
        if ui
            .button(
                egui::RichText::new("+ Add Bookmark")
                    .color(palette.accent_primary),
            )
            .clicked()
        {
            on_action(ChromeAction::AddBookmark(
                String::new(),
                String::new(),
            ));
        }

        ui.add_space(8.0);

        // Sample bookmarks
        let sample_bookmarks = vec![
            ("📁", "Bookmarks Bar", true),
            ("📁", "Work", true),
            ("📁", "Personal", true),
            ("🔗", "GitHub", false),
            ("🔗", "Stack Overflow", false),
            ("🔗", "Rust Documentation", false),
            ("🔗", "MDN Web Docs", false),
        ];

        for (icon, name, is_folder) in sample_bookmarks {
            let response = egui::Frame::none()
                .fill(Color32::TRANSPARENT)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if is_folder {
                            if ui
                                .button(
                                    egui::RichText::new("▶").color(palette.text_secondary),
                                )
                                .clicked()
                            {
                                // Toggle folder expand
                            }
                        }
                        ui.label(
                            egui::RichText::new(format!("{} {}", icon, name))
                                .size(13.0)
                                .color(palette.text_primary),
                        );
                    });
                });

            if response.response.hovered() {
                ui.painter().rect_filled(
                    response.response.rect,
                    Rounding::same(4.0),
                    palette.bg_hover,
                );
            }
        }
    }

    /// Render history panel
    fn render_history(
        ui: &mut Ui,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        ui.heading(
            egui::RichText::new("History")
                .size(16.0)
                .color(palette.text_primary),
        );

        ui.add_space(8.0);

        // Search history
        ui.text_edit_singleline(&mut String::new());

        ui.add_space(4.0);

        // Clear history button
        if ui
            .button(
                egui::RichText::new("Clear History")
                    .size(12.0)
                    .color(palette.text_secondary),
            )
            .clicked()
        {
            on_action(ChromeAction::ClearHistory);
        }

        ui.add_space(8.0);

        // Sample history items grouped by date
        let history_groups = vec![
            ("Today", vec![
                ("Rust Programming Language", "https://www.rust-lang.org", "10:30"),
                ("egui - GitHub", "https://github.com/emilk/egui", "09:15"),
                ("CEF Documentation", "https://bitbucket.org/chromiumembedded/cef", "08:00"),
            ]),
            ("Yesterday", vec![
                ("MDN Web Docs", "https://developer.mozilla.org", "16:45"),
                ("crates.io", "https://crates.io", "14:20"),
            ]),
        ];

        for (date, items) in history_groups {
            ui.label(
                egui::RichText::new(date)
                    .size(12.0)
                    .color(palette.text_secondary),
            );

            for (title, url, time) in items {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(time)
                            .size(11.0)
                            .color(palette.text_tertiary),
                    );
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(title)
                                .size(13.0)
                                .color(palette.text_primary),
                        );
                        ui.label(
                            egui::RichText::new(url)
                                .size(11.0)
                                .color(palette.text_secondary),
                        );
                    });
                });
            }

            ui.add_space(4.0);
        }
    }

    /// Render extensions panel
    fn render_extensions(
        ui: &mut Ui,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        ui.heading(
            egui::RichText::new("Extensions")
                .size(16.0)
                .color(palette.text_primary),
        );

        ui.add_space(8.0);

        // Open extension store
        if ui
            .button(
                egui::RichText::new("🧩 Extension Center")
                    .color(palette.accent_primary),
            )
            .clicked()
        {
            on_action(ChromeAction::OpenExtensionManager);
        }

        ui.add_space(8.0);

        // Sample installed extensions
        let extensions = vec![
            ("🔒", "Password Manager", "v2.1.0", true),
            ("🛡", "Ad Blocker Pro", "v4.0.2", true),
            ("🌐", "Translate Helper", "v1.3.5", true),
            ("📝", "Note Taker", "v0.9.1", false),
            ("🎨", "Theme Customizer", "v1.0.0", true),
        ];

        for (icon, name, version, enabled) in extensions {
            egui::Frame::group(ui.style())
                .fill(palette.surface)
                .rounding(Rounding::same(corner_radius()))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(icon);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(name)
                                    .size(13.0)
                                    .color(palette.text_primary),
                            );
                            ui.label(
                                egui::RichText::new(version)
                                    .size(11.0)
                                    .color(palette.text_secondary),
                            );
                        });
                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            let toggle_text = if enabled { "ON" } else { "OFF" };
                            ui.label(
                                egui::RichText::new(toggle_text)
                                    .size(11.0)
                                    .color(if enabled {
                                        palette.success
                                    } else {
                                        palette.text_tertiary
                                    }),
                            );
                        });
                    });
                });
        }
    }

    /// Render downloads panel
    fn render_downloads(
        ui: &mut Ui,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        ui.heading(
            egui::RichText::new("Downloads")
                .size(16.0)
                .color(palette.text_primary),
        );

        ui.add_space(8.0);

        // Open downloads folder
        if ui
            .button(
                egui::RichText::new("📂 Open Downloads Folder")
                    .color(palette.accent_primary),
            )
            .clicked()
        {
            on_action(ChromeAction::OpenDownloads);
        }

        if ui
            .button(
                egui::RichText::new("Clear All")
                    .size(12.0)
                    .color(palette.text_secondary),
            )
            .clicked()
        {
            on_action(ChromeAction::ClearDownloads);
        }

        ui.add_space(8.0);

        // Sample downloads
        let downloads = vec![
            ("📄", "nova-browser-setup.exe", "125 MB", "Completed", true),
            ("🖼", "screenshot-2026.png", "2.4 MB", "Completed", true),
            ("📦", "project.zip", "48 MB", "In Progress", false),
            ("🎵", "podcast-ep42.mp3", "85 MB", "Paused", false),
        ];

        for (icon, name, size, status, completed) in downloads {
            egui::Frame::group(ui.style())
                .fill(palette.surface)
                .rounding(Rounding::same(corner_radius()))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(icon);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(name)
                                    .size(13.0)
                                    .color(palette.text_primary),
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(size)
                                        .size(11.0)
                                        .color(palette.text_secondary),
                                );
                                ui.label(
                                    egui::RichText::new(status)
                                        .size(11.0)
                                        .color(if completed {
                                            palette.success
                                        } else {
                                            palette.warning
                                        }),
                                );
                            });
                        });
                    });
                });
        }
    }

    /// Render password manager panel
    fn render_passwords(
        ui: &mut Ui,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        ui.heading(
            egui::RichText::new("Password Manager")
                .size(16.0)
                .color(palette.text_primary),
        );

        ui.add_space(8.0);

        // Generate password
        if ui
            .button(
                egui::RichText::new("🔑 Generate Password")
                    .color(palette.accent_primary),
            )
            .clicked()
        {
            on_action(ChromeAction::GeneratePassword);
        }

        ui.add_space(8.0);

        // Master password status
        ui.label(
            egui::RichText::new("🔒 Master Password: Set")
                .size(12.0)
                .color(palette.success),
        );

        ui.add_space(8.0);

        // Sample saved passwords
        let saved_passwords = vec![
            ("github.com", "user@example.com", "••••••••"),
            ("google.com", "user@gmail.com", "••••••••"),
            ("stackoverflow.com", "dev@example.com", "••••••••"),
        ];

        for (site, username, password) in saved_passwords {
            egui::Frame::group(ui.style())
                .fill(palette.surface)
                .rounding(Rounding::same(corner_radius()))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(site)
                                .size(14.0)
                                .color(palette.text_primary),
                        );
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(username)
                                    .size(12.0)
                                    .color(palette.text_secondary),
                            );
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new(password)
                                        .size(12.0)
                                        .color(palette.text_tertiary),
                                );
                                if ui
                                    .button(
                                        egui::RichText::new("👁").color(palette.accent_primary),
                                    )
                                    .clicked()
                                {
                                    // Show password
                                }
                                if ui
                                    .button(
                                        egui::RichText::new("📋").color(palette.accent_primary),
                                    )
                                    .clicked()
                                {
                                    on_action(ChromeAction::AutoFillPassword);
                                }
                            });
                        });
                    });
                });
        }
    }
}