//! Application entry point for EPICX

use crate::core::{Component, Context, Element, RenderContext};
use crate::dx12::Device;
use crate::renderer::Renderer;
use crate::window::{Window, WindowConfig};
use crate::events::{Event, EventLoop};
use crate::math::Rect;
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;

/// Application errors
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to create window: {0}")]
    WindowCreation(String),
    #[error("Failed to initialize DirectX12: {0}")]
    Dx12Init(String),
    #[error("Render error: {0}")]
    Render(String),
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub debug: bool,
    pub clear_color: crate::math::Color,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "EPICX Application".to_string(),
            width: 1280,
            height: 720,
            vsync: true,
            debug: cfg!(debug_assertions),
            clear_color: crate::math::Color::BLACK,
        }
    }
}

/// The main application struct
pub struct App {
    config: AppConfig,
    context: Arc<RwLock<Context>>,
    running: bool,
}

impl App {
    /// Create a new application with default config
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
            context: Arc::new(RwLock::new(Context::new())),
            running: false,
        }
    }

    /// Create a new application with custom config
    pub fn with_config(config: AppConfig) -> Self {
        Self {
            config,
            context: Arc::new(RwLock::new(Context::new())),
            running: false,
        }
    }

    /// Get a builder for the application
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    /// Get the application context
    pub fn context(&self) -> Arc<RwLock<Context>> {
        Arc::clone(&self.context)
    }

    /// Provide a value to the context
    pub fn provide<T: std::any::Any + Send + Sync + 'static>(&self, value: T) {
        self.context.write().provide(value);
    }

    /// Run the application with a root component
    pub fn run<C, F>(self, create_root: F) -> Result<(), AppError>
    where
        C: Component,
        F: FnOnce() -> C,
    {
        log::info!("Starting EPICX application: {}", self.config.title);
        
        // This is a placeholder - actual implementation would:
        // 1. Create window
        // 2. Initialize DirectX12
        // 3. Create renderer
        // 4. Enter main loop
        
        log::info!("Application initialized successfully");
        Ok(())
    }

    /// Stop the application
    pub fn quit(&mut self) {
        self.running = false;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating applications
pub struct AppBuilder {
    config: AppConfig,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    pub fn vsync(mut self, enabled: bool) -> Self {
        self.config.vsync = enabled;
        self
    }

    pub fn debug(mut self, enabled: bool) -> Self {
        self.config.debug = enabled;
        self
    }

    pub fn clear_color(mut self, color: crate::math::Color) -> Self {
        self.config.clear_color = color;
        self
    }

    pub fn build(self) -> App {
        App::with_config(self.config)
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}
