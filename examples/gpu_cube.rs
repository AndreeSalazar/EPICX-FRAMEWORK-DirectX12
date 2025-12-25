//! GPU Cube - Pure DirectX12 Hardware Accelerated
//! 
//! A rotating 3D cube rendered entirely on the GPU using DirectX12.
//! No software rasterization - pure GPU power!
//!
//! Run with: cargo run --example gpu_cube --release

use std::time::Instant;
use std::mem::size_of;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::core::Interface;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::{
    Direct3D::*,
    Direct3D::Fxc::*,
    Direct3D12::*,
    Dxgi::*,
    Dxgi::Common::*,
};

// ============================================================================
// VERTEX DATA - Cube with colors
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

const CUBE_VERTICES: [Vertex; 24] = [
    // Front face (red)
    Vertex { position: [-1.0, -1.0,  1.0], color: [1.0, 0.2, 0.2, 1.0] },
    Vertex { position: [ 1.0, -1.0,  1.0], color: [1.0, 0.2, 0.2, 1.0] },
    Vertex { position: [ 1.0,  1.0,  1.0], color: [1.0, 0.3, 0.3, 1.0] },
    Vertex { position: [-1.0,  1.0,  1.0], color: [1.0, 0.3, 0.3, 1.0] },
    // Back face (green)
    Vertex { position: [ 1.0, -1.0, -1.0], color: [0.2, 1.0, 0.2, 1.0] },
    Vertex { position: [-1.0, -1.0, -1.0], color: [0.2, 1.0, 0.2, 1.0] },
    Vertex { position: [-1.0,  1.0, -1.0], color: [0.3, 1.0, 0.3, 1.0] },
    Vertex { position: [ 1.0,  1.0, -1.0], color: [0.3, 1.0, 0.3, 1.0] },
    // Top face (blue)
    Vertex { position: [-1.0,  1.0,  1.0], color: [0.2, 0.2, 1.0, 1.0] },
    Vertex { position: [ 1.0,  1.0,  1.0], color: [0.2, 0.2, 1.0, 1.0] },
    Vertex { position: [ 1.0,  1.0, -1.0], color: [0.3, 0.3, 1.0, 1.0] },
    Vertex { position: [-1.0,  1.0, -1.0], color: [0.3, 0.3, 1.0, 1.0] },
    // Bottom face (yellow)
    Vertex { position: [-1.0, -1.0, -1.0], color: [1.0, 1.0, 0.2, 1.0] },
    Vertex { position: [ 1.0, -1.0, -1.0], color: [1.0, 1.0, 0.2, 1.0] },
    Vertex { position: [ 1.0, -1.0,  1.0], color: [1.0, 1.0, 0.3, 1.0] },
    Vertex { position: [-1.0, -1.0,  1.0], color: [1.0, 1.0, 0.3, 1.0] },
    // Right face (magenta)
    Vertex { position: [ 1.0, -1.0,  1.0], color: [1.0, 0.2, 1.0, 1.0] },
    Vertex { position: [ 1.0, -1.0, -1.0], color: [1.0, 0.2, 1.0, 1.0] },
    Vertex { position: [ 1.0,  1.0, -1.0], color: [1.0, 0.3, 1.0, 1.0] },
    Vertex { position: [ 1.0,  1.0,  1.0], color: [1.0, 0.3, 1.0, 1.0] },
    // Left face (cyan)
    Vertex { position: [-1.0, -1.0, -1.0], color: [0.2, 1.0, 1.0, 1.0] },
    Vertex { position: [-1.0, -1.0,  1.0], color: [0.2, 1.0, 1.0, 1.0] },
    Vertex { position: [-1.0,  1.0,  1.0], color: [0.3, 1.0, 1.0, 1.0] },
    Vertex { position: [-1.0,  1.0, -1.0], color: [0.3, 1.0, 1.0, 1.0] },
];

const CUBE_INDICES: [u16; 36] = [
    0,  1,  2,  0,  2,  3,   // Front
    4,  5,  6,  4,  6,  7,   // Back
    8,  9,  10, 8,  10, 11,  // Top
    12, 13, 14, 12, 14, 15,  // Bottom
    16, 17, 18, 16, 18, 19,  // Right
    20, 21, 22, 20, 22, 23,  // Left
];

