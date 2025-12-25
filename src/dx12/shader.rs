//! Shader compilation and management

use super::{Dx12Error, Dx12Result};

/// Shader types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Pixel,
    Compute,
    Geometry,
    Hull,
    Domain,
}

impl ShaderType {
    /// Get the shader model target string
    pub fn target(&self) -> &'static str {
        match self {
            ShaderType::Vertex => "vs_5_1",
            ShaderType::Pixel => "ps_5_1",
            ShaderType::Compute => "cs_5_1",
            ShaderType::Geometry => "gs_5_1",
            ShaderType::Hull => "hs_5_1",
            ShaderType::Domain => "ds_5_1",
        }
    }
}

/// Compiled shader bytecode
pub struct Shader {
    bytecode: Vec<u8>,
    shader_type: ShaderType,
}

impl Shader {
    /// Create a shader from pre-compiled bytecode
    pub fn from_bytecode(bytecode: Vec<u8>, shader_type: ShaderType) -> Self {
        Self {
            bytecode,
            shader_type,
        }
    }

    /// Get the bytecode
    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    /// Get the shader type
    pub fn shader_type(&self) -> ShaderType {
        self.shader_type
    }
}

/// Shader compiler (placeholder - actual compilation requires D3DCompiler)
pub struct ShaderCompiler;

impl ShaderCompiler {
    /// Create a new shader compiler
    pub fn new() -> Self {
        Self
    }

    /// Compile HLSL source code
    /// Note: This is a placeholder. Real implementation would use D3DCompiler.
    pub fn compile(
        &self,
        _source: &str,
        _entry_point: &str,
        shader_type: ShaderType,
    ) -> Dx12Result<Shader> {
        // Placeholder - in a real implementation, this would:
        // 1. Call D3DCompile or use dxc (DirectX Shader Compiler)
        // 2. Return the compiled bytecode
        
        Err(Dx12Error::ShaderCompilation(
            "Runtime shader compilation not implemented. Use pre-compiled shaders.".to_string(),
        ))
    }

    /// Load a pre-compiled shader from a file
    pub fn load_compiled(&self, path: &str, shader_type: ShaderType) -> Dx12Result<Shader> {
        let bytecode = std::fs::read(path).map_err(|e| {
            Dx12Error::ShaderCompilation(format!("Failed to load shader file: {}", e))
        })?;

        Ok(Shader::from_bytecode(bytecode, shader_type))
    }
}

impl Default for ShaderCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in shader source code (HLSL)
pub mod builtin {
    /// Simple vertex shader for 2D rendering
    pub const VERTEX_2D: &str = r#"
struct VSInput {
    float2 position : POSITION;
    float4 color : COLOR;
};

struct VSOutput {
    float4 position : SV_POSITION;
    float4 color : COLOR;
};

VSOutput main(VSInput input) {
    VSOutput output;
    output.position = float4(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}
"#;

    /// Simple pixel shader
    pub const PIXEL_SIMPLE: &str = r#"
struct PSInput {
    float4 position : SV_POSITION;
    float4 color : COLOR;
};

float4 main(PSInput input) : SV_TARGET {
    return input.color;
}
"#;

    /// Textured vertex shader
    pub const VERTEX_TEXTURED: &str = r#"
struct VSInput {
    float3 position : POSITION;
    float2 texcoord : TEXCOORD;
};

struct VSOutput {
    float4 position : SV_POSITION;
    float2 texcoord : TEXCOORD;
};

cbuffer Constants : register(b0) {
    float4x4 mvp;
};

VSOutput main(VSInput input) {
    VSOutput output;
    output.position = mul(mvp, float4(input.position, 1.0));
    output.texcoord = input.texcoord;
    return output;
}
"#;

    /// Textured pixel shader
    pub const PIXEL_TEXTURED: &str = r#"
Texture2D tex : register(t0);
SamplerState samp : register(s0);

struct PSInput {
    float4 position : SV_POSITION;
    float2 texcoord : TEXCOORD;
};

float4 main(PSInput input) : SV_TARGET {
    return tex.Sample(samp, input.texcoord);
}
"#;
}
