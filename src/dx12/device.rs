//! DirectX12 Device wrapper

use super::{Dx12Error, Dx12Result};
use windows::{
    Win32::Graphics::{
        Direct3D::D3D_FEATURE_LEVEL_12_0,
        Direct3D12::*,
        Dxgi::*,
    },
};

/// Wrapper around ID3D12Device
pub struct Device {
    device: ID3D12Device,
    adapter: IDXGIAdapter1,
    factory: IDXGIFactory4,
    debug_enabled: bool,
}

impl Device {
    /// Create a new DirectX12 device
    pub fn new(debug: bool) -> Dx12Result<Self> {
        unsafe {
            // Enable debug layer if requested
            if debug {
                let mut debug_controller: Option<ID3D12Debug> = None;
                if D3D12GetDebugInterface(&mut debug_controller).is_ok() {
                    if let Some(debug) = debug_controller {
                        debug.EnableDebugLayer();
                    }
                }
            }

            // Create DXGI factory
            let factory_flags = if debug { DXGI_CREATE_FACTORY_DEBUG.0 } else { 0 };
            let factory: IDXGIFactory4 = CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(factory_flags))?;

            // Find a suitable adapter
            let adapter = Self::find_adapter(&factory)?;

            // Create the device
            let mut device: Option<ID3D12Device> = None;
            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_0, &mut device)?;

            let device = device.ok_or_else(|| {
                Dx12Error::DeviceCreation("Failed to create D3D12 device".to_string())
            })?;

            Ok(Self {
                device,
                adapter,
                factory,
                debug_enabled: debug,
            })
        }
    }

    /// Find a suitable GPU adapter
    unsafe fn find_adapter(factory: &IDXGIFactory4) -> Dx12Result<IDXGIAdapter1> {
        let mut adapter_index = 0;
        loop {
            let adapter = factory.EnumAdapters1(adapter_index);
            match adapter {
                Ok(adapter) => {
                    let desc = adapter.GetDesc1()?;
                    
                    // Skip software adapters
                    if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0 {
                        adapter_index += 1;
                        continue;
                    }

                    // Check if the adapter supports D3D12
                    let result: Result<(), windows::core::Error> = D3D12CreateDevice(
                        &adapter,
                        D3D_FEATURE_LEVEL_12_0,
                        std::ptr::null_mut::<Option<ID3D12Device>>(),
                    );

                    if result.is_ok() {
                        return Ok(adapter);
                    }

                    adapter_index += 1;
                }
                Err(_) => break,
            }
        }

        Err(Dx12Error::DeviceCreation(
            "No suitable DirectX12 adapter found".to_string(),
        ))
    }

    /// Get the raw device handle
    pub fn raw(&self) -> &ID3D12Device {
        &self.device
    }

    /// Get the DXGI factory
    pub fn factory(&self) -> &IDXGIFactory4 {
        &self.factory
    }

    /// Get the adapter
    pub fn adapter(&self) -> &IDXGIAdapter1 {
        &self.adapter
    }

    /// Check if debug mode is enabled
    pub fn is_debug_enabled(&self) -> bool {
        self.debug_enabled
    }

    /// Create a command queue
    pub fn create_command_queue(
        &self,
        queue_type: D3D12_COMMAND_LIST_TYPE,
    ) -> Dx12Result<ID3D12CommandQueue> {
        unsafe {
            let desc = D3D12_COMMAND_QUEUE_DESC {
                Type: queue_type,
                Priority: D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0,
                Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
                NodeMask: 0,
            };

            let queue: ID3D12CommandQueue = self.device.CreateCommandQueue(&desc)?;
            Ok(queue)
        }
    }

    /// Create a command allocator
    pub fn create_command_allocator(
        &self,
        list_type: D3D12_COMMAND_LIST_TYPE,
    ) -> Dx12Result<ID3D12CommandAllocator> {
        unsafe {
            let allocator: ID3D12CommandAllocator =
                self.device.CreateCommandAllocator(list_type)?;
            Ok(allocator)
        }
    }

    /// Create a descriptor heap
    pub fn create_descriptor_heap(
        &self,
        heap_type: D3D12_DESCRIPTOR_HEAP_TYPE,
        num_descriptors: u32,
        shader_visible: bool,
    ) -> Dx12Result<ID3D12DescriptorHeap> {
        unsafe {
            let desc = D3D12_DESCRIPTOR_HEAP_DESC {
                Type: heap_type,
                NumDescriptors: num_descriptors,
                Flags: if shader_visible {
                    D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE
                } else {
                    D3D12_DESCRIPTOR_HEAP_FLAG_NONE
                },
                NodeMask: 0,
            };

            let heap: ID3D12DescriptorHeap = self.device.CreateDescriptorHeap(&desc)?;
            Ok(heap)
        }
    }

    /// Get descriptor handle increment size
    pub fn get_descriptor_increment_size(
        &self,
        heap_type: D3D12_DESCRIPTOR_HEAP_TYPE,
    ) -> u32 {
        unsafe { self.device.GetDescriptorHandleIncrementSize(heap_type) }
    }

    /// Create a fence
    pub fn create_fence(&self, initial_value: u64) -> Dx12Result<ID3D12Fence> {
        unsafe {
            let fence: ID3D12Fence =
                self.device.CreateFence(initial_value, D3D12_FENCE_FLAG_NONE)?;
            Ok(fence)
        }
    }
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("debug_enabled", &self.debug_enabled)
            .finish()
    }
}
