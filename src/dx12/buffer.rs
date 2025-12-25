//! Buffer resources for DirectX12

use super::{Device, Dx12Error, Dx12Result};
use windows::Win32::Graphics::Direct3D12::*;

/// Buffer usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferUsage {
    Vertex,
    Index,
    Constant,
    Structured,
    Upload,
    Readback,
}

/// Buffer description
#[derive(Debug, Clone)]
pub struct BufferDesc {
    pub size: u64,
    pub usage: BufferUsage,
    pub stride: u32,
}

/// Generic buffer wrapper
pub struct Buffer {
    resource: ID3D12Resource,
    desc: BufferDesc,
    gpu_address: u64,
}

impl Buffer {
    /// Create a new buffer
    pub fn new(device: &Device, desc: BufferDesc) -> Dx12Result<Self> {
        unsafe {
            let heap_type = match desc.usage {
                BufferUsage::Upload => D3D12_HEAP_TYPE_UPLOAD,
                BufferUsage::Readback => D3D12_HEAP_TYPE_READBACK,
                _ => D3D12_HEAP_TYPE_DEFAULT,
            };

            let heap_props = D3D12_HEAP_PROPERTIES {
                Type: heap_type,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            };

            let resource_desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Alignment: 0,
                Width: desc.size,
                Height: 1,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_UNKNOWN,
                SampleDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                Flags: D3D12_RESOURCE_FLAG_NONE,
            };

            let initial_state = match desc.usage {
                BufferUsage::Upload => D3D12_RESOURCE_STATE_GENERIC_READ,
                BufferUsage::Readback => D3D12_RESOURCE_STATE_COPY_DEST,
                _ => D3D12_RESOURCE_STATE_COMMON,
            };

            let mut resource: Option<ID3D12Resource> = None;
            device.raw().CreateCommittedResource(
                &heap_props,
                D3D12_HEAP_FLAG_NONE,
                &resource_desc,
                initial_state,
                None,
                &mut resource,
            )?;

            let resource = resource.ok_or_else(|| {
                Dx12Error::BufferCreation("Failed to create buffer resource".to_string())
            })?;

            let gpu_address = resource.GetGPUVirtualAddress();

            Ok(Self {
                resource,
                desc,
                gpu_address,
            })
        }
    }

    /// Get the raw resource
    pub fn raw(&self) -> &ID3D12Resource {
        &self.resource
    }

    /// Get the GPU virtual address
    pub fn gpu_address(&self) -> u64 {
        self.gpu_address
    }

    /// Get the buffer size
    pub fn size(&self) -> u64 {
        self.desc.size
    }

    /// Map the buffer for CPU access
    pub fn map(&self) -> Dx12Result<*mut u8> {
        unsafe {
            let mut data: *mut std::ffi::c_void = std::ptr::null_mut();
            self.resource.Map(0, None, Some(&mut data))?;
            Ok(data as *mut u8)
        }
    }

    /// Unmap the buffer
    pub fn unmap(&self) {
        unsafe {
            self.resource.Unmap(0, None);
        }
    }

    /// Write data to the buffer (for upload buffers)
    pub fn write<T: Copy>(&self, data: &[T]) -> Dx12Result<()> {
        let ptr = self.map()?;
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const u8,
                ptr,
                std::mem::size_of_val(data),
            );
        }
        self.unmap();
        Ok(())
    }
}

/// Vertex buffer wrapper
pub struct VertexBuffer {
    buffer: Buffer,
    view: D3D12_VERTEX_BUFFER_VIEW,
}

impl VertexBuffer {
    /// Create a new vertex buffer
    pub fn new(device: &Device, size: u64, stride: u32) -> Dx12Result<Self> {
        let buffer = Buffer::new(
            device,
            BufferDesc {
                size,
                usage: BufferUsage::Upload,
                stride,
            },
        )?;

        let view = D3D12_VERTEX_BUFFER_VIEW {
            BufferLocation: buffer.gpu_address(),
            SizeInBytes: size as u32,
            StrideInBytes: stride,
        };

        Ok(Self { buffer, view })
    }

    /// Get the vertex buffer view
    pub fn view(&self) -> &D3D12_VERTEX_BUFFER_VIEW {
        &self.view
    }

    /// Write vertex data
    pub fn write<T: Copy>(&self, data: &[T]) -> Dx12Result<()> {
        self.buffer.write(data)
    }
}

/// Index buffer wrapper
pub struct IndexBuffer {
    buffer: Buffer,
    view: D3D12_INDEX_BUFFER_VIEW,
    index_count: u32,
}

impl IndexBuffer {
    /// Create a new index buffer (16-bit indices)
    pub fn new_u16(device: &Device, count: u32) -> Dx12Result<Self> {
        let size = (count * 2) as u64;
        let buffer = Buffer::new(
            device,
            BufferDesc {
                size,
                usage: BufferUsage::Upload,
                stride: 2,
            },
        )?;

        let view = D3D12_INDEX_BUFFER_VIEW {
            BufferLocation: buffer.gpu_address(),
            SizeInBytes: size as u32,
            Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R16_UINT,
        };

        Ok(Self {
            buffer,
            view,
            index_count: count,
        })
    }

    /// Create a new index buffer (32-bit indices)
    pub fn new_u32(device: &Device, count: u32) -> Dx12Result<Self> {
        let size = (count * 4) as u64;
        let buffer = Buffer::new(
            device,
            BufferDesc {
                size,
                usage: BufferUsage::Upload,
                stride: 4,
            },
        )?;

        let view = D3D12_INDEX_BUFFER_VIEW {
            BufferLocation: buffer.gpu_address(),
            SizeInBytes: size as u32,
            Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32_UINT,
        };

        Ok(Self {
            buffer,
            view,
            index_count: count,
        })
    }

    /// Get the index buffer view
    pub fn view(&self) -> &D3D12_INDEX_BUFFER_VIEW {
        &self.view
    }

    /// Get the index count
    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    /// Write index data
    pub fn write<T: Copy>(&self, data: &[T]) -> Dx12Result<()> {
        self.buffer.write(data)
    }
}

/// Constant buffer wrapper
pub struct ConstantBuffer {
    buffer: Buffer,
    aligned_size: u64,
}

impl ConstantBuffer {
    /// Create a new constant buffer
    pub fn new(device: &Device, size: u64) -> Dx12Result<Self> {
        // Constant buffers must be 256-byte aligned
        let aligned_size = (size + 255) & !255;

        let buffer = Buffer::new(
            device,
            BufferDesc {
                size: aligned_size,
                usage: BufferUsage::Upload,
                stride: 0,
            },
        )?;

        Ok(Self {
            buffer,
            aligned_size,
        })
    }

    /// Get the GPU address
    pub fn gpu_address(&self) -> u64 {
        self.buffer.gpu_address()
    }

    /// Get the aligned size
    pub fn aligned_size(&self) -> u64 {
        self.aligned_size
    }

    /// Write constant data
    pub fn write<T: Copy>(&self, data: &T) -> Dx12Result<()> {
        let ptr = self.buffer.map()?;
        unsafe {
            std::ptr::copy_nonoverlapping(
                data as *const T as *const u8,
                ptr,
                std::mem::size_of::<T>(),
            );
        }
        self.buffer.unmap();
        Ok(())
    }
}
