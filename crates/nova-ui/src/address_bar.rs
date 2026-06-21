// Address bar component
// Smart search, navigation, and action buttons

use egui::{Color32, Frame, Layout, Margin, Rect, Rounding, Stroke, Ui, Vec2};
use url::Url;

use crate::{
    chrome::ChromeAction,
    theme::{corner_radius, get_palette, NovaPalette},
    UiState,
};

pub struct AddressBar;

impl AddressBar {
    /// Render the full address bar for desktop/tablet
    pub fn render(
        ui: &mut Ui,
        ui_state: &mut UiState,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        let bar_height = 36.0;

        ui.horizontal(|ui| {
            // Navigation buttons
            Self::render_nav_buttons(ui, palette, on_action);

            ui.add_space(4.0);

            // Address/search bar
            Self::render_url_bar(ui, ui_state, palette, bar_height, on_action);

            ui.add_space(4.0);

            // Action buttons
            Self::render_action_buttons(ui, ui_state, palette, on_action);
        });
    }

    /// Render a compact address bar for mobile
    pub fn render_compact(
        ui: &mut Ui,
        ui_state: &mut UiState,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        let available = ui.available_width();
        let bar_width = available - 8.0;

        ui.allocate_ui(
            Vec2::new(bar_width, 32.0),
            |ui| {
                Self::render_url_bar(ui, ui_state, palette, 32.0, on_action);
            },
        );
    }

    /// Render navigation buttons (back, forward, refresh, home)
    fn render_nav_buttons(
        ui: &mut Ui,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        let btn_size = Vec2::new(32.0, 32.0);

        // Back
        let back_btn = egui::Button::new("◀")
            .fill(Color32::TRANSPARENT)
            .min_size(btn_size);
        if ui.add(back_btn).clicked() {
            on_action(ChromeAction::GoBack);
        }

        // Forward
        let fwd_btn = egui::Button::new("▶")
            .fill(Color32::TRANSPARENT)
            .min_size(btn_size);
        if ui.add(fwd_btn).clicked() {
            on_action(ChromeAction::GoForward);
        }

        // Refresh
        let refresh_btn = egui::Button::new("↻")
            .fill(Color32::TRANSPARENT)
            .min_size(btn_size);
        if ui.add(refresh_btn).clicked() {
            on_action(ChromeAction::Refresh);
        }

        // Home
        let home_btn = egui::Button::new("⌂")
            .fill(Color32::TRANSPARENT)
            .min_size(btn_size);
        if ui.add(home_btn).clicked() {
            on_action(ChromeAction::GoHome);
        }
    }

    /// Render the URL/search bar
    fn render_url_bar(
        ui: &mut Ui,
        ui_state: &mut UiState,
        palette: &NovaPalette,
        height: f32,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        let available = ui.available_width();
        let bar_width = (available - 120.0).max(200.0);

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(bar_width, height),
            egui::Sense::click(),
        );

        let painter = ui.painter();

        // Bar background
        let bg_color = if ui_state.address_bar_focused {
            palette.address_bar_focus_bg
        } else {
            palette.address_bar_bg
        };

        painter.rect_filled(
            rect,
            Rounding::same(corner_radius() * 2.0),
            bg_color,
        );

        // Border
        let border_color = if ui_state.address_bar_focused {
            palette.border_focused
        } else if response.hovered() {
            palette.accent_secondary
        } else {
            palette.border
        };

        painter.rect_stroke(
            rect,
            Rounding::same(corner_radius() * 2.0),
            Stroke::new(1.5, border_color),
        );

        // Security icon (lock for HTTPS)
        let lock_icon = "🔒";
        painter.text(
            rect.left_center() + Vec2::new(12.0, 0.0),
            egui::Align2::LEFT_CENTER,
            lock_icon,
            egui::FontId::proportional(12.0),
            palette.success,
        );

        // URL text
        let url_text = if ui_state.address_bar_focused {
            "https://www.example.com"
        } else {
            "Search or enter URL"
        };

        painter.text(
            rect.left_center() + Vec2::new(32.0, 0.0),
            egui::Align2::LEFT_CENTER,
            url_text,
            egui::FontId::proportional(13.0),
            if ui_state.address_bar_focused {
                palette.text_primary
            } else {
                palette.text_tertiary
            },
        );

        // Voice search button
        let voice_rect = Rect::from_center_size(
            rect.right_center() + Vec2::new(-48.0, 0.0),
            Vec2::new(24.0, 24.0),
        );
        painter.text(
            voice_rect.center(),
            egui::Align2::CENTER_CENTER,
            "🎤",
            egui::FontId::proportional(14.0),
            palette.text_secondary,
        );

