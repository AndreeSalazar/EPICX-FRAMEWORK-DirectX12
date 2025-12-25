//! Graphics abstraction layer (Level B)
//!
//! This module provides a mid-level abstraction over DirectX12.
//! It inherits from dx12 (Level A) and provides a cleaner API.
//!
//! Hierarchy:
//! - Level A (dx12): Raw DirectX12 wrappers
//! - Level B (graphics): This module - cleaner abstractions
//! - Level C (easy): Simple, high-level API for general use

mod context;
mod frame;
mod resources;

pub use context::GraphicsContext;
pub use frame::{Frame, FrameResources};
pub use resources::{GpuBuffer, GpuTexture, GpuMesh, Material};

use crate::dx12::{Device, CommandQueue, SwapChain, Dx12Result, Dx12Error};
use crate::math::Color;

/// Graphics configuration
#[derive(Debug, Clone)]
pub struct GraphicsConfig {
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub debug: bool,
    pub buffer_count: u32,
    pub clear_color: Color,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            vsync: true,
            debug: cfg!(debug_assertions),
            buffer_count: 2,
            clear_color: Color::BLACK,
        }
    }
}

/// Main graphics system - Level B abstraction
/// 
/// Encapsulates all DirectX12 resources and provides a cleaner API.
pub struct Graphics {
    device: Device,
    command_queue: CommandQueue,
    config: GraphicsConfig,
    frame_index: u64,
}

impl Graphics {
    /// Create a new graphics system
    pub fn new(config: GraphicsConfig) -> Dx12Result<Self> {
        let device = Device::new(config.debug)?;
        let command_queue = CommandQueue::graphics(&device)?;

        Ok(Self {
            device,
            command_queue,
            config,
            frame_index: 0,
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

    /// Get mutable command queue
    pub fn command_queue_mut(&mut self) -> &mut CommandQueue {
        &mut self.command_queue
    }

    /// Get the configuration
    pub fn config(&self) -> &GraphicsConfig {
        &self.config
    }

    /// Get current frame index
    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    /// Begin a new frame
    pub fn begin_frame(&mut self) -> Dx12Result<()> {
        self.frame_index += 1;
        Ok(())
    }

    /// End the current frame
    pub fn end_frame(&mut self) -> Dx12Result<()> {
        Ok(())
    }

    /// Flush all GPU work
    pub fn flush(&mut self) -> Dx12Result<()> {
        self.command_queue.flush()
    }

    /// Resize the graphics system
    pub fn resize(&mut self, width: u32, height: u32) -> Dx12Result<()> {
        self.config.width = width;
        self.config.height = height;
        Ok(())
    }
}

impl Drop for Graphics {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