// ============================================================================
// CONSTANT BUFFER - Transform matrices
// ============================================================================

#[repr(C, align(256))]
#[derive(Clone, Copy)]
struct Constants {
    world: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
}

// ============================================================================
// HLSL SHADERS - Compiled at runtime
// ============================================================================

const VERTEX_SHADER: &str = r#"
cbuffer Constants : register(b0) {
    float4x4 World;
    float4x4 View;
    float4x4 Projection;
};

struct VSInput {
    float3 position : POSITION;
    float4 color : COLOR;
};

struct PSInput {
    float4 position : SV_POSITION;
    float4 color : COLOR;
};

PSInput main(VSInput input) {
    PSInput output;
    float4 worldPos = mul(float4(input.position, 1.0), World);
    float4 viewPos = mul(worldPos, View);
    output.position = mul(viewPos, Projection);
    output.color = input.color;
    return output;
}
"#;

const PIXEL_SHADER: &str = r#"
struct PSInput {
    float4 position : SV_POSITION;
    float4 color : COLOR;
};

float4 main(PSInput input) : SV_TARGET {
    return input.color;
}
"#;

// ============================================================================
// MATH HELPERS - Clean, no branches
// ============================================================================

fn mat4_identity() -> [[f32; 4]; 4] {
    [[1.0, 0.0, 0.0, 0.0],
     [0.0, 1.0, 0.0, 0.0],
     [0.0, 0.0, 1.0, 0.0],
     [0.0, 0.0, 0.0, 1.0]]
}

fn mat4_rotation_y(angle: f32) -> [[f32; 4]; 4] {
    let (s, c) = angle.sin_cos();
    [[c, 0.0, s, 0.0],
     [0.0, 1.0, 0.0, 0.0],
     [-s, 0.0, c, 0.0],
     [0.0, 0.0, 0.0, 1.0]]
}

fn mat4_rotation_x(angle: f32) -> [[f32; 4]; 4] {
    let (s, c) = angle.sin_cos();
    [[1.0, 0.0, 0.0, 0.0],
     [0.0, c, -s, 0.0],
     [0.0, s, c, 0.0],
     [0.0, 0.0, 0.0, 1.0]]
}

fn mat4_multiply(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            result[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j];
        }
    }
    result
}

fn mat4_look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let zaxis = normalize([eye[0] - target[0], eye[1] - target[1], eye[2] - target[2]]);
    let xaxis = normalize(cross(up, zaxis));
    let yaxis = cross(zaxis, xaxis);
    
    [[xaxis[0], yaxis[0], zaxis[0], 0.0],
     [xaxis[1], yaxis[1], zaxis[1], 0.0],
     [xaxis[2], yaxis[2], zaxis[2], 0.0],
     [-dot(xaxis, eye), -dot(yaxis, eye), -dot(zaxis, eye), 1.0]]
}

fn mat4_perspective(fov: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let h = 1.0 / (fov * 0.5).tan();
    let w = h / aspect;
    let r = far / (near - far);
    
    [[w, 0.0, 0.0, 0.0],
     [0.0, h, 0.0, 0.0],
     [0.0, 0.0, r, -1.0],
     [0.0, 0.0, r * near, 0.0]]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / len, v[1] / len, v[2] / len]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

// ============================================================================
// GPU RENDERER - Pure DirectX12
// ============================================================================

struct GpuRenderer {
    device: ID3D12Device,
    command_queue: ID3D12CommandQueue,
    swap_chain: IDXGISwapChain3,
    rtv_heap: ID3D12DescriptorHeap,
    dsv_heap: ID3D12DescriptorHeap,
    rtv_size: u32,
    render_targets: Vec<ID3D12Resource>,
    depth_buffer: ID3D12Resource,
    command_allocators: Vec<ID3D12CommandAllocator>,
    command_list: ID3D12GraphicsCommandList,
    fence: ID3D12Fence,
    fence_values: Vec<u64>,
    fence_event: windows::Win32::Foundation::HANDLE,
    frame_index: u32,
    
    // Pipeline
    root_signature: ID3D12RootSignature,
    pipeline_state: ID3D12PipelineState,
    
