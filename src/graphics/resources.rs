//! GPU Resources - simplified resource management

use crate::dx12::{Device, Buffer, BufferDesc, BufferUsage, VertexBuffer, IndexBuffer, Texture, TextureDesc, Dx12Result};
use crate::math::{Color, Vec2, Vec3};

/// A GPU buffer with automatic management
pub struct GpuBuffer {
    buffer: Buffer,
    size: u64,
    name: String,
}

impl GpuBuffer {
    /// Create a new GPU buffer
    pub fn new(device: &Device, size: u64, name: impl Into<String>) -> Dx12Result<Self> {
        let buffer = Buffer::new(device, BufferDesc {
            size,
            usage: BufferUsage::Upload,
            stride: 0,
        })?;

        Ok(Self {
            buffer,
            size,
            name: name.into(),
        })
    }

    /// Write data to the buffer
    pub fn write<T: Copy>(&self, data: &[T]) -> Dx12Result<()> {
        self.buffer.write(data)
    }

    /// Get the buffer size
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get the buffer name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the GPU address
    pub fn gpu_address(&self) -> u64 {
        self.buffer.gpu_address()
    }
}

/// A GPU texture with automatic management
pub struct GpuTexture {
    texture: Texture,
    width: u32,
    height: u32,
    name: String,
}

impl GpuTexture {
    /// Create a new GPU texture
    pub fn new(device: &Device, width: u32, height: u32, name: impl Into<String>) -> Dx12Result<Self> {
        let texture = Texture::new(device, TextureDesc {
            width,
            height,
            ..Default::default()
        })?;

        Ok(Self {
            texture,
            width,
            height,
            name: name.into(),
        })
    }

    /// Get the texture width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the texture height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the texture name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Vertex data for a mesh
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub texcoord: Vec2,
    pub color: Color,
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, texcoord: Vec2, color: Color) -> Self {
        Self { position, normal, texcoord, color }
    }

    pub fn simple(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vec3::new(x, y, z),
            normal: Vec3::Y,
            texcoord: Vec2::ZERO,
            color: Color::WHITE,
        }
    }
}

/// A GPU mesh with vertices and indices
pub struct GpuMesh {
    vertex_buffer: VertexBuffer,
    index_buffer: Option<IndexBuffer>,
    vertex_count: u32,
    index_count: u32,
    name: String,
}

impl GpuMesh {
    /// Create a new mesh from vertices
    pub fn new(device: &Device, vertices: &[Vertex], name: impl Into<String>) -> Dx12Result<Self> {
        let vertex_size = std::mem::size_of::<Vertex>();
        let buffer_size = (vertices.len() * vertex_size) as u64;
        
        let vertex_buffer = VertexBuffer::new(device, buffer_size, vertex_size as u32)?;
        vertex_buffer.write(vertices)?;

        Ok(Self {
            vertex_buffer,
            index_buffer: None,
            vertex_count: vertices.len() as u32,
            index_count: 0,
            name: name.into(),
        })
    }

    /// Create a new indexed mesh
    pub fn new_indexed(
        device: &Device,
        vertices: &[Vertex],
        indices: &[u32],
        name: impl Into<String>,
    ) -> Dx12Result<Self> {
        let vertex_size = std::mem::size_of::<Vertex>();
        let buffer_size = (vertices.len() * vertex_size) as u64;
        
        let vertex_buffer = VertexBuffer::new(device, buffer_size, vertex_size as u32)?;
        vertex_buffer.write(vertices)?;

        let index_buffer = IndexBuffer::new_u32(device, indices.len() as u32)?;
        index_buffer.write(indices)?;

        Ok(Self {
            vertex_buffer,
            index_buffer: Some(index_buffer),
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
            name: name.into(),
        })
    }

    /// Get the vertex count
    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }

    /// Get the index count
    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    /// Check if the mesh is indexed
    pub fn is_indexed(&self) -> bool {
        self.index_buffer.is_some()
    }

    /// Get the mesh name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the vertex buffer view
    pub fn vertex_buffer(&self) -> &VertexBuffer {
        &self.vertex_buffer
    }

    /// Get the index buffer (if any)
    pub fn index_buffer(&self) -> Option<&IndexBuffer> {
        self.index_buffer.as_ref()
    }
}

/// Material properties for rendering
#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub base_color: Color,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: Color,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            base_color: Color::WHITE,
            metallic: 0.0,
            roughness: 0.5,
            emissive: Color::BLACK,
        }
    }
}

impl Material {
    /// Create a new material
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the base color
    pub fn with_color(mut self, color: Color) -> Self {
        self.base_color = color;
        self
    }

    /// Set metallic value
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic;
        self
    }

    /// Set roughness value
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness;
        self
    }
}
