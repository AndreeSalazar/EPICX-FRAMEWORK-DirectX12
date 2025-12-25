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
pub mod renderer3d;

pub use context::GraphicsContext;
pub use frame::{Frame, FrameResources};
pub use resources::{GpuBuffer, GpuTexture, GpuMesh, Material};
pub use renderer3d::{Vertex3D, Mesh3D, Camera3D, Transform3D, Object3D, TransformConstants};

use crate::dx12::{Device, CommandQueue, SwapChain, SwapChainConfig, CommandAllocator, CommandList, Dx12Result};
use crate::math::Color;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D12::*;

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
            clear_color: Color::from_hex(0x1a1a2e),
        }
    }
}

/// Main graphics system - Level B abstraction
/// 
/// Encapsulates ALL DirectX12 resources and provides a cleaner API.
/// This is the main entry point for graphics operations.
pub struct Graphics {
    device: Device,
    command_queue: CommandQueue,
    swap_chain: SwapChain,
    allocator: CommandAllocator,
    config: GraphicsConfig,
    frame_index: u64,
}

impl Graphics {
    /// Create a new graphics system with a window
    pub fn new(hwnd: HWND, config: GraphicsConfig) -> Dx12Result<Self> {
        let device = Device::new(config.debug)?;
        let command_queue = CommandQueue::graphics(&device)?;
        
        let swap_config = SwapChainConfig {
            width: config.width,
            height: config.height,
            buffer_count: config.buffer_count,
            vsync: config.vsync,
            ..Default::default()
        };
        
        let swap_chain = SwapChain::new(&device, &command_queue, hwnd, swap_config)?;
        let allocator = CommandAllocator::new(&device, D3D12_COMMAND_LIST_TYPE_DIRECT)?;

        Ok(Self {
            device,
            command_queue,
            swap_chain,
            allocator,
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
    
    /// Get width
    pub fn width(&self) -> u32 {
        self.config.width
    }
    
    /// Get height
    pub fn height(&self) -> u32 {
        self.config.height
    }

    /// Begin a new frame - returns a RenderFrame for drawing
    pub fn begin_frame(&mut self) -> Dx12Result<RenderFrame> {
        self.frame_index += 1;
        self.allocator.reset()?;
        
        let cmd_list = CommandList::new(&self.device, &self.allocator, None)?;
        let back_buffer = self.swap_chain.current_back_buffer();
        let rtv = self.swap_chain.current_rtv();
        
        // Transition to render target
        unsafe {
            let barrier = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: std::mem::transmute_copy(back_buffer),
                        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: D3D12_RESOURCE_STATE_PRESENT,
                        StateAfter: D3D12_RESOURCE_STATE_RENDER_TARGET,
                    }),
                },
            };
            cmd_list.raw().ResourceBarrier(&[barrier]);
        }
        
        Ok(RenderFrame {
            cmd_list,
            rtv,
            back_buffer: std::mem::ManuallyDrop::new(Some(back_buffer.clone())),
            width: self.config.width,
            height: self.config.height,
        })
    }

    /// End the current frame and present
    pub fn end_frame(&mut self, frame: RenderFrame) -> Dx12Result<()> {
        // Transition back to present
        unsafe {
            let barrier = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: std::mem::transmute_copy(&frame.back_buffer),
                        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: D3D12_RESOURCE_STATE_RENDER_TARGET,
                        StateAfter: D3D12_RESOURCE_STATE_PRESENT,
                    }),
                },
            };
            frame.cmd_list.raw().ResourceBarrier(&[barrier]);
        }
        
        frame.cmd_list.close()?;
        self.command_queue.execute(&[&frame.cmd_list]);
        self.swap_chain.present()?;
        self.command_queue.flush()?;
        
        Ok(())
    }

    /// Flush all GPU work
    pub fn flush(&mut self) -> Dx12Result<()> {
        self.command_queue.flush()
    }

    /// Resize the graphics system
    pub fn resize(&mut self, width: u32, height: u32) -> Dx12Result<()> {
        if width == 0 || height == 0 {
            return Ok(());
        }
        self.flush()?;
        self.swap_chain.resize(&self.device, width, height)?;
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

/// A frame being rendered - provides simple drawing API
pub struct RenderFrame {
    cmd_list: CommandList,
    rtv: D3D12_CPU_DESCRIPTOR_HANDLE,
    back_buffer: std::mem::ManuallyDrop<Option<ID3D12Resource>>,
    pub width: u32,
    pub height: u32,
}

impl RenderFrame {
    /// Clear the screen with a color
    pub fn clear(&self, color: Color) {
        let clear_color = [color.r, color.g, color.b, color.a];
        unsafe {
            self.cmd_list.raw().ClearRenderTargetView(self.rtv, &clear_color, None);
        }
    }
    
    /// Clear with RGBA values
    pub fn clear_rgba(&self, r: f32, g: f32, b: f32, a: f32) {
        let clear_color = [r, g, b, a];
        unsafe {
            self.cmd_list.raw().ClearRenderTargetView(self.rtv, &clear_color, None);
        }
    }
    
    /// Get the raw command list for advanced operations
    pub fn cmd_list(&self) -> &CommandList {
        &self.cmd_list
    }
    
    /// Set viewport
    pub fn set_viewport(&self, x: f32, y: f32, width: f32, height: f32) {
        let viewport = D3D12_VIEWPORT {
            TopLeftX: x,
            TopLeftY: y,
            Width: width,
            Height: height,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };
        unsafe {
            self.cmd_list.raw().RSSetViewports(&[viewport]);
        }
    }
    
    /// Set scissor rect
    pub fn set_scissor(&self, left: i32, top: i32, right: i32, bottom: i32) {
        let scissor = windows::Win32::Foundation::RECT {
            left,
            top,
            right,
            bottom,
        };
        unsafe {
            self.cmd_list.raw().RSSetScissorRects(&[scissor]);
        }
    }
    
    /// Set full viewport and scissor
    pub fn set_full_viewport(&self) {
        self.set_viewport(0.0, 0.0, self.width as f32, self.height as f32);
        self.set_scissor(0, 0, self.width as i32, self.height as i32);
    }
}
