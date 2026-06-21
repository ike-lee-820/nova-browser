// Theme system for Nova Browser
// Supports light and dark themes with consistent design tokens

use egui::{Color32, Context, FontId, Rounding, Stroke, Style, Visuals};
use nova_core::ThemeMode;

/// Color palette for the Nova theme
pub struct NovaPalette {
    // Background colors
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_tertiary: Color32,
    pub bg_hover: Color32,
    pub bg_active: Color32,

    // Surface colors
    pub surface: Color32,
    pub surface_raised: Color32,
    pub surface_overlay: Color32,

    // Text colors
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_tertiary: Color32,
    pub text_disabled: Color32,
    pub text_on_accent: Color32,

    // Accent colors
    pub accent_primary: Color32,
    pub accent_secondary: Color32,
    pub accent_tertiary: Color32,

    // Status colors
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub info: Color32,

    // Border colors
    pub border: Color32,
    pub border_focused: Color32,
    pub divider: Color32,

    // Tab colors
    pub tab_active_bg: Color32,
    pub tab_inactive_bg: Color32,
    pub tab_hover_bg: Color32,

    // Toolbar colors
    pub toolbar_bg: Color32,
    pub address_bar_bg: Color32,
    pub address_bar_focus_bg: Color32,

    // Sidebar colors
    pub sidebar_bg: Color32,
    pub sidebar_item_hover: Color32,
    pub sidebar_item_active: Color32,

    // Shadow
    pub shadow_color: Color32,

    // Incognito colors
    pub incognito_bg: Color32,
    pub incognito_toolbar: Color32,
}

/// Light theme palette
pub const LIGHT_PALETTE: NovaPalette = NovaPalette {
    bg_primary: Color32::from_rgb(255, 255, 255),
    bg_secondary: Color32::from_rgb(248, 249, 250),
    bg_tertiary: Color32::from_rgb(241, 243, 245),
    bg_hover: Color32::from_rgb(233, 236, 239),
    bg_active: Color32::from_rgb(222, 226, 230),

    surface: Color32::from_rgb(255, 255, 255),
    surface_raised: Color32::from_rgb(255, 255, 255),
    surface_overlay: Color32::from_rgba_premultiplied(255, 255, 255, 240),

    text_primary: Color32::from_rgb(33, 37, 41),
    text_secondary: Color32::from_rgb(108, 117, 125),
    text_tertiary: Color32::from_rgb(173, 181, 189),
    text_disabled: Color32::from_rgb(206, 212, 218),
    text_on_accent: Color32::from_rgb(255, 255, 255),

    accent_primary: Color32::from_rgb(67, 97, 238),
    accent_secondary: Color32::from_rgb(114, 139, 250),
    accent_tertiary: Color32::from_rgb(222, 230, 255),

    success: Color32::from_rgb(40, 167, 69),
    warning: Color32::from_rgb(255, 193, 7),
    error: Color32::from_rgb(220, 53, 69),
    info: Color32::from_rgb(23, 162, 184),

    border: Color32::from_rgb(222, 226, 230),
    border_focused: Color32::from_rgb(67, 97, 238),
    divider: Color32::from_rgb(233, 236, 239),

    tab_active_bg: Color32::from_rgb(255, 255, 255),
    tab_inactive_bg: Color32::from_rgb(241, 243, 245),
    tab_hover_bg: Color32::from_rgb(233, 236, 239),

    toolbar_bg: Color32::from_rgb(248, 249, 250),
    address_bar_bg: Color32::from_rgb(255, 255, 255),
    address_bar_focus_bg: Color32::from_rgb(255, 255, 255),

    sidebar_bg: Color32::from_rgb(248, 249, 250),
    sidebar_item_hover: Color32::from_rgb(233, 236, 239),
    sidebar_item_active: Color32::from_rgb(222, 226, 230),

    shadow_color: Color32::from_rgba_premultiplied(0, 0, 0, 30),

    incognito_bg: Color32::from_rgb(40, 42, 54),
    incognito_toolbar: Color32::from_rgb(50, 52, 64),
};

/// Dark theme palette
pub const DARK_PALETTE: NovaPalette = NovaPalette {
    bg_primary: Color32::from_rgb(30, 30, 30),
    bg_secondary: Color32::from_rgb(37, 37, 38),
    bg_tertiary: Color32::from_rgb(45, 45, 45),
    bg_hover: Color32::from_rgb(55, 55, 55),
    bg_active: Color32::from_rgb(65, 65, 65),

    surface: Color32::from_rgb(43, 43, 43),
    surface_raised: Color32::from_rgb(50, 50, 50),
    surface_overlay: Color32::from_rgba_premultiplied(30, 30, 30, 240),

    text_primary: Color32::from_rgb(236, 236, 236),
    text_secondary: Color32::from_rgb(170, 170, 170),
    text_tertiary: Color32::from_rgb(120, 120, 120),
    text_disabled: Color32::from_rgb(80, 80, 80),
    text_on_accent: Color32::from_rgb(255, 255, 255),

    accent_primary: Color32::from_rgb(86, 156, 255),
    accent_secondary: Color32::from_rgb(77, 140, 230),
    accent_tertiary: Color32::from_rgb(40, 60, 100),

    success: Color32::from_rgb(72, 199, 103),
    warning: Color32::from_rgb(255, 210, 61),
    error: Color32::from_rgb(255, 90, 90),
    info: Color32::from_rgb(77, 184, 255),

    border: Color32::from_rgb(60, 60, 60),
    border_focused: Color32::from_rgb(86, 156, 255),
    divider: Color32::from_rgb(50, 50, 50),

    tab_active_bg: Color32::from_rgb(30, 30, 30),
    tab_inactive_bg: Color32::from_rgb(37, 37, 38),
    tab_hover_bg: Color32::from_rgb(45, 45, 45),

    toolbar_bg: Color32::from_rgb(37, 37, 38),
    address_bar_bg: Color32::from_rgb(45, 45, 45),
    address_bar_focus_bg: Color32::from_rgb(50, 50, 50),

    sidebar_bg: Color32::from_rgb(33, 33, 33),
    sidebar_item_hover: Color32::from_rgb(50, 50, 50),
    sidebar_item_active: Color32::from_rgb(60, 60, 60),

    shadow_color: Color32::from_rgba_premultiplied(0, 0, 0, 80),

    incognito_bg: Color32::from_rgb(40, 42, 54),
    incognito_toolbar: Color32::from_rgb(50, 52, 64),
};

