use egui::{Context, ScrollArea, Ui, Window};
use nova_features::settings::Settings;

/// Renders the comprehensive settings dialog
pub fn render_settings_dialog(ctx: &Context, settings: &mut Settings, show: &mut bool) {
    if !*show {
        return;
    }

    Window::new("Settings")
        .resizable(true)
        .default_size([800.0, 600.0])
        .collapsible(false)
        .show(ctx, |ui| {
            render_settings_content(ui, settings, show);
        });
}

fn render_settings_content(ui: &mut Ui, s: &mut Settings, show: &mut bool) {
    // Tabs for categories
    ui.horizontal(|ui| {
        ui.selectable_label(true, "Privacy & Security");
        ui.selectable_label(false, "Appearance");
        ui.selectable_label(false, "Functionality");
        ui.selectable_label(false, "Advanced");
        ui.selectable_label(false, "Mobile");
    });
    ui.separator();

    ScrollArea::vertical().show(ui, |ui| {
        // ===== PRIVACY & SECURITY =====
        ui.collapsing("Cookie Management", |ui| {
            ui.checkbox(
                &mut s.allow_third_party_cookies,
                "Allow third-party cookies",
            );
            ui.checkbox(
                &mut s.block_third_party_cookies_by_default,
                "Block third-party cookies by default",
            );
        });
        ui.collapsing("Tracking Protection", |ui| {
            ui.checkbox(&mut s.block_ad_trackers, "Block ad trackers");
            ui.checkbox(&mut s.block_fingerprinting, "Block fingerprinting");
            ui.checkbox(
                &mut s.block_social_media_trackers,
                "Block social media trackers",
            );
        });
        ui.collapsing("HTTPS", |ui| {
            ui.checkbox(&mut s.force_https, "Force HTTPS");
            ui.checkbox(
                &mut s.warn_insecure_connections,
                "Warn on insecure connections",
            );
        });
        ui.collapsing("Safe Browsing", |ui| {
            ui.checkbox(&mut s.safe_browsing_enabled, "Enable Safe Browsing");
            ui.checkbox(&mut s.scan_downloads, "Scan downloads");
            ui.checkbox(&mut s.phishing_protection, "Phishing protection");
        });
        ui.collapsing("Password Manager", |ui| {
            ui.checkbox(
                &mut s.password_generator_enabled,
                "Password generator",
            );
            ui.checkbox(&mut s.auto_fill_passwords, "Auto-fill passwords");
            ui.checkbox(&mut s.leak_detection, "Leak detection");
            ui.checkbox(&mut s.use_master_password, "Use master password");
            ui.checkbox(&mut s.biometric_protection, "Biometric protection");
        });
        ui.collapsing("Do Not Track", |ui| {
            ui.checkbox(&mut s.send_dnt_header, "Send Do Not Track header");
        });
        ui.collapsing("Site Isolation", |ui| {
            ui.checkbox(
                &mut s.site_isolation_enabled,
                "Enable site isolation",
            );
            ui.checkbox(&mut s.strict_isolation, "Strict isolation");
        });

        ui.separator();

        // ===== APPEARANCE =====
        ui.collapsing("Theme", |ui| {
            ui.checkbox(&mut s.show_shortcuts, "Show shortcuts on new tab");
            ui.checkbox(
                &mut s.show_search_box,
                "Show search box on new tab",
            );
        });
        ui.collapsing("Bookmarks", |ui| {
            ui.checkbox(
                &mut s.bookmarks_bar_visible,
                "Show bookmarks bar",
            );
            ui.checkbox(&mut s.auto_sort_bookmarks, "Auto-sort bookmarks");
        });
        ui.collapsing("Downloads", |ui| {
            ui.checkbox(&mut s.ask_before_download, "Ask before download");
            ui.checkbox(
                &mut s.show_download_notifications,
                "Show download notifications",
            );
            ui.checkbox(&mut s.warn_dangerous_files, "Warn dangerous files");
        });
        ui.collapsing("Fonts", |ui| {
            let mut font_size = s.minimum_font_size as i32;
            if ui
                .add(
                    egui::Slider::new(&mut font_size, 8..=72)
                        .text("Minimum font size"),
                )
                .changed()
            {
                s.minimum_font_size = font_size.max(0) as u32;
            }
            ui.add(
                egui::Slider::new(&mut s.default_page_zoom, 0.25..=5.0)
                    .text("Default zoom"),
            );
        });
        ui.collapsing("Language", |ui| {
            ui.checkbox(&mut s.translate_pages, "Translate pages");
            ui.checkbox(&mut s.spell_check_enabled, "Spell check");
        });
        ui.collapsing("Reading Mode", |ui| {
            ui.add(
                egui::Slider::new(&mut s.reading_mode_font_size, 10.0..=36.0)
                    .text("Font size"),
            );
            ui.add(
                egui::Slider::new(&mut s.reading_mode_line_width, 400.0..=1200.0)
                    .text("Line width"),
            );
        });
        ui.collapsing("Sidebar", |ui| {
            ui.checkbox(&mut s.sidebar_visible, "Show sidebar");
        });

        ui.separator();

        // ===== FUNCTIONALITY =====
        ui.collapsing("Search", |ui| {
            ui.checkbox(
                &mut s.address_bar_suggestions,
                "Address bar suggestions",
            );
            ui.checkbox(
                &mut s.bookmark_suggestions,
                "Bookmark suggestions",
            );
            ui.checkbox(&mut s.history_suggestions, "History suggestions");
            ui.checkbox(&mut s.auto_complete_enabled, "Auto-complete");
            ui.checkbox(&mut s.paste_and_search, "Paste and search");
        });
        ui.collapsing("Tabs", |ui| {
            ui.checkbox(&mut s.tab_groups_enabled, "Tab groups");
            ui.checkbox(&mut s.vertical_tabs, "Vertical tabs");
            ui.checkbox(
                &mut s.hibernate_inactive_tabs,
                "Hibernate inactive tabs",
            );
            ui.checkbox(
                &mut s.tab_preview_on_hover,
                "Tab preview on hover",
            );
            ui.checkbox(&mut s.multi_select_tabs, "Multi-select tabs");
        });
        ui.collapsing("Auto-fill", |ui| {
            ui.checkbox(&mut s.auto_fill_addresses, "Auto-fill addresses");
            ui.checkbox(
                &mut s.auto_fill_credit_cards,
                "Auto-fill credit cards",
            );
            ui.checkbox(&mut s.auto_fill_passwords, "Auto-fill passwords");
            ui.checkbox(
                &mut s.auto_fill_form_history,
                "Auto-fill form history",
            );
        });
        ui.collapsing("Sync", |ui| {
            ui.checkbox(&mut s.sync_enabled, "Enable sync");
            if s.sync_enabled {
                ui.checkbox(&mut s.sync_bookmarks, "Sync bookmarks");
                ui.checkbox(&mut s.sync_history, "Sync history");
                ui.checkbox(&mut s.sync_passwords, "Sync passwords");
                ui.checkbox(&mut s.sync_extensions, "Sync extensions");
                ui.checkbox(&mut s.sync_settings, "Sync settings");
                ui.checkbox(&mut s.sync_open_tabs, "Sync open tabs");
            }
        });
        ui.collapsing("Extensions", |ui| {
            ui.checkbox(&mut s.extensions_enabled, "Extensions enabled");
            ui.checkbox(&mut s.developer_mode, "Developer mode");
            ui.checkbox(
                &mut s.allow_unpacked_extensions,
                "Allow unpacked extensions",
            );
            ui.checkbox(
                &mut s.auto_update_extensions,
                "Auto-update extensions",
            );
        });
        ui.collapsing("PWA", |ui| {
            ui.checkbox(&mut s.pwa_support_enabled, "PWA support");
            ui.checkbox(&mut s.install_prompt_enabled, "Install prompt");
        });
        ui.collapsing("Workspace", |ui| {
            ui.checkbox(&mut s.workspace_enabled, "Workspace enabled");
            ui.checkbox(&mut s.split_view_enabled, "Split view enabled");
        });

        ui.separator();

        // ===== ADVANCED =====
        ui.collapsing("Network", |ui| {
            ui.checkbox(&mut s.ipv6_enabled, "IPv6");
            ui.checkbox(&mut s.quic_enabled, "QUIC");
            ui.checkbox(&mut s.http3_enabled, "HTTP/3");
        });
        ui.collapsing("DNS", |ui| {
            ui.checkbox(
                &mut s.dns_over_https_enabled,
                "DNS over HTTPS",
            );
            ui.checkbox(
                &mut s.dns_over_tls_enabled,
                "DNS over TLS",
            );
        });
        ui.collapsing("Hardware", |ui| {
            ui.checkbox(
                &mut s.gpu_acceleration_enabled,
                "GPU acceleration",
            );
            ui.checkbox(&mut s.webgl_enabled, "WebGL");
            ui.checkbox(&mut s.canvas_acceleration, "Canvas acceleration");
            ui.checkbox(
                &mut s.video_decode_acceleration,
                "Video decode acceleration",
            );
        });
        ui.collapsing("Content", |ui| {
            ui.checkbox(&mut s.javascript_enabled, "JavaScript");
            ui.checkbox(&mut s.images_enabled, "Images");
            ui.checkbox(&mut s.popup_blocker_enabled, "Popup blocker");
            ui.checkbox(&mut s.pdf_viewer_enabled, "PDF viewer");
        });
        ui.collapsing("Developer Tools", |ui| {
            ui.checkbox(&mut s.dev_tools_enabled, "Developer tools");
            ui.checkbox(
                &mut s.remote_debugging_enabled,
                "Remote debugging",
            );
            if s.remote_debugging_enabled {
                let mut port = s.remote_debugging_port as i32;
                if ui
                    .add(
                        egui::DragValue::new(&mut port)
                            .clamp_range(1024..=65535)
                            .prefix("Port: "),
                    )
                    .changed()
                {
                    s.remote_debugging_port = port.max(0) as u16;
                }
            }
        });
        ui.collapsing("Performance", |ui| {
            let mut mem = s.memory_limit_mb as i32;
            if ui
                .add(
                    egui::Slider::new(&mut mem, 256..=32768)
                        .text("Memory limit (MB)"),
                )
                .changed()
            {
                s.memory_limit_mb = mem.max(0) as u32;
            }
            let mut procs = s.max_process_count as i32;
            if ui
                .add(
                    egui::Slider::new(&mut procs, 1..=64)
                        .text("Max processes"),
                )
                .changed()
            {
                s.max_process_count = procs.max(0) as u32;
            }
            ui.checkbox(&mut s.energy_saving_mode, "Energy saving mode");
        });
        ui.collapsing("Accessibility", |ui| {
            ui.checkbox(&mut s.screen_reader_support, "Screen reader");
            ui.checkbox(&mut s.high_contrast_mode, "High contrast");
            ui.checkbox(&mut s.reduced_animations, "Reduced animations");
            ui.checkbox(&mut s.keyboard_navigation, "Keyboard navigation");
            ui.checkbox(&mut s.caret_browsing, "Caret browsing");
        });
        ui.collapsing("User Agent", |ui| {
            let mut ua = s.custom_user_agent.clone().unwrap_or_default();
            if ui.text_edit_singleline(&mut ua).changed() {
                s.custom_user_agent = if ua.is_empty() {
                    None
                } else {
                    Some(ua)
                };
            }
        });

        ui.separator();

        // ===== MOBILE =====
        ui.collapsing("Gestures", |ui| {
            ui.checkbox(&mut s.swipe_navigation, "Swipe navigation");
            ui.checkbox(&mut s.pull_to_refresh, "Pull to refresh");
            ui.checkbox(&mut s.long_press_menu, "Long press menu");
            ui.checkbox(&mut s.pinch_to_zoom, "Pinch to zoom");
        });
        ui.collapsing("Data Saving", |ui| {
            ui.checkbox(&mut s.data_saver_enabled, "Data saver");
            ui.checkbox(&mut s.compress_images, "Compress images");
            ui.checkbox(&mut s.lazy_load_images, "Lazy load images");
            ui.checkbox(
                &mut s.restrict_background_data,
                "Restrict background data",
            );
        });
        ui.collapsing("Desktop Site", |ui| {
            ui.checkbox(&mut s.force_desktop_site, "Force desktop site");
        });
        ui.collapsing("Sharing", |ui| {
            ui.checkbox(&mut s.native_share_menu, "Native share menu");
            ui.checkbox(&mut s.qr_code_generator, "QR code generator");
        });

        ui.add_space(16.0);
        ui.separator();
        if ui.button("Close").clicked() {
            *show = false;
        }
    });
}

