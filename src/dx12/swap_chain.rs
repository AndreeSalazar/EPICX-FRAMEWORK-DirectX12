//! Swap Chain wrapper

use super::{Device, Dx12Error, Dx12Result, CommandQueue};
use windows::core::Interface;
use windows::Win32::{
    Foundation::HWND,
    Graphics::{
        Direct3D12::*,
        Dxgi::{Common::*, *},
    },
};

/// Swap chain configuration
#[derive(Debug, Clone)]
pub struct SwapChainConfig {
    pub width: u32,
    pub height: u32,
    pub buffer_count: u32,
    pub format: DXGI_FORMAT,
    pub vsync: bool,
}

impl Default for SwapChainConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            buffer_count: 2,
            format: DXGI_FORMAT_R8G8B8A8_UNORM,
            vsync: true,
        }
    }
}

/// Swap chain wrapper
pub struct SwapChain {
    swap_chain: IDXGISwapChain3,
    config: SwapChainConfig,
    back_buffers: Vec<ID3D12Resource>,
    rtv_heap: ID3D12DescriptorHeap,
    rtv_descriptor_size: u32,
    current_back_buffer: u32,
}

impl SwapChain {
    /// Create a new swap chain
    pub fn new(
        device: &Device,
        command_queue: &CommandQueue,
        hwnd: HWND,
        config: SwapChainConfig,
    ) -> Dx12Result<Self> {
        unsafe {
            let desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: config.width,
                Height: config.height,
                Format: config.format,
                Stereo: false.into(),
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: config.buffer_count,
                Scaling: DXGI_SCALING_STRETCH,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                AlphaMode: DXGI_ALPHA_MODE_UNSPECIFIED,
                Flags: DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH.0 as u32,
            };

            let swap_chain: IDXGISwapChain3 = device
                .factory()
                .CreateSwapChainForHwnd(command_queue.raw(), hwnd, &desc, None, None)?
                .cast()?;

            // Create RTV descriptor heap
            let rtv_heap = device.create_descriptor_heap(
                D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                config.buffer_count,
                false,
            )?;

            let rtv_descriptor_size =
                device.get_descriptor_increment_size(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);

            // Get back buffers and create RTVs
            let mut back_buffers = Vec::with_capacity(config.buffer_count as usize);
            let rtv_handle = rtv_heap.GetCPUDescriptorHandleForHeapStart();

            for i in 0..config.buffer_count {
                let buffer: ID3D12Resource = swap_chain.GetBuffer(i)?;

                let handle = D3D12_CPU_DESCRIPTOR_HANDLE {
                    ptr: rtv_handle.ptr + (i * rtv_descriptor_size) as usize,
                };

                device.raw().CreateRenderTargetView(&buffer, None, handle);
                back_buffers.push(buffer);
            }

            let current_back_buffer = swap_chain.GetCurrentBackBufferIndex();

            Ok(Self {
                swap_chain,
                config,
                back_buffers,
                rtv_heap,
                rtv_descriptor_size,
                current_back_buffer,
            })
        }
    }

    /// Get the current back buffer index
    pub fn current_back_buffer_index(&self) -> u32 {
        self.current_back_buffer
    }

    /// Get the current back buffer resource
    pub fn current_back_buffer(&self) -> &ID3D12Resource {
        &self.back_buffers[self.current_back_buffer as usize]
    }

    /// Get the RTV handle for the current back buffer
    pub fn current_rtv(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe {
            let handle = self.rtv_heap.GetCPUDescriptorHandleForHeapStart();
            D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: handle.ptr + (self.current_back_buffer * self.rtv_descriptor_size) as usize,
            }
        }
    }

    /// Present the frame
    pub fn present(&mut self) -> Dx12Result<()> {
        unsafe {
            let sync_interval = if self.config.vsync { 1 } else { 0 };
            self.swap_chain.Present(sync_interval, DXGI_PRESENT(0)).ok()?;
            self.current_back_buffer = self.swap_chain.GetCurrentBackBufferIndex();
            Ok(())
        }
    }

    /// Resize the swap chain
    pub fn resize(&mut self, device: &Device, width: u32, height: u32) -> Dx12Result<()> {
        unsafe {
            // Release back buffers
            self.back_buffers.clear();

            // Resize buffers
            self.swap_chain.ResizeBuffers(
                self.config.buffer_count,
                width,
                height,
                self.config.format,
                DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH,
            )?;

            // Recreate back buffers and RTVs
            let rtv_handle = self.rtv_heap.GetCPUDescriptorHandleForHeapStart();

            for i in 0..self.config.buffer_count {
                let buffer: ID3D12Resource = self.swap_chain.GetBuffer(i)?;

                let handle = D3D12_CPU_DESCRIPTOR_HANDLE {
                    ptr: rtv_handle.ptr + (i * self.rtv_descriptor_size) as usize,
                };

                device.raw().CreateRenderTargetView(&buffer, None, handle);
                self.back_buffers.push(buffer);
            }

            self.config.width = width;
            self.config.height = height;
            self.current_back_buffer = self.swap_chain.GetCurrentBackBufferIndex();

            Ok(())
        }
    }

    /// Get the swap chain configuration
    pub fn config(&self) -> &SwapChainConfig {
        &self.config
    }

    /// Get the width
    pub fn width(&self) -> u32 {
        self.config.width
    }

    /// Get the height
    pub fn height(&self) -> u32 {
        self.config.height
    }
}
