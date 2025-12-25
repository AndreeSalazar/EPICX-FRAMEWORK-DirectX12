//! Texture resources for DirectX12

use super::{Device, Dx12Error, Dx12Result};
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::*};

/// Texture description
#[derive(Debug, Clone)]
pub struct TextureDesc {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mip_levels: u32,
    pub format: DXGI_FORMAT,
    pub dimension: D3D12_RESOURCE_DIMENSION,
}

impl Default for TextureDesc {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            depth: 1,
            mip_levels: 1,
            format: DXGI_FORMAT_R8G8B8A8_UNORM,
            dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        }
    }
}

/// Texture wrapper
pub struct Texture {
    resource: ID3D12Resource,
    desc: TextureDesc,
}

impl Texture {
    /// Create a new texture
    pub fn new(device: &Device, desc: TextureDesc) -> Dx12Result<Self> {
        unsafe {
            let heap_props = D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            };

            let resource_desc = D3D12_RESOURCE_DESC {
                Dimension: desc.dimension,
                Alignment: 0,
                Width: desc.width as u64,
                Height: desc.height,
                DepthOrArraySize: desc.depth as u16,
                MipLevels: desc.mip_levels as u16,
                Format: desc.format,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
                Flags: D3D12_RESOURCE_FLAG_NONE,
            };

            let mut resource: Option<ID3D12Resource> = None;
            device.raw().CreateCommittedResource(
                &heap_props,
                D3D12_HEAP_FLAG_NONE,
                &resource_desc,
                D3D12_RESOURCE_STATE_COMMON,
                None,
                &mut resource,
            )?;

            let resource = resource.ok_or_else(|| {
                Dx12Error::TextureCreation("Failed to create texture resource".to_string())
            })?;

            Ok(Self { resource, desc })
        }
    }

    /// Get the raw resource
    pub fn raw(&self) -> &ID3D12Resource {
        &self.resource
    }

    /// Get the texture description
    pub fn desc(&self) -> &TextureDesc {
        &self.desc
    }

    /// Get the width
    pub fn width(&self) -> u32 {
        self.desc.width
    }

    /// Get the height
    pub fn height(&self) -> u32 {
        self.desc.height
    }
}

/// Render target wrapper
pub struct RenderTarget {
    texture: Texture,
    rtv_handle: D3D12_CPU_DESCRIPTOR_HANDLE,
}

impl RenderTarget {
    /// Create a new render target
    pub fn new(
        device: &Device,
        width: u32,
        height: u32,
        format: DXGI_FORMAT,
        rtv_heap: &ID3D12DescriptorHeap,
        heap_index: u32,
    ) -> Dx12Result<Self> {
        let desc = TextureDesc {
            width,
            height,
            format,
            ..Default::default()
        };

        // Create texture with render target flag
        unsafe {
            let heap_props = D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            };

            let resource_desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Alignment: 0,
                Width: width as u64,
                Height: height,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: format,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
                Flags: D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
            };

            let clear_value = D3D12_CLEAR_VALUE {
                Format: format,
                Anonymous: D3D12_CLEAR_VALUE_0 {
                    Color: [0.0, 0.0, 0.0, 1.0],
                },
            };

            let mut resource: Option<ID3D12Resource> = None;
            device.raw().CreateCommittedResource(
                &heap_props,
                D3D12_HEAP_FLAG_NONE,
                &resource_desc,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                Some(&clear_value),
                &mut resource,
            )?;

            let resource = resource.ok_or_else(|| {
                Dx12Error::TextureCreation("Failed to create render target".to_string())
            })?;

            let rtv_descriptor_size =
                device.get_descriptor_increment_size(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);

            let rtv_handle = D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: rtv_heap.GetCPUDescriptorHandleForHeapStart().ptr
                    + (heap_index * rtv_descriptor_size) as usize,
            };

            device.raw().CreateRenderTargetView(&resource, None, rtv_handle);

            Ok(Self {
                texture: Texture { resource, desc },
                rtv_handle,
            })
        }
    }

    /// Get the RTV handle
    pub fn rtv(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        self.rtv_handle
    }

    /// Get the underlying texture
    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}

/// Depth stencil wrapper
pub struct DepthStencil {
    texture: Texture,
    dsv_handle: D3D12_CPU_DESCRIPTOR_HANDLE,
}

impl DepthStencil {
    /// Create a new depth stencil
    pub fn new(
        device: &Device,
        width: u32,
        height: u32,
        dsv_heap: &ID3D12DescriptorHeap,
        heap_index: u32,
    ) -> Dx12Result<Self> {
        unsafe {
            let format = DXGI_FORMAT_D32_FLOAT;

            let heap_props = D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            };

            let resource_desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Alignment: 0,
                Width: width as u64,
                Height: height,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: format,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
                Flags: D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
            };

            let clear_value = D3D12_CLEAR_VALUE {
                Format: format,
                Anonymous: D3D12_CLEAR_VALUE_0 {
                    DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
                        Depth: 1.0,
                        Stencil: 0,
                    },
                },
            };

            let mut resource: Option<ID3D12Resource> = None;
            device.raw().CreateCommittedResource(
                &heap_props,
                D3D12_HEAP_FLAG_NONE,
                &resource_desc,
                D3D12_RESOURCE_STATE_DEPTH_WRITE,
                Some(&clear_value),
                &mut resource,
            )?;

            let resource = resource.ok_or_else(|| {
                Dx12Error::TextureCreation("Failed to create depth stencil".to_string())
            })?;

            let dsv_descriptor_size =
                device.get_descriptor_increment_size(D3D12_DESCRIPTOR_HEAP_TYPE_DSV);

            let dsv_handle = D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: dsv_heap.GetCPUDescriptorHandleForHeapStart().ptr
                    + (heap_index * dsv_descriptor_size) as usize,
            };

            device.raw().CreateDepthStencilView(&resource, None, dsv_handle);

            let desc = TextureDesc {
                width,
                height,
                format,
                ..Default::default()
            };

            Ok(Self {
                texture: Texture { resource, desc },
                dsv_handle,
            })
        }
    }

    /// Get the DSV handle
    pub fn dsv(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        self.dsv_handle
    }

    /// Get the underlying texture
    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}
