//! Nova Browser - Main Application Entry Point
//!
//! This is the main application that ties together the UI, features, and core modules.

use std::path::PathBuf;

use anyhow::Result;
use log::info;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoopBuilder},
};

use nova_core::config::AppConfig;
use nova_core::window::{BrowserWindow, desktop_window_params};
use nova_features::FeatureManager;
use nova_ui::UiState;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting Nova Browser...");

    // Load application configuration (returns Option, use default if not found)
    let config = AppConfig::load().unwrap_or_default();
    info!("Configuration loaded");

    // Determine data directory for persistent features
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nova-browser");

    // Create winit event loop
    let event_loop = EventLoopBuilder::new().build()?;

    // Create browser window
    let params = desktop_window_params();
    let mut browser_window = BrowserWindow::new(&event_loop, params)?;
    info!("Window created: {}x{}", browser_window.width, browser_window.height);

    // Initialize feature manager (constructor handles all init internally)
    let _features = FeatureManager::new(data_dir);

    // Initialize UI state
    let theme_mode = config.theme;
    let locale = config.language.clone();
    let _ui_state = UiState::new(theme_mode, &locale);

    info!("Entering main event loop...");

    // Run the event loop
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Close requested, exiting...");
                elwt.exit();
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                browser_window.width = physical_size.width;
                browser_window.height = physical_size.height;
                browser_window.update_scale_factor();
            }

            Event::AboutToWait => {
                // Request a redraw to keep the UI updated
                browser_window.window.request_redraw();
            }

            _ => {}
        }
    })?;

    Ok(())
}