    // Buffers
    vertex_buffer: ID3D12Resource,
    vertex_buffer_view: D3D12_VERTEX_BUFFER_VIEW,
    index_buffer: ID3D12Resource,
    index_buffer_view: D3D12_INDEX_BUFFER_VIEW,
    constant_buffer: ID3D12Resource,
    cbv_heap: ID3D12DescriptorHeap,
    
    // State
    width: u32,
    height: u32,
    rotation: f32,
    gpu_name: String,
}

impl GpuRenderer {
    fn new(hwnd: HWND, width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            // Create factory and find best GPU
            let factory: IDXGIFactory6 = CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0))?;
            
            let adapter: IDXGIAdapter1 = factory.EnumAdapterByGpuPreference(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE)?;
            let desc = adapter.GetDesc1()?;
            let gpu_name = String::from_utf16_lossy(&desc.Description[..desc.Description.iter().position(|&c| c == 0).unwrap_or(128)]);
            
            println!("[GPU] Using: {}", gpu_name);
            println!("[GPU] VRAM: {} MB", desc.DedicatedVideoMemory / (1024 * 1024));
            
            // Create device
            let mut device: Option<ID3D12Device> = None;
            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_0, &mut device)?;
            let device = device.unwrap();
            
            // Command queue
            let queue_desc = D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                ..Default::default()
            };
            let command_queue: ID3D12CommandQueue = device.CreateCommandQueue(&queue_desc)?;
            
            // Swap chain
            let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: width,
                Height: height,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                BufferCount: 2,
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                ..Default::default()
            };
            
            let swap_chain: IDXGISwapChain3 = factory.CreateSwapChainForHwnd(&command_queue, hwnd, &swap_chain_desc, None, None)?.cast()?;
            let frame_index = swap_chain.GetCurrentBackBufferIndex();
            
            // RTV heap
            let rtv_heap: ID3D12DescriptorHeap = device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                NumDescriptors: 2,
                ..Default::default()
            })?;
            let rtv_size = device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);
            
            // Create RTVs
            let mut render_targets = Vec::new();
            let mut rtv_handle = rtv_heap.GetCPUDescriptorHandleForHeapStart();
            for i in 0..2 {
                let rt: ID3D12Resource = swap_chain.GetBuffer(i)?;
                device.CreateRenderTargetView(&rt, None, rtv_handle);
                render_targets.push(rt);
                rtv_handle.ptr += rtv_size as usize;
            }
            
            // DSV heap and depth buffer
            let dsv_heap: ID3D12DescriptorHeap = device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
                NumDescriptors: 1,
                ..Default::default()
            })?;
            
            let depth_buffer = Self::create_depth_buffer(&device, width, height)?;
            device.CreateDepthStencilView(&depth_buffer, None, dsv_heap.GetCPUDescriptorHandleForHeapStart());
            
            // Command allocators
            let mut command_allocators = Vec::new();
            for _ in 0..2 {
                let allocator: ID3D12CommandAllocator = device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;
                command_allocators.push(allocator);
            }
            
            // Command list
            let command_list: ID3D12GraphicsCommandList = device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &command_allocators[0], None)?;
            command_list.Close()?;
            
            // Fence
            let fence: ID3D12Fence = device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?;
            let fence_event = windows::Win32::System::Threading::CreateEventW(None, false, false, None)?;
            
            // Create pipeline
            let (root_signature, pipeline_state) = Self::create_pipeline(&device)?;
            
            // Create buffers
            let (vertex_buffer, vertex_buffer_view) = Self::create_vertex_buffer(&device)?;
            let (index_buffer, index_buffer_view) = Self::create_index_buffer(&device)?;
            let (constant_buffer, cbv_heap) = Self::create_constant_buffer(&device)?;
            
            Ok(Self {
                device,
                command_queue,
                swap_chain,
                rtv_heap,
                dsv_heap,
                rtv_size,
                render_targets,
                depth_buffer,
                command_allocators,
                command_list,
                fence,
                fence_values: vec![0, 0],
                fence_event,
                frame_index,
                root_signature,
                pipeline_state,
                vertex_buffer,
                vertex_buffer_view,
                index_buffer,
                index_buffer_view,
                constant_buffer,
                cbv_heap,
                width,
                height,
                rotation: 0.0,
                gpu_name,
            })
        }
    }
    
    fn create_depth_buffer(device: &ID3D12Device, width: u32, height: u32) -> Result<ID3D12Resource, Box<dyn std::error::Error>> {
        unsafe {
            let heap_props = D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                ..Default::default()
            };
            
            let desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Width: width as u64,
                Height: height,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: DXGI_FORMAT_D32_FLOAT,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Flags: D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
                ..Default::default()
            };
            
            let clear_value = D3D12_CLEAR_VALUE {
                Format: DXGI_FORMAT_D32_FLOAT,
                Anonymous: D3D12_CLEAR_VALUE_0 { DepthStencil: D3D12_DEPTH_STENCIL_VALUE { Depth: 1.0, Stencil: 0 } },
            };
            
            let mut resource: Option<ID3D12Resource> = None;
            device.CreateCommittedResource(&heap_props, D3D12_HEAP_FLAG_NONE, &desc, D3D12_RESOURCE_STATE_DEPTH_WRITE, Some(&clear_value), &mut resource)?;
            Ok(resource.unwrap())
        }
    }
    
    fn create_pipeline(device: &ID3D12Device) -> Result<(ID3D12RootSignature, ID3D12PipelineState), Box<dyn std::error::Error>> {
        unsafe {
            // Root signature with one CBV
            let root_param = D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_CBV,
                Anonymous: D3D12_ROOT_PARAMETER_0 {
                    Descriptor: D3D12_ROOT_DESCRIPTOR { ShaderRegister: 0, RegisterSpace: 0 },
                },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_VERTEX,
            };
            
            let root_sig_desc = D3D12_ROOT_SIGNATURE_DESC {
                NumParameters: 1,
                pParameters: &root_param,
                Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
                ..Default::default()
            };
            
            let mut signature_blob: Option<ID3DBlob> = None;
            let mut error_blob: Option<ID3DBlob> = None;
            D3D12SerializeRootSignature(&root_sig_desc, D3D_ROOT_SIGNATURE_VERSION_1, &mut signature_blob, Some(&mut error_blob))?;
            
            let signature_blob = signature_blob.unwrap();
            let root_signature: ID3D12RootSignature = device.CreateRootSignature(
                0,
                std::slice::from_raw_parts(signature_blob.GetBufferPointer() as *const u8, signature_blob.GetBufferSize()),
            )?;
            
            // Compile shaders
            let vs_blob = Self::compile_shader(VERTEX_SHADER, "main", "vs_5_0")?;
            let ps_blob = Self::compile_shader(PIXEL_SHADER, "main", "ps_5_0")?;
            
            // Input layout
            let input_elements = [
                D3D12_INPUT_ELEMENT_DESC {
                    SemanticName: windows::core::s!("POSITION"),
                    Format: DXGI_FORMAT_R32G32B32_FLOAT,
                    InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                    ..Default::default()
                },
                D3D12_INPUT_ELEMENT_DESC {
                    SemanticName: windows::core::s!("COLOR"),
                    Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
                    AlignedByteOffset: 12,
                    InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                    ..Default::default()
                },
            ];
            
            // Pipeline state
            let pso_desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
                pRootSignature: std::mem::transmute_copy(&root_signature),
                VS: D3D12_SHADER_BYTECODE {
                    pShaderBytecode: vs_blob.GetBufferPointer(),
                    BytecodeLength: vs_blob.GetBufferSize(),
                },
                PS: D3D12_SHADER_BYTECODE {
                    pShaderBytecode: ps_blob.GetBufferPointer(),
                    BytecodeLength: ps_blob.GetBufferSize(),
                },
                InputLayout: D3D12_INPUT_LAYOUT_DESC {
                    pInputElementDescs: input_elements.as_ptr(),
                    NumElements: input_elements.len() as u32,
                },
                RasterizerState: D3D12_RASTERIZER_DESC {
                    FillMode: D3D12_FILL_MODE_SOLID,
                    CullMode: D3D12_CULL_MODE_BACK,
                    FrontCounterClockwise: false.into(),
                    DepthClipEnable: true.into(),
                    ..Default::default()
                },
                BlendState: D3D12_BLEND_DESC {
                    RenderTarget: [D3D12_RENDER_TARGET_BLEND_DESC {
                        RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
                        ..Default::default()
                    }; 8],
                    ..Default::default()
                },
                DepthStencilState: D3D12_DEPTH_STENCIL_DESC {
                    DepthEnable: true.into(),
                    DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
                    DepthFunc: D3D12_COMPARISON_FUNC_LESS,
                    ..Default::default()
                },
                SampleMask: u32::MAX,
                PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
                NumRenderTargets: 1,
                RTVFormats: {
                    let mut formats = [DXGI_FORMAT_UNKNOWN; 8];
                    formats[0] = DXGI_FORMAT_R8G8B8A8_UNORM;
                    formats
                },
                DSVFormat: DXGI_FORMAT_D32_FLOAT,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                ..Default::default()
            };
            
            let pipeline_state: ID3D12PipelineState = device.CreateGraphicsPipelineState(&pso_desc)?;
            
            Ok((root_signature, pipeline_state))
        }
    }
    
    fn compile_shader(source: &str, entry: &str, target: &str) -> Result<ID3DBlob, Box<dyn std::error::Error>> {
        unsafe {
            let mut shader_blob: Option<ID3DBlob> = None;
            let mut error_blob: Option<ID3DBlob> = None;
            
            let entry_cstr = std::ffi::CString::new(entry)?;
            let target_cstr = std::ffi::CString::new(target)?;
            
            let result = D3DCompile(
                source.as_ptr() as *const _,
                source.len(),
                None,
                None,
                None,
                windows::core::PCSTR(entry_cstr.as_ptr() as *const u8),
                windows::core::PCSTR(target_cstr.as_ptr() as *const u8),
                D3DCOMPILE_OPTIMIZATION_LEVEL3,
                0,
                &mut shader_blob,
                Some(&mut error_blob),
            );
            
            if result.is_err() {
                if let Some(err) = error_blob {
                    let msg = std::slice::from_raw_parts(err.GetBufferPointer() as *const u8, err.GetBufferSize());
                    return Err(format!("Shader compile error: {}", String::from_utf8_lossy(msg)).into());
                }
            }
            
            Ok(shader_blob.unwrap())
        }
    }
    
    fn create_vertex_buffer(device: &ID3D12Device) -> Result<(ID3D12Resource, D3D12_VERTEX_BUFFER_VIEW), Box<dyn std::error::Error>> {
        unsafe {
            let buffer_size = size_of::<[Vertex; 24]>() as u64;
            
            let heap_props = D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_UPLOAD, ..Default::default() };
            let desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Width: buffer_size,
                Height: 1,
                DepthOrArraySize: 1,
                MipLevels: 1,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                ..Default::default()
            };
            
            let mut buffer: Option<ID3D12Resource> = None;
            device.CreateCommittedResource(&heap_props, D3D12_HEAP_FLAG_NONE, &desc, D3D12_RESOURCE_STATE_GENERIC_READ, None, &mut buffer)?;
            let buffer = buffer.unwrap();
            
            // Copy data
            let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            buffer.Map(0, None, Some(&mut data_ptr))?;
            std::ptr::copy_nonoverlapping(CUBE_VERTICES.as_ptr(), data_ptr as *mut Vertex, 24);
            buffer.Unmap(0, None);
            
            let view = D3D12_VERTEX_BUFFER_VIEW {
                BufferLocation: buffer.GetGPUVirtualAddress(),
                SizeInBytes: buffer_size as u32,
                StrideInBytes: size_of::<Vertex>() as u32,
            };
            
            Ok((buffer, view))
        }
    }
    
    fn create_index_buffer(device: &ID3D12Device) -> Result<(ID3D12Resource, D3D12_INDEX_BUFFER_VIEW), Box<dyn std::error::Error>> {
        unsafe {
            let buffer_size = size_of::<[u16; 36]>() as u64;
            
            let heap_props = D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_UPLOAD, ..Default::default() };
            let desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Width: buffer_size,
                Height: 1,
                DepthOrArraySize: 1,
                MipLevels: 1,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                ..Default::default()
            };
            
            let mut buffer: Option<ID3D12Resource> = None;
            device.CreateCommittedResource(&heap_props, D3D12_HEAP_FLAG_NONE, &desc, D3D12_RESOURCE_STATE_GENERIC_READ, None, &mut buffer)?;
            let buffer = buffer.unwrap();
            
            let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            buffer.Map(0, None, Some(&mut data_ptr))?;
            std::ptr::copy_nonoverlapping(CUBE_INDICES.as_ptr(), data_ptr as *mut u16, 36);
            buffer.Unmap(0, None);
            
            let view = D3D12_INDEX_BUFFER_VIEW {
                BufferLocation: buffer.GetGPUVirtualAddress(),
                SizeInBytes: buffer_size as u32,
                Format: DXGI_FORMAT_R16_UINT,
            };
            
            Ok((buffer, view))
        }
    }
    
    fn create_constant_buffer(device: &ID3D12Device) -> Result<(ID3D12Resource, ID3D12DescriptorHeap), Box<dyn std::error::Error>> {
        unsafe {
            let buffer_size = (size_of::<Constants>() + 255) & !255; // 256-byte aligned
            
            let heap_props = D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_UPLOAD, ..Default::default() };
            let desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Width: buffer_size as u64,
                Height: 1,
                DepthOrArraySize: 1,
                MipLevels: 1,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                ..Default::default()
            };
            
            let mut buffer: Option<ID3D12Resource> = None;
            device.CreateCommittedResource(&heap_props, D3D12_HEAP_FLAG_NONE, &desc, D3D12_RESOURCE_STATE_GENERIC_READ, None, &mut buffer)?;
            let buffer = buffer.unwrap();
            
            // CBV heap
            let cbv_heap: ID3D12DescriptorHeap = device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                NumDescriptors: 1,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                ..Default::default()
            })?;
            
            Ok((buffer, cbv_heap))
        }
    }
    
    fn update(&mut self, dt: f32) {
        self.rotation += dt * 1.5;
    }
    
    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let frame_idx = self.frame_index as usize;
            
            // Wait for previous frame
            let fence_value = self.fence_values[frame_idx];
            if self.fence.GetCompletedValue() < fence_value {
                self.fence.SetEventOnCompletion(fence_value, self.fence_event)?;
                windows::Win32::System::Threading::WaitForSingleObject(self.fence_event, u32::MAX);
            }
            
            // Update constant buffer
            let aspect = self.width as f32 / self.height as f32;
            let world = mat4_multiply(mat4_rotation_y(self.rotation), mat4_rotation_x(self.rotation * 0.7));
            let view = mat4_look_at([0.0, 2.0, -5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
            let projection = mat4_perspective(std::f32::consts::FRAC_PI_4, aspect, 0.1, 100.0);
            
            let constants = Constants { world, view, projection };
            
            let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            self.constant_buffer.Map(0, None, Some(&mut data_ptr))?;
            std::ptr::copy_nonoverlapping(&constants, data_ptr as *mut Constants, 1);
            self.constant_buffer.Unmap(0, None);
            
            // Reset allocator and command list
            self.command_allocators[frame_idx].Reset()?;
            self.command_list.Reset(&self.command_allocators[frame_idx], &self.pipeline_state)?;
            
            // Set state
            self.command_list.SetGraphicsRootSignature(&self.root_signature);
            self.command_list.SetGraphicsRootConstantBufferView(0, self.constant_buffer.GetGPUVirtualAddress());
            
            let viewport = D3D12_VIEWPORT {
                Width: self.width as f32,
                Height: self.height as f32,
                MaxDepth: 1.0,
                ..Default::default()
            };
            let scissor = windows::Win32::Foundation::RECT {
                right: self.width as i32,
                bottom: self.height as i32,
                ..Default::default()
            };
            self.command_list.RSSetViewports(&[viewport]);
            self.command_list.RSSetScissorRects(&[scissor]);
            
            // Transition to render target
            let barrier = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: std::mem::transmute_copy(&self.render_targets[frame_idx]),
                        StateBefore: D3D12_RESOURCE_STATE_PRESENT,
                        StateAfter: D3D12_RESOURCE_STATE_RENDER_TARGET,
                        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                    }),
                },
                ..Default::default()
            };
            self.command_list.ResourceBarrier(&[barrier]);
            
            // Get RTV and DSV handles
            let mut rtv_handle = self.rtv_heap.GetCPUDescriptorHandleForHeapStart();
            rtv_handle.ptr += (frame_idx * self.rtv_size as usize) as usize;
            let dsv_handle = self.dsv_heap.GetCPUDescriptorHandleForHeapStart();
            
            // Clear and set render targets
            self.command_list.ClearRenderTargetView(rtv_handle, &[0.05, 0.05, 0.1, 1.0], None);
            self.command_list.ClearDepthStencilView(dsv_handle, D3D12_CLEAR_FLAG_DEPTH, 1.0, 0, &[]);
            self.command_list.OMSetRenderTargets(1, Some(&rtv_handle), false, Some(&dsv_handle));
            
            // Draw cube
            self.command_list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            self.command_list.IASetVertexBuffers(0, Some(&[self.vertex_buffer_view]));
            self.command_list.IASetIndexBuffer(Some(&self.index_buffer_view));
            self.command_list.DrawIndexedInstanced(36, 1, 0, 0, 0);
            
            // Transition to present
            let barrier = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: std::mem::transmute_copy(&self.render_targets[frame_idx]),
                        StateBefore: D3D12_RESOURCE_STATE_RENDER_TARGET,
                        StateAfter: D3D12_RESOURCE_STATE_PRESENT,
                        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                    }),
                },
                ..Default::default()
            };
            self.command_list.ResourceBarrier(&[barrier]);
            
            // Execute
            self.command_list.Close()?;
            let cmd_lists = [Some(self.command_list.cast::<ID3D12CommandList>()?)];
            self.command_queue.ExecuteCommandLists(&cmd_lists);
            
            // Present
            self.swap_chain.Present(1, DXGI_PRESENT(0)).ok()?;
            
            // Signal fence
            self.fence_values[frame_idx] = self.fence_values[frame_idx].wrapping_add(1);
            self.command_queue.Signal(&self.fence, self.fence_values[frame_idx])?;
            
            self.frame_index = self.swap_chain.GetCurrentBackBufferIndex();
            
            Ok(())
        }
    }
}