/// Get the current palette based on theme mode
pub fn get_palette(is_dark: bool) -> &'static NovaPalette {
    if is_dark {
        &DARK_PALETTE
    } else {
        &LIGHT_PALETTE
    }
}

/// Apply the theme to an egui context
pub fn apply_theme(ctx: &Context, is_dark: bool) {
    let palette = get_palette(is_dark);

    let mut style = (*ctx.style()).clone();

    // Configure visuals
    style.visuals = Visuals {
        dark_mode: is_dark,
        override_text_color: Some(palette.text_primary),
        window_rounding: Rounding::same(12.0),
        window_shadow: egui::epaint::Shadow {
            offset: [0.0, 4.0].into(),
            blur: 16.0,
            spread: 0.0,
            color: palette.shadow_color,
        },
        window_fill: palette.bg_primary,
        panel_fill: palette.bg_secondary,
        faint_bg_color: palette.bg_tertiary,
        extreme_bg_color: palette.bg_active,
        code_bg_color: palette.bg_tertiary,
        warn_fg_color: palette.warning,
        error_fg_color: palette.error,
        hyperlink_color: palette.accent_primary,
        selection: egui::style::Selection {
            bg_fill: palette.accent_tertiary,
            stroke: Stroke::new(1.0, palette.accent_primary),
        },
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: palette.bg_secondary,
                weak_bg_fill: palette.bg_secondary,
                bg_stroke: Stroke::new(1.0, palette.border),
                rounding: Rounding::same(8.0),
                fg_stroke: Stroke::new(1.0, palette.text_primary),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: palette.bg_secondary,
                weak_bg_fill: palette.bg_tertiary,
                bg_stroke: Stroke::new(1.0, palette.border),
                rounding: Rounding::same(8.0),
                fg_stroke: Stroke::new(1.0, palette.text_primary),
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: palette.bg_hover,
                weak_bg_fill: palette.bg_hover,
                bg_stroke: Stroke::new(1.0, palette.accent_secondary),
                rounding: Rounding::same(8.0),
                fg_stroke: Stroke::new(1.5, palette.text_primary),
                expansion: 1.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: palette.bg_active,
                weak_bg_fill: palette.bg_active,
                bg_stroke: Stroke::new(1.0, palette.accent_primary),
                rounding: Rounding::same(8.0),
                fg_stroke: Stroke::new(2.0, palette.accent_primary),
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: palette.bg_active,
                weak_bg_fill: palette.bg_active,
                bg_stroke: Stroke::new(1.0, palette.accent_primary),
                rounding: Rounding::same(8.0),
                fg_stroke: Stroke::new(2.0, palette.accent_primary),
                expansion: 1.0,
            },
        },
        ..Default::default()
    };

    // Configure spacing
    style.spacing = egui::style::Spacing {
        item_spacing: egui::Vec2::new(8.0, 6.0),
        button_padding: egui::Vec2::new(12.0, 6.0),
        indent: 16.0,
        interact_size: egui::Vec2::new(32.0, 32.0),
        slider_width: 200.0,
        combo_width: 200.0,
        text_edit_width: 400.0,
        icon_width: 16.0,
        icon_spacing: 4.0,
        tooltip_width: 300.0,
        indent_ends_with_horizontal_line: false,
        combo_height: 200.0,
        scroll: egui::style::ScrollStyle {
            bar_width: 8.0,
            handle_min_length: 40.0,
            bar_inner_margin: 2.0,
            bar_outer_margin: 2.0,
            ..Default::default()
        },
        ..Default::default()
    };

    ctx.set_style(style);
}

/// Configure fonts for the Nova browser
pub fn configure_fonts(ctx: &Context) {
    // Use system default fonts - egui handles this by default
    // The fonts can be customized by users through settings
    let mut fonts = egui::FontDefinitions::default();

    // Adjust font sizes for better readability
    for (_, font_data) in fonts.font_data.iter_mut() {
        // Font data is already configured with system defaults
    }

    ctx.set_fonts(fonts);
}

/// Get the appropriate corner radius for UI elements
pub fn corner_radius() -> f32 {
    8.0
}

/// Get the appropriate small corner radius
pub fn small_corner_radius() -> f32 {
    4.0
}

/// Get the appropriate large corner radius
pub fn large_corner_radius() -> f32 {
    12.0
}