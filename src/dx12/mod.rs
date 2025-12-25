//! DirectX12 abstraction layer for EPICX
//!
//! This module provides safe Rust wrappers around DirectX12 APIs.

mod device;
mod command_queue;
mod swap_chain;
mod pipeline;
mod buffer;
mod texture;
mod descriptor_heap;
mod fence;
mod shader;

pub use device::Device;
pub use command_queue::{CommandQueue, CommandList, CommandAllocator};
pub use swap_chain::{SwapChain, SwapChainConfig};
pub use pipeline::{Pipeline, PipelineState, RootSignature};
pub use buffer::{Buffer, BufferDesc, BufferUsage, VertexBuffer, IndexBuffer, ConstantBuffer};
pub use texture::{Texture, TextureDesc, RenderTarget, DepthStencil};
pub use descriptor_heap::{DescriptorHeap, DescriptorHandle};
pub use fence::Fence;
pub use shader::{Shader, ShaderType, ShaderCompiler};

use thiserror::Error;

/// DirectX12 errors
#[derive(Error, Debug)]
pub enum Dx12Error {
    #[error("Failed to create device: {0}")]
    DeviceCreation(String),
    #[error("Failed to create command queue: {0}")]
    CommandQueueCreation(String),
    #[error("Failed to create swap chain: {0}")]
    SwapChainCreation(String),
    #[error("Failed to create pipeline: {0}")]
    PipelineCreation(String),
    #[error("Failed to create buffer: {0}")]
    BufferCreation(String),
    #[error("Failed to create texture: {0}")]
    TextureCreation(String),
    #[error("Failed to compile shader: {0}")]
    ShaderCompilation(String),
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    #[error("Windows API error: {0}")]
    WindowsApi(#[from] windows::core::Error),
}

pub type Dx12Result<T> = Result<T, Dx12Error>;