/// Renders the main menu dialog
pub fn render_menu_dialog(ctx: &Context, show_menu: &mut bool) {
    egui::Window::new("Menu")
        .anchor(egui::Align2::RIGHT_TOP, [-8.0, 48.0])
        .collapsible(false)
        .title_bar(false)
        .resizable(false)
        .show(ctx, |ui| {
            if ui.button("New Tab").clicked() {
                *show_menu = false;
            }
            if ui.button("New Incognito Tab").clicked() {
                *show_menu = false;
            }
            ui.separator();
            if ui.button("Reading Mode").clicked() {
                *show_menu = false;
            }
            if ui.button("Split View").clicked() {
                *show_menu = false;
            }
            ui.separator();
            if ui.button("Developer Tools").clicked() {
                *show_menu = false;
            }
            if ui.button("Bookmarks").clicked() {
                *show_menu = false;
            }
            if ui.button("History").clicked() {
                *show_menu = false;
            }
            if ui.button("Downloads").clicked() {
                *show_menu = false;
            }
            if ui.button("Passwords").clicked() {
                *show_menu = false;
            }
            if ui.button("Extensions").clicked() {
                *show_menu = false;
            }
            ui.separator();
            if ui.button("Settings").clicked() {
                *show_menu = false;
            }
        });
}