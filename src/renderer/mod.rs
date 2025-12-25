//! Renderer module for EPICX
//!
//! Handles the rendering pipeline and element tree traversal.

mod render_pass;

pub use render_pass::RenderPass;

use crate::core::Element;
use crate::dx12::{Device, CommandQueue, SwapChain, CommandList, CommandAllocator};
use crate::math::Color;
use thiserror::Error;

/// Renderer errors
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to initialize renderer: {0}")]
    Initialization(String),
    #[error("Failed to render frame: {0}")]
    RenderFrame(String),
    #[error("DirectX12 error: {0}")]
    Dx12(#[from] crate::dx12::Dx12Error),
}

pub type RenderResult<T> = Result<T, RenderError>;

/// The main renderer
pub struct Renderer {
    device: Device,
    command_queue: CommandQueue,
    swap_chain: Option<SwapChain>,
    clear_color: Color,
    frame_count: u64,
}

impl Renderer {
    /// Create a new renderer
    pub fn new(debug: bool) -> RenderResult<Self> {
        let device = Device::new(debug)?;
        let command_queue = CommandQueue::graphics(&device)?;

        Ok(Self {
            device,
            command_queue,
            swap_chain: None,
            clear_color: Color::BLACK,
            frame_count: 0,
        })
    }

    /// Get the device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get the command queue
    pub fn command_queue(&self) -> &CommandQueue {
        &self.command_queue
    }

    /// Set the clear color
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Get the current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Begin a new frame
    pub fn begin_frame(&mut self) -> RenderResult<()> {
        // In a full implementation, this would:
        // 1. Wait for the previous frame to complete
        // 2. Reset command allocators
        // 3. Begin recording commands
        Ok(())
    }

    /// Render an element tree
    pub fn render_element(&mut self, element: &Element) -> RenderResult<()> {
        // Traverse the element tree and generate draw commands
        self.render_element_recursive(element)?;
        Ok(())
    }

    fn render_element_recursive(&mut self, element: &Element) -> RenderResult<()> {
        // Skip invisible elements
        if !element.style.visible {
            return Ok(());
        }

        // Render this element based on its type
        match &element.element_type {
            crate::core::element::ElementType::Empty => {}
            crate::core::element::ElementType::Rect => {
                // Draw rectangle
            }
            crate::core::element::ElementType::Circle => {
                // Draw circle
            }
            crate::core::element::ElementType::Text => {
                // Draw text
            }
            crate::core::element::ElementType::Image => {
                // Draw image
            }
            crate::core::element::ElementType::Group => {
                // Just render children
            }
            _ => {}
        }

        // Render children
        for child in &element.children {
            self.render_element_recursive(child)?;
        }

        Ok(())
    }

    /// End the frame and present
    pub fn end_frame(&mut self) -> RenderResult<()> {
        // In a full implementation, this would:
        // 1. Close command lists
        // 2. Execute command lists
        // 3. Present the swap chain
        // 4. Signal fence

        self.frame_count += 1;
        Ok(())
    }

    /// Flush all pending GPU work
    pub fn flush(&mut self) -> RenderResult<()> {
        self.command_queue.flush()?;
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // Ensure all GPU work is complete before destroying resources
        let _ = self.flush();
    }
}
