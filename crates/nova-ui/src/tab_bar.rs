// Tab bar component
// Manages browser tabs with drag, close, preview, and pinning support

use egui::{Color32, Frame, Layout, Margin, Rect, Rounding, Stroke, Ui, Vec2};
use nova_core::TabInfo;
use uuid::Uuid;

use crate::{
    chrome::ChromeAction,
    theme::{corner_radius, get_palette, NovaPalette},
    UiState,
};

pub struct TabBar;

impl TabBar {
    /// Render the tab bar
    pub fn render(
        ui: &mut Ui,
        ui_state: &mut UiState,
        tabs: &[TabInfo],
        active_tab_id: Option<Uuid>,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        let available_width = ui.available_width();
        let tab_height = 36.0;
        let tab_width = (available_width - 40.0) / tabs.len().max(1) as f32;
        let tab_width = tab_width.clamp(120.0, 240.0);

        // Scrollable tab area
        ui.horizontal(|ui| {
            // New tab button
            let new_tab_btn = egui::Button::new("+")
                .fill(Color32::TRANSPARENT)
                .min_size(Vec2::new(32.0, tab_height));

            if ui.add(new_tab_btn).clicked() {
                on_action(ChromeAction::NewTab);
            }

            ui.separator();

            // Render each tab
            for (idx, tab) in tabs.iter().enumerate() {
                let is_active = active_tab_id.map_or(false, |id| id == tab.id);

                let tab_response = Self::render_tab(
                    ui,
                    tab,
                    is_active,
                    tab_width,
                    tab_height,
                    palette,
                    ui_state,
                    on_action,
                );

                // Handle tab interactions
                if tab_response.clicked() {
                    on_action(ChromeAction::SelectTab(tab.id));
                }

                if tab_response.secondary_clicked() {
                    on_action(ChromeAction::CloseTab(tab.id));
                }
            }
        });
    }

