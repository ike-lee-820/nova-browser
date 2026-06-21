// Window management for Nova Browser

use std::sync::Arc;
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowId},
};

use crate::{NovaError, WindowType};

/// Represents a browser window
pub struct BrowserWindow {
    pub id: WindowId,
    pub window: Arc<Window>,
    pub window_type: WindowType,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub is_maximized: bool,
    pub is_fullscreen: bool,
    pub scale_factor: f64,
}

/// Window creation parameters
#[derive(Debug, Clone)]
pub struct WindowParams {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub window_type: WindowType,
    pub position: Option<(i32, i32)>,
    pub transparent: bool,
    pub decorations: bool,
    pub resizable: bool,
    pub always_on_top: bool,
}

impl Default for WindowParams {
    fn default() -> Self {
        Self {
            title: String::from("Nova Browser"),
            width: 1280,
            height: 800,
            min_width: 400,
            min_height: 300,
            window_type: WindowType::Normal,
            position: None,
            transparent: false,
            decorations: true,
            resizable: true,
            always_on_top: false,
        }
    }
}

impl BrowserWindow {
    /// Create a new browser window
    pub fn new<T: 'static>(
        event_loop: &EventLoopWindowTarget<T>,
        params: WindowParams,
    ) -> Result<Self, NovaError> {
        let window = Window::new(event_loop).map_err(|e| {
            NovaError::WindowError(format!("Failed to create window: {}", e))
        })?;

        window.set_title(&params.title);
        window.set_min_inner_size(Some(LogicalSize::new(
            params.min_width as f64,
            params.min_height as f64,
        )));
        window.set_resizable(params.resizable);

        if let Some((x, y)) = params.position {
            window.set_outer_position(PhysicalPosition::new(x, y));
        }

        if params.window_type == WindowType::Incognito {
            window.set_title(&format!("{} - Incognito", params.title));
        }

        let window = Arc::new(window);
        let id = window.id();
        let size = window.inner_size();

        Ok(Self {
            id,
            window,
            window_type: params.window_type,
            title: params.title,
            width: size.width,
            height: size.height,
            is_maximized: false,
            is_fullscreen: false,
            scale_factor: 1.0,
        })
    }

    /// Get the raw window handle for CEF embedding
    pub fn raw_window_handle(&self) -> Option<winit::raw_window_handle::RawWindowHandle> {
        use winit::raw_window_handle::HasRawWindowHandle;
        self.window.raw_window_handle().ok()
    }

    /// Get the content area rect (excluding chrome/titlebar)
    pub fn content_rect(&self) -> (i32, i32, i32, i32) {
        let chrome_height = 100; // toolbar + tab bar height
        (
            0,
            chrome_height,
            self.width as i32,
            (self.height as i32 - chrome_height).max(0),
        )
    }

    /// Resize the window
    pub fn resize(&mut self, width: u32, height: u32) {
        let _ = self
            .window
            .request_inner_size(LogicalSize::new(width as f64, height as f64));
        self.width = width;
        self.height = height;
    }

    /// Toggle maximize
    pub fn toggle_maximize(&mut self) {
        self.is_maximized = !self.is_maximized;
        self.window.set_maximized(self.is_maximized);
    }

    /// Toggle fullscreen
    pub fn toggle_fullscreen(&mut self) {
        if self.is_fullscreen {
            self.window.set_fullscreen(None);
        } else {
            self.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }
        self.is_fullscreen = !self.is_fullscreen;
    }

    /// Minimize window
    pub fn minimize(&self) {
        self.window.set_minimized(true);
    }

    /// Close window
    pub fn close(&self) {
        // Window will be closed when dropped
    }

    /// Set window title
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        self.window.set_title(title);
    }

    /// Get scale factor
    pub fn update_scale_factor(&mut self) {
        self.scale_factor = self.window.scale_factor();
    }
}

/// Desktop window parameters for browser chrome
pub fn desktop_window_params() -> WindowParams {
    WindowParams {
        title: String::from("Nova Browser"),
        width: 1280,
        height: 800,
        min_width: 800,
        min_height: 500,
        ..Default::default()
    }
}

/// Tablet window parameters
pub fn tablet_window_params() -> WindowParams {
    WindowParams {
        title: String::from("Nova Browser"),
        width: 1024,
        height: 768,
        min_width: 600,
        min_height: 400,
        ..Default::default()
    }
}

/// Mobile window parameters (compact layout)
pub fn mobile_window_params() -> WindowParams {
    WindowParams {
        title: String::from("Nova Browser"),
        width: 390,
        height: 844,
        min_width: 320,
        min_height: 480,
        ..Default::default()
    }
}