// ============================================================================
// APPLICATION
// ============================================================================

struct App {
    window: Option<Window>,
    renderer: Option<GpuRenderer>,
    last_frame: Instant,
    frame_count: u64,
    last_fps: Instant,
    fps: f32,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            last_frame: Instant::now(),
            frame_count: 0,
            last_fps: Instant::now(),
            fps: 0.0,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!();
        println!("╔═══════════════════════════════════════════════════════════════════╗");
        println!("║           EPICX - Pure GPU DirectX12 Cube                         ║");
        println!("║                                                                   ║");
        println!("║  100% GPU accelerated - No software rendering!                    ║");
        println!("║  Using HLSL shaders compiled to GPU bytecode                      ║");
        println!("║                                                                   ║");
        println!("║  Press ESC to exit                                                ║");
        println!("╚═══════════════════════════════════════════════════════════════════╝");
        println!();
        
        let attrs = Window::default_attributes()
            .with_title("EPICX GPU Cube")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
        
        let window = event_loop.create_window(attrs).expect("Window failed");
        let size = window.inner_size();
        
        let hwnd = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::Win32(h) => HWND(h.hwnd.get() as *mut std::ffi::c_void),
            _ => panic!("Unsupported"),
        };
        
        match GpuRenderer::new(hwnd, size.width, size.height) {
            Ok(renderer) => {
                println!();
                println!("[INIT] DirectX12 pipeline ready!");
                println!("[INIT] Vertex buffer: 24 vertices");
                println!("[INIT] Index buffer: 36 indices");
                println!("[INIT] Shaders compiled to GPU");
                println!();
                self.renderer = Some(renderer);
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to create renderer: {}", e);
                event_loop.exit();
                return;
            }
        }
        
        self.window = Some(window);
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) {
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f32();
                self.last_frame = now;
                
                if let Some(renderer) = &mut self.renderer {
                    renderer.update(dt);
                    if let Err(e) = renderer.render() {
                        eprintln!("Render error: {}", e);
                    }
                }
                
                // FPS
                self.frame_count += 1;
                if self.last_fps.elapsed().as_secs_f32() >= 0.5 {
                    self.fps = self.frame_count as f32 * 2.0;
                    self.frame_count = 0;
                    self.last_fps = Instant::now();
                    
                    if let (Some(window), Some(renderer)) = (&self.window, &self.renderer) {
                        window.set_title(&format!("EPICX GPU Cube | {} | FPS: {:.0}", renderer.gpu_name, self.fps));
                    }
                }
            }
            _ => {}
        }
    }
    
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(w) = &self.window { w.request_redraw(); }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::new())?;
    Ok(())
}