        // Bookmark button
        let bookmark_rect = Rect::from_center_size(
            rect.right_center() + Vec2::new(-20.0, 0.0),
            Vec2::new(24.0, 24.0),
        );
        painter.text(
            bookmark_rect.center(),
            egui::Align2::CENTER_CENTER,
            "☆",
            egui::FontId::proportional(14.0),
            palette.text_secondary,
        );

        // Handle click to focus
        if response.clicked() {
            ui_state.address_bar_focused = true;
        }

        // Handle click on voice search
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            if voice_rect.contains(pos) && ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary)) {
                on_action(ChromeAction::VoiceSearch);
            }
            if bookmark_rect.contains(pos) && ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary)) {
                on_action(ChromeAction::ToggleBookmark);
            }
        }

        // Search suggestions dropdown
        if ui_state.address_bar_focused && !ui_state.search_suggestions.is_empty() {
            Self::render_suggestions(ui, ui_state, palette, rect, on_action);
        }
    }

    /// Render action buttons (extensions, downloads, menu, etc.)
    fn render_action_buttons(
        ui: &mut Ui,
        ui_state: &mut UiState,
        palette: &NovaPalette,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        let btn_size = Vec2::new(32.0, 32.0);

        // Extensions
        let ext_btn = egui::Button::new("🧩")
            .fill(Color32::TRANSPARENT)
            .min_size(btn_size);
        if ui.add(ext_btn).clicked() {
            on_action(ChromeAction::OpenExtensionManager);
        }

        // Downloads
        let dl_btn = egui::Button::new("⬇")
            .fill(Color32::TRANSPARENT)
            .min_size(btn_size);
        if ui.add(dl_btn).clicked() {
            on_action(ChromeAction::OpenDownloads);
        }

        // Bookmarks/History
        let bm_btn = egui::Button::new("📑")
            .fill(Color32::TRANSPARENT)
            .min_size(btn_size);
        if ui.add(bm_btn).clicked() {
            on_action(ChromeAction::ToggleSidebar);
        }

        // Split view
        let split_btn = egui::Button::new("◫")
            .fill(Color32::TRANSPARENT)
            .min_size(btn_size);
        if ui.add(split_btn).clicked() {
            on_action(ChromeAction::ToggleSplitView);
        }

        // Reading mode
        let reader_btn = egui::Button::new("📖")
            .fill(Color32::TRANSPARENT)
            .min_size(btn_size);
        if ui.add(reader_btn).clicked() {
            on_action(ChromeAction::ToggleReadingMode);
        }

        // Theme toggle
        let theme_btn = egui::Button::new(if ui_state.theme_mode.is_dark() { "☀" } else { "🌙" })
            .fill(Color32::TRANSPARENT)
            .min_size(btn_size);
        if ui.add(theme_btn).clicked() {
            on_action(ChromeAction::ToggleDarkMode);
        }

        // Menu
        let menu_btn = egui::Button::new("⋯")
            .fill(Color32::TRANSPARENT)
            .min_size(btn_size);
        if ui.add(menu_btn).clicked() {
            on_action(ChromeAction::ToggleMenu);
        }
    }

    /// Render search suggestions dropdown
    fn render_suggestions(
        ui: &mut Ui,
        ui_state: &UiState,
        palette: &NovaPalette,
        bar_rect: Rect,
        on_action: &mut dyn FnMut(ChromeAction),
    ) {
        let suggestion_count = ui_state.search_suggestions.len();
        let popup_height = (suggestion_count as f32 * 36.0).min(300.0);

        let popup_rect = Rect::from_min_size(
            bar_rect.left_bottom() + Vec2::new(0.0, 4.0),
            Vec2::new(bar_rect.width(), popup_height),
        );

        ui.painter().rect_filled(
            popup_rect,
            Rounding::same(corner_radius()),
            palette.surface,
        );

        ui.painter().rect_stroke(
            popup_rect,
            Rounding::same(corner_radius()),
            Stroke::new(1.0, palette.border),
        );

        // Render each suggestion
        for (i, suggestion) in ui_state.search_suggestions.iter().enumerate() {
            let item_rect = Rect::from_min_size(
                popup_rect.min + Vec2::new(0.0, i as f32 * 36.0),
                Vec2::new(popup_rect.width(), 36.0),
            );

            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                if item_rect.contains(pos) {
                    ui.painter().rect_filled(item_rect, Rounding::same(0.0), palette.bg_hover);
                }
            }

            ui.painter().text(
                item_rect.left_center() + Vec2::new(12.0, 0.0),
                egui::Align2::LEFT_CENTER,
                suggestion,
                egui::FontId::proportional(13.0),
                palette.text_primary,
            );
        }
    }
}