//! Graphics Pipeline wrapper

use super::{Device, Dx12Error, Dx12Result, Shader};
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::*};
use std::ffi::CString;

/// Root signature wrapper
pub struct RootSignature {
    signature: ID3D12RootSignature,
}

impl RootSignature {
    /// Create a simple root signature
    pub fn new_simple(device: &Device) -> Dx12Result<Self> {
        unsafe {
            let desc = D3D12_ROOT_SIGNATURE_DESC {
                NumParameters: 0,
                pParameters: std::ptr::null(),
                NumStaticSamplers: 0,
                pStaticSamplers: std::ptr::null(),
                Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
            };

            let mut signature_blob = None;
            let mut error_blob = None;

            D3D12SerializeRootSignature(
                &desc,
                D3D_ROOT_SIGNATURE_VERSION_1,
                &mut signature_blob,
                Some(&mut error_blob),
            )?;

            let signature_blob = signature_blob.ok_or_else(|| {
                Dx12Error::PipelineCreation("Failed to serialize root signature".to_string())
            })?;

            let signature: ID3D12RootSignature = device.raw().CreateRootSignature(
                0,
                std::slice::from_raw_parts(
                    signature_blob.GetBufferPointer() as *const u8,
                    signature_blob.GetBufferSize(),
                ),
            )?;

            Ok(Self { signature })
        }
    }

    /// Get the raw root signature
    pub fn raw(&self) -> &ID3D12RootSignature {
        &self.signature
    }
}

/// Pipeline state wrapper
pub struct PipelineState {
    state: ID3D12PipelineState,
}

impl PipelineState {
    /// Get the raw pipeline state
    pub fn raw(&self) -> &ID3D12PipelineState {
        &self.state
    }
}

/// Graphics pipeline builder
pub struct Pipeline {
    root_signature: RootSignature,
    pipeline_state: Option<PipelineState>,
}

impl Pipeline {
    /// Create a new pipeline with a root signature
    pub fn new(device: &Device) -> Dx12Result<Self> {
        let root_signature = RootSignature::new_simple(device)?;
        Ok(Self {
            root_signature,
            pipeline_state: None,
        })
    }

    /// Get the root signature
    pub fn root_signature(&self) -> &RootSignature {
        &self.root_signature
    }

    /// Get the pipeline state
    pub fn pipeline_state(&self) -> Option<&PipelineState> {
        self.pipeline_state.as_ref()
    }

    /// Create a simple graphics pipeline
    pub fn create_graphics_pipeline(
        device: &Device,
        root_signature: &RootSignature,
        vertex_shader: &[u8],
        pixel_shader: &[u8],
        input_layout: &[D3D12_INPUT_ELEMENT_DESC],
    ) -> Dx12Result<PipelineState> {
        unsafe {
            let desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
                pRootSignature: std::mem::transmute_copy(root_signature.raw()),
                VS: D3D12_SHADER_BYTECODE {
                    pShaderBytecode: vertex_shader.as_ptr() as *const _,
                    BytecodeLength: vertex_shader.len(),
                },
                PS: D3D12_SHADER_BYTECODE {
                    pShaderBytecode: pixel_shader.as_ptr() as *const _,
                    BytecodeLength: pixel_shader.len(),
                },
                BlendState: D3D12_BLEND_DESC {
                    AlphaToCoverageEnable: false.into(),
                    IndependentBlendEnable: false.into(),
                    RenderTarget: [
                        D3D12_RENDER_TARGET_BLEND_DESC {
                            BlendEnable: false.into(),
                            LogicOpEnable: false.into(),
                            SrcBlend: D3D12_BLEND_ONE,
                            DestBlend: D3D12_BLEND_ZERO,
                            BlendOp: D3D12_BLEND_OP_ADD,
                            SrcBlendAlpha: D3D12_BLEND_ONE,
                            DestBlendAlpha: D3D12_BLEND_ZERO,
                            BlendOpAlpha: D3D12_BLEND_OP_ADD,
                            LogicOp: D3D12_LOGIC_OP_NOOP,
                            RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
                        },
                        Default::default(),
                        Default::default(),
                        Default::default(),
                        Default::default(),
                        Default::default(),
                        Default::default(),
                        Default::default(),
                    ],
                },
                SampleMask: u32::MAX,
                RasterizerState: D3D12_RASTERIZER_DESC {
                    FillMode: D3D12_FILL_MODE_SOLID,
                    CullMode: D3D12_CULL_MODE_BACK,
                    FrontCounterClockwise: false.into(),
                    DepthBias: 0,
                    DepthBiasClamp: 0.0,
                    SlopeScaledDepthBias: 0.0,
                    DepthClipEnable: true.into(),
                    MultisampleEnable: false.into(),
                    AntialiasedLineEnable: false.into(),
                    ForcedSampleCount: 0,
                    ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
                },
                DepthStencilState: D3D12_DEPTH_STENCIL_DESC {
                    DepthEnable: false.into(),
                    DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
                    DepthFunc: D3D12_COMPARISON_FUNC_LESS,
                    StencilEnable: false.into(),
                    StencilReadMask: 0xFF,
                    StencilWriteMask: 0xFF,
                    FrontFace: Default::default(),
                    BackFace: Default::default(),
                },
                InputLayout: D3D12_INPUT_LAYOUT_DESC {
                    pInputElementDescs: input_layout.as_ptr(),
                    NumElements: input_layout.len() as u32,
                },
                PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
                NumRenderTargets: 1,
                RTVFormats: [
                    DXGI_FORMAT_R8G8B8A8_UNORM,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_FORMAT_UNKNOWN,
                ],
                DSVFormat: DXGI_FORMAT_UNKNOWN,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                ..Default::default()
            };

            let state: ID3D12PipelineState = device.raw().CreateGraphicsPipelineState(&desc)?;
            Ok(PipelineState { state })
        }
    }
}
