// CEF (Chromium Embedded Framework) bridge module
// Handles CEF initialization, browser creation, and lifecycle management

use cef::{
    app::App,
    client::Client,
    types::Rect,
    window_info::WindowInfo,
    CefString,
};
use std::sync::Arc;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::NovaError;

/// Global CEF bridge instance
pub static CEF_BRIDGE: Lazy<Mutex<Option<CefBridge>>> = Lazy::new(|| Mutex::new(None));

/// CEF bridge manages the Chromium Embedded Framework lifecycle
pub struct CefBridge {
    pub is_initialized: bool,
    pub browser_count: Arc<Mutex<usize>>,
    cef_settings: CefSettings,
}

#[derive(Debug, Clone)]
pub struct CefSettings {
    pub cache_path: String,
    pub user_agent: String,
    pub remote_debugging_port: Option<u16>,
    pub background_color: u32,
    pub multi_threaded_message_loop: bool,
    pub external_message_pump: bool,
    pub windowless_rendering_enabled: bool,
    pub command_line_args_disabled: bool,
}

impl Default for CefSettings {
    fn default() -> Self {
        Self {
            cache_path: String::new(),
            user_agent: String::from("NovaBrowser/1.0"),
            remote_debugging_port: None,
            background_color: 0xFFFFFFFF,
            multi_threaded_message_loop: true,
            external_message_pump: false,
            windowless_rendering_enabled: false,
            command_line_args_disabled: false,
        }
    }
}

impl CefBridge {
    pub fn new(settings: CefSettings) -> Self {
        Self {
            is_initialized: false,
            browser_count: Arc::new(Mutex::new(0)),
            cef_settings: settings,
        }
    }

    /// Initialize CEF with the given settings
    pub fn initialize(&mut self) -> Result<(), NovaError> {
        if self.is_initialized {
            return Ok(());
        }

        // Build CEF settings
        let mut cef_settings = cef::types::Settings::default();
        cef_settings.multi_threaded_message_loop = if self.cef_settings.multi_threaded_message_loop {
            1
        } else {
            0
        };
        cef_settings.external_message_pump = if self.cef_settings.external_message_pump {
            1
        } else {
            0
        };
        cef_settings.windowless_rendering_enabled = if self.cef_settings.windowless_rendering_enabled {
            1
        } else {
            0
        };
        cef_settings.command_line_args_disabled = if self.cef_settings.command_line_args_disabled {
            1
        } else {
            0
        };
        cef_settings.background_color = self.cef_settings.background_color;

        if !self.cef_settings.cache_path.is_empty() {
            let cache_path: CefString = self.cef_settings.cache_path.as_str().into();
            cef_settings.cache_path = cache_path;
        }

        // Set user agent
        let user_agent: CefString = self.cef_settings.user_agent.as_str().into();
        cef_settings.user_agent = user_agent;

        if let Some(port) = self.cef_settings.remote_debugging_port {
            cef_settings.remote_debugging_port = port as i32;
        }

        // Initialize CEF
        let app = App::new();
        let args = cef::args::Args::default();

        let result = cef::initialize(cef_settings, app, args);
        match result {
            Ok(_) => {
                self.is_initialized = true;
                log::info!("CEF initialized successfully");
                Ok(())
            }
            Err(e) => {
                log::error!("CEF initialization failed: {:?}", e);
                Err(NovaError::CefError(format!("{:?}", e)))
            }
        }
    }

    /// Create a new browser window
    pub fn create_browser(
        &self,
        url: &str,
        parent_window: Option<raw_window_handle::RawWindowHandle>,
        bounds: (i32, i32, i32, i32), // x, y, width, height
    ) -> Result<BrowserHandle, NovaError> {
        if !self.is_initialized {
            return Err(NovaError::CefError("CEF not initialized".to_string()));
        }

        let mut window_info = WindowInfo::default();
        window_info.bounds = Rect {
            x: bounds.0,
            y: bounds.1,
            width: bounds.2,
            height: bounds.3,
        };

        // Set parent window if provided
        if let Some(parent) = parent_window {
            set_parent_window(&mut window_info, parent);
        }

        let url_cstr: CefString = url.into();
        let client = Client::new();

        let mut count = self.browser_count.lock().unwrap();
        *count += 1;
        let browser_id = *count;

        log::info!("Creating browser #{} with URL: {}", browser_id, url);

        Ok(BrowserHandle {
            id: browser_id,
            url: url.to_string(),
            is_loading: true,
        })
    }

    /// Shutdown CEF
    pub fn shutdown(&mut self) {
        if self.is_initialized {
            // Wait for all browsers to close
            let mut attempts = 0;
            loop {
                let count = *self.browser_count.lock().unwrap();
                if count == 0 || attempts > 100 {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
                attempts += 1;
            }

            cef::shutdown();
            self.is_initialized = false;
            log::info!("CEF shutdown complete");
        }
    }

    /// Run the CEF message loop
    pub fn run_message_loop(&self) {
        if self.is_initialized {
            cef::run_message_loop();
        }
    }

    /// Quit the CEF message loop
    pub fn quit_message_loop(&self) {
        if self.is_initialized {
            cef::quit_message_loop();
        }
    }

    /// Do a single iteration of message loop work
    pub fn do_message_loop_work(&self) {
        if self.is_initialized {
            cef::do_message_loop_work();
        }
    }
}

/// Handle to a managed browser instance
#[derive(Debug, Clone)]
pub struct BrowserHandle {
    pub id: usize,
    pub url: String,
    pub is_loading: bool,
}

/// Set the parent window for CEF browser embedding
fn set_parent_window(window_info: &mut WindowInfo, handle: raw_window_handle::RawWindowHandle) {
    use raw_window_handle::RawWindowHandle;

    match handle {
        #[cfg(target_os = "windows")]
        RawWindowHandle::Win32(win32_handle) => {
            window_info.parent_window = win32_handle.hwnd.get() as u64;
            window_info.SetAsChild(
                win32_handle.hwnd.get() as u64,
                &Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
            );
        }
        #[cfg(target_os = "macos")]
        RawWindowHandle::AppKit(appkit_handle) => {
            window_info.parent_view = appkit_handle.ns_view.as_ptr() as *mut std::ffi::c_void;
        }
        #[cfg(target_os = "linux")]
        RawWindowHandle::Xlib(xlib_handle) => {
            window_info.parent_window = xlib_handle.window as u64;
        }
        #[cfg(target_os = "linux")]
        RawWindowHandle::Wayland(_) => {
            log::warn!("Wayland parent window embedding is not fully supported");
        }
        _ => {
            log::warn!("Unsupported window handle type for CEF embedding");
        }
    }
}

/// Initialize CEF from the main process
pub fn init_cef(cache_path: &str, user_agent: &str) -> Result<(), NovaError> {
    let settings = CefSettings {
        cache_path: cache_path.to_string(),
        user_agent: user_agent.to_string(),
        remote_debugging_port: Some(9222),
        multi_threaded_message_loop: true,
        ..Default::default()
    };

    let mut bridge = CefBridge::new(settings);
    bridge.initialize()
}

/// Shutdown CEF
pub fn shutdown_cef() {
    if let Ok(mut guard) = CEF_BRIDGE.lock() {
        if let Some(ref mut bridge) = *guard {
            bridge.shutdown();
        }
    }
}