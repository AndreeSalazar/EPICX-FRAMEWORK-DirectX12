// ADead-GPU Triangle Shader
// Minimal HLSL - compiles to DXIL
// NO fancy features. Just transforms and color.

struct VSInput {
    float3 position : POSITION;
    float4 color : COLOR;
};

struct PSInput {
    float4 position : SV_POSITION;
    float4 color : COLOR;
};

// Vertex Shader - just pass through
PSInput VSMain(VSInput input) {
    PSInput output;
    output.position = float4(input.position, 1.0f);
    output.color = input.color;
    return output;
}

// Pixel Shader - just output color
float4 PSMain(PSInput input) : SV_TARGET {
    return input.color;
}

