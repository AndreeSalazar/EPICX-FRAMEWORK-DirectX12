//! Descriptor Heap wrapper

use super::{Device, Dx12Result};
use windows::Win32::Graphics::Direct3D12::*;

/// Descriptor handle (CPU and GPU)
#[derive(Debug, Clone, Copy)]
pub struct DescriptorHandle {
    pub cpu: D3D12_CPU_DESCRIPTOR_HANDLE,
    pub gpu: Option<D3D12_GPU_DESCRIPTOR_HANDLE>,
}

/// Descriptor heap wrapper
pub struct DescriptorHeap {
    heap: ID3D12DescriptorHeap,
    heap_type: D3D12_DESCRIPTOR_HEAP_TYPE,
    descriptor_size: u32,
    num_descriptors: u32,
    shader_visible: bool,
    current_index: u32,
}

impl DescriptorHeap {
    /// Create a new descriptor heap
    pub fn new(
        device: &Device,
        heap_type: D3D12_DESCRIPTOR_HEAP_TYPE,
        num_descriptors: u32,
        shader_visible: bool,
    ) -> Dx12Result<Self> {
        let heap = device.create_descriptor_heap(heap_type, num_descriptors, shader_visible)?;
        let descriptor_size = device.get_descriptor_increment_size(heap_type);

        Ok(Self {
            heap,
            heap_type,
            descriptor_size,
            num_descriptors,
            shader_visible,
            current_index: 0,
        })
    }

    /// Create a CBV/SRV/UAV heap
    pub fn cbv_srv_uav(device: &Device, num_descriptors: u32) -> Dx12Result<Self> {
        Self::new(
            device,
            D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            num_descriptors,
            true,
        )
    }

    /// Create a sampler heap
    pub fn sampler(device: &Device, num_descriptors: u32) -> Dx12Result<Self> {
        Self::new(
            device,
            D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,
            num_descriptors,
            true,
        )
    }

    /// Create an RTV heap
    pub fn rtv(device: &Device, num_descriptors: u32) -> Dx12Result<Self> {
        Self::new(
            device,
            D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            num_descriptors,
            false,
        )
    }

    /// Create a DSV heap
    pub fn dsv(device: &Device, num_descriptors: u32) -> Dx12Result<Self> {
        Self::new(
            device,
            D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
            num_descriptors,
            false,
        )
    }

    /// Get the raw heap
    pub fn raw(&self) -> &ID3D12DescriptorHeap {
        &self.heap
    }

    /// Get the heap type
    pub fn heap_type(&self) -> D3D12_DESCRIPTOR_HEAP_TYPE {
        self.heap_type
    }

    /// Get the descriptor size
    pub fn descriptor_size(&self) -> u32 {
        self.descriptor_size
    }

    /// Get a handle at a specific index
    pub fn get_handle(&self, index: u32) -> DescriptorHandle {
        unsafe {
            let cpu_start = self.heap.GetCPUDescriptorHandleForHeapStart();
            let cpu = D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: cpu_start.ptr + (index * self.descriptor_size) as usize,
            };

            let gpu = if self.shader_visible {
                let gpu_start = self.heap.GetGPUDescriptorHandleForHeapStart();
                Some(D3D12_GPU_DESCRIPTOR_HANDLE {
                    ptr: gpu_start.ptr + (index * self.descriptor_size) as u64,
                })
            } else {
                None
            };

            DescriptorHandle { cpu, gpu }
        }
    }

    /// Allocate the next descriptor handle
    pub fn allocate(&mut self) -> Option<DescriptorHandle> {
        if self.current_index >= self.num_descriptors {
            return None;
        }

        let handle = self.get_handle(self.current_index);
        self.current_index += 1;
        Some(handle)
    }

    /// Reset the allocator
    pub fn reset(&mut self) {
        self.current_index = 0;
    }

    /// Get the number of allocated descriptors
    pub fn allocated_count(&self) -> u32 {
        self.current_index
    }

    /// Get the total capacity
    pub fn capacity(&self) -> u32 {
        self.num_descriptors
    }
}
