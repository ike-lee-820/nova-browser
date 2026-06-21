// Custom button widgets for Nova Browser

use egui::{Color32, Response, Rounding, Stroke, Ui, Vec2};

use crate::theme::{corner_radius, small_corner_radius};

/// Rounded icon button
pub fn icon_button(ui: &mut Ui, icon: &str, size: f32) -> Response {
    let btn_size = Vec2::new(size, size);
    ui.add(
        egui::Button::new(icon)
            .fill(Color32::TRANSPARENT)
            .min_size(btn_size),
    )
}

/// Primary action button with accent color
pub fn primary_button(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .color(Color32::WHITE)
                .size(14.0),
        )
        .fill(Color32::from_rgb(67, 97, 238))
        .rounding(Rounding::same(corner_radius()))
        .min_size(Vec2::new(100.0, 36.0)),
    )
}

/// Secondary button with outline style
pub fn secondary_button(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .color(Color32::from_rgb(67, 97, 238))
                .size(14.0),
        )
        .fill(Color32::TRANSPARENT)
        .rounding(Rounding::same(corner_radius()))
        .min_size(Vec2::new(100.0, 36.0)),
    )
}

/// Danger button (red)
pub fn danger_button(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .color(Color32::WHITE)
                .size(14.0),
        )
        .fill(Color32::from_rgb(220, 53, 69))
        .rounding(Rounding::same(corner_radius()))
        .min_size(Vec2::new(100.0, 36.0)),
    )
}

/// Toggle switch
pub fn toggle_switch(ui: &mut Ui, enabled: &mut bool) -> Response {
    let size = Vec2::new(40.0, 22.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if response.clicked() {
        *enabled = !*enabled;
    }

    let painter = ui.painter();
    let track_color = if *enabled {
        Color32::from_rgb(67, 97, 238)
    } else {
        Color32::from_rgb(200, 200, 200)
    };

    // Track
    painter.rect_filled(rect, Rounding::same(11.0), track_color);

    // Thumb
    let thumb_x = if *enabled {
        rect.right() - 10.0
    } else {
        rect.left() + 10.0
    };
    let thumb_center = egui::Pos2::new(thumb_x, rect.center().y);
    painter.circle_filled(thumb_center, 8.0, Color32::WHITE);

    response
}

/// Toolbar button with tooltip
pub fn toolbar_button(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
    let response = ui.add(
        egui::Button::new(icon)
            .fill(Color32::TRANSPARENT)
            .min_size(Vec2::new(32.0, 32.0)),
    );

    if response.hovered() {
        egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new(tooltip), |ui| {
            ui.label(tooltip);
        });
    }

    response
}

/// Badge (notification count)
pub fn badge(ui: &mut Ui, count: u32) {
    let pos = ui.next_widget_position();
    let painter = ui.painter();
    let center = pos + Vec2::new(8.0, -4.0);
    let radius = 9.0;

    painter.circle_filled(center, radius, Color32::from_rgb(220, 53, 69));
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        &count.to_string(),
        egui::FontId::proportional(10.0),
        Color32::WHITE,
    );
}

/// Search input with icon
pub fn search_input(ui: &mut Ui, text: &mut String, placeholder: &str) -> Response {
    let mut response = ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(placeholder)
            .desired_width(ui.available_width()),
    );

    response
}