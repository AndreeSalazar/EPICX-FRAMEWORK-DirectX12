//! Graphics Context - manages rendering state

use super::{Graphics, GraphicsConfig};
use crate::dx12::{Device, CommandQueue, Dx12Result};
use crate::math::{Color, Rect};

/// Graphics context for rendering operations
/// 
/// Provides a stateful interface for rendering commands.
pub struct GraphicsContext<'a> {
    graphics: &'a mut Graphics,
    viewport: Rect,
    clear_color: Color,
}

impl<'a> GraphicsContext<'a> {
    /// Create a new graphics context
    pub fn new(graphics: &'a mut Graphics) -> Self {
        let config = graphics.config().clone();
        Self {
            graphics,
            viewport: Rect::new(0.0, 0.0, config.width as f32, config.height as f32),
            clear_color: config.clear_color,
        }
    }

    /// Get the device
    pub fn device(&self) -> &Device {
        self.graphics.device()
    }

    /// Get the command queue
    pub fn command_queue(&self) -> &CommandQueue {
        self.graphics.command_queue()
    }

    /// Set the viewport
    pub fn set_viewport(&mut self, viewport: Rect) {
        self.viewport = viewport;
    }

    /// Get the current viewport
    pub fn viewport(&self) -> Rect {
        self.viewport
    }

    /// Set the clear color
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Clear the screen
    pub fn clear(&mut self) {
        // In a full implementation, this would issue clear commands
    }

    /// Present the frame
    pub fn present(&mut self) -> Dx12Result<()> {
        self.graphics.end_frame()
    }
}