    /// Render a single tab
    fn render_tab(
        ui: &mut Ui,
        tab: &TabInfo,
        is_active: bool,
        width: f32,
        height: f32,
        palette: &NovaPalette,
        ui_state: &UiState,
        on_action: &mut dyn FnMut(ChromeAction),
    ) -> egui::Response {
        let desired_size = Vec2::new(width, height);
        let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Tab background
            let bg_color = if is_active {
                palette.tab_active_bg
            } else if response.hovered() {
                palette.tab_hover_bg
            } else {
                palette.tab_inactive_bg
            };

            // Draw tab background with rounded top corners
            let tab_rect = Rect::from_min_size(rect.min, Vec2::new(width, height));
            painter.rect_filled(
                tab_rect,
                Rounding {
                    nw: corner_radius(),
                    ne: corner_radius(),
                    sw: 0.0,
                    se: 0.0,
                },
                bg_color,
            );

            // Active tab indicator
            if is_active {
                let indicator_rect = Rect::from_min_size(
                    tab_rect.min + Vec2::new(0.0, tab_rect.height() - 2.0),
                    Vec2::new(tab_rect.width(), 2.0),
                );
                painter.rect_filled(indicator_rect, Rounding::same(1.0), palette.accent_primary);
            }

            // Separator line between tabs
            if !is_active {
                painter.line_segment(
                    [
                        tab_rect.right_top(),
                        tab_rect.right_bottom(),
                    ],
                    Stroke::new(0.5, palette.border),
                );
            }

            // Favicon
            let icon_pos = tab_rect.min + Vec2::new(8.0, 10.0);
            if let Some(ref favicon) = tab.favicon {
                painter.text(
                    icon_pos + Vec2::new(8.0, 8.0),
                    egui::Align2::LEFT_CENTER,
                    favicon,
                    egui::FontId::proportional(12.0),
                    palette.text_primary,
                );
            } else {
                painter.text(
                    icon_pos + Vec2::new(8.0, 8.0),
                    egui::Align2::LEFT_CENTER,
                    "🌐",
                    egui::FontId::proportional(12.0),
                    palette.text_secondary,
                );
            }

            // Tab title
            let title_x = tab_rect.min.x + 32.0;
            let title_max_width = tab_rect.width() - 60.0;
            let title = if tab.is_loading {
                format!("{} ⏳", tab.title)
            } else if tab.is_pinned {
                format!("📌 {}", tab.title)
            } else {
                tab.title.clone()
            };

            let title_galley = painter.layout_no_wrap(
                title.clone(),
                egui::FontId::proportional(12.0),
                palette.text_primary,
            );

            if title_galley.size().x > title_max_width {
                // Truncate text
                let truncated = format!("{}…", &title[..(title.len().min(20))]);
                painter.text(
                    egui::Pos2::new(title_x, tab_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &truncated,
                    egui::FontId::proportional(12.0),
                    palette.text_primary,
                );
            } else {
                painter.text(
                    egui::Pos2::new(title_x, tab_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &title,
                    egui::FontId::proportional(12.0),
                    palette.text_primary,
                );
            }

            // Close button
            let close_rect = Rect::from_center_size(
                tab_rect.right_top() + Vec2::new(-16.0, height / 2.0),
                Vec2::new(16.0, 16.0),
            );

            let close_hovered = ui
                .input(|i| i.pointer.hover_pos())
                .map_or(false, |pos| close_rect.contains(pos));

            let close_color = if close_hovered {
                palette.error
            } else {
                palette.text_tertiary
            };

            painter.text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                "×",
                egui::FontId::proportional(14.0),
                close_color,
            );

            // Handle close button click
            if close_hovered && response.clicked() {
                // Check if click was on close button
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    if close_rect.contains(pos) {
                        on_action(ChromeAction::CloseTab(tab.id));
                    }
                }
            }

            // Mute indicator
            if tab.is_muted {
                painter.text(
                    tab_rect.right_top() + Vec2::new(-28.0, 10.0),
                    egui::Align2::CENTER_CENTER,
                    "🔇",
                    egui::FontId::proportional(10.0),
                    palette.text_secondary,
                );
            }
        }

        response
    }

    /// Render mobile tab overview (grid of tabs)
    pub fn render_tab_overview(
        ui: &mut Ui,
        tabs: &[TabInfo],
        active_tab_id: Option<Uuid>,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        ui.heading(
            egui::RichText::new("Tabs")
                .size(18.0)
                .color(palette.text_primary),
        );

        ui.add_space(8.0);

        // Close all tabs button
        if ui
            .button(
                egui::RichText::new("Close All Tabs")
                    .color(palette.error),
            )
            .clicked()
        {
            on_action(ChromeAction::CloseAllTabs);
        }

        ui.add_space(12.0);

        // Grid of tab cards
        egui::ScrollArea::vertical().show(ui, |ui| {
            for tab in tabs {
                let is_active = active_tab_id.map_or(false, |id| id == tab.id);

                let response = egui::Frame::group(ui.style())
                    .fill(if is_active {
                        palette.accent_tertiary
                    } else {
                        palette.surface
                    })
                    .rounding(Rounding::same(corner_radius()))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Favicon
                            ui.label(
                                tab.favicon
                                    .as_deref()
                                    .unwrap_or("🌐"),
                            );

                            // Title and URL
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(&tab.title)
                                        .size(14.0)
                                        .color(palette.text_primary),
                                );
                                ui.label(
                                    egui::RichText::new(&tab.url)
                                        .size(11.0)
                                        .color(palette.text_secondary),
                                );
                            });

                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui
                                    .button(
                                        egui::RichText::new("×").color(palette.error),
                                    )
                                    .clicked()
                                {
                                    on_action(ChromeAction::CloseTab(tab.id));
                                }
                            });
                        });
                    });

                if response.response.clicked() {
                    on_action(ChromeAction::SelectTab(tab.id));
                }
            }
        });
    }
}