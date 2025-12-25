//! Window management for EPICX

use crate::math::Rect;
use thiserror::Error;

/// Window errors
#[derive(Error, Debug)]
pub enum WindowError {
    #[error("Failed to create window: {0}")]
    Creation(String),
    #[error("Window system error: {0}")]
    System(String),
}

pub type WindowResult<T> = Result<T, WindowError>;

/// Window configuration
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub fullscreen: bool,
    pub vsync: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "EPICX Window".to_string(),
            width: 1280,
            height: 720,
            resizable: true,
            fullscreen: false,
            vsync: true,
        }
    }
}

/// Window wrapper
pub struct Window {
    config: WindowConfig,
    should_close: bool,
}

impl Window {
    /// Create a new window
    pub fn new(config: WindowConfig) -> WindowResult<Self> {
        // In a full implementation, this would create an actual window
        // using winit or raw Win32 API
        
        log::info!("Creating window: {} ({}x{})", config.title, config.width, config.height);
        
        Ok(Self {
            config,
            should_close: false,
        })
    }

    /// Get the window configuration
    pub fn config(&self) -> &WindowConfig {
        &self.config
    }

    /// Get the window size
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Get the window bounds as a Rect
    pub fn bounds(&self) -> Rect {
        Rect::new(0.0, 0.0, self.config.width as f32, self.config.height as f32)
    }

    /// Check if the window should close
    pub fn should_close(&self) -> bool {
        self.should_close
    }

    /// Request the window to close
    pub fn close(&mut self) {
        self.should_close = true;
    }

    /// Set the window title
    pub fn set_title(&mut self, title: &str) {
        self.config.title = title.to_string();
    }

    /// Resize the window
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
    }

    /// Set fullscreen mode
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.config.fullscreen = fullscreen;
    }

    /// Poll window events
    pub fn poll_events(&mut self) {
        // In a full implementation, this would poll OS events
    }
}
