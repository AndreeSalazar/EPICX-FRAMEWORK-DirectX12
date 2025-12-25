//! Render pass abstraction

use crate::math::{Color, Rect};

/// Render pass configuration
#[derive(Debug, Clone)]
pub struct RenderPassConfig {
    pub clear_color: Option<Color>,
    pub clear_depth: Option<f32>,
    pub viewport: Rect,
}

impl Default for RenderPassConfig {
    fn default() -> Self {
        Self {
            clear_color: Some(Color::BLACK),
            clear_depth: Some(1.0),
            viewport: Rect::new(0.0, 0.0, 1280.0, 720.0),
        }
    }
}

/// A render pass represents a single rendering operation
pub struct RenderPass {
    config: RenderPassConfig,
    active: bool,
}

impl RenderPass {
    /// Create a new render pass
    pub fn new(config: RenderPassConfig) -> Self {
        Self {
            config,
            active: false,
        }
    }

    /// Begin the render pass
    pub fn begin(&mut self) {
        self.active = true;
    }

    /// End the render pass
    pub fn end(&mut self) {
        self.active = false;
    }

    /// Check if the pass is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the configuration
    pub fn config(&self) -> &RenderPassConfig {
        &self.config
    }

    /// Set the viewport
    pub fn set_viewport(&mut self, viewport: Rect) {
        self.config.viewport = viewport;
    }
}
