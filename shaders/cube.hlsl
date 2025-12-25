// ADead-GPU Cube Shader
// With transform matrix support
// Target: NVIDIA RTX 3060 (SM 6.6)

// Constant buffer for transforms
cbuffer TransformBuffer : register(b0) {
    float4x4 WorldViewProj;
    float4x4 World;
    float4 CameraPos;
    float Time;
    float3 _pad;
};

struct VSInput {
    float3 position : POSITION;
    float3 normal : NORMAL;
    float4 color : COLOR;
    float2 texcoord : TEXCOORD;
};

struct PSInput {
    float4 position : SV_POSITION;
    float3 worldPos : WORLDPOS;
    float3 normal : NORMAL;
    float4 color : COLOR;
    float2 texcoord : TEXCOORD;
};

// Vertex Shader
PSInput VSMain(VSInput input) {
    PSInput output;
    
    output.position = mul(WorldViewProj, float4(input.position, 1.0f));
    output.worldPos = mul(World, float4(input.position, 1.0f)).xyz;
    output.normal = normalize(mul((float3x3)World, input.normal));
    output.color = input.color;
    output.texcoord = input.texcoord;
    
    return output;
}

// Pixel Shader with basic lighting
float4 PSMain(PSInput input) : SV_TARGET {
    // Simple directional light
    float3 lightDir = normalize(float3(0.5f, 1.0f, 0.3f));
    float3 viewDir = normalize(CameraPos.xyz - input.worldPos);
    
    // Diffuse
    float ndotl = max(dot(input.normal, lightDir), 0.0f);
    float3 diffuse = input.color.rgb * ndotl;
    
    // Ambient
    float3 ambient = input.color.rgb * 0.2f;
    
    // Specular (Blinn-Phong)
    float3 halfVec = normalize(lightDir + viewDir);
    float spec = pow(max(dot(input.normal, halfVec), 0.0f), 32.0f);
    float3 specular = float3(1.0f, 1.0f, 1.0f) * spec * 0.5f;
    
    float3 finalColor = ambient + diffuse + specular;
    
    return float4(finalColor, input.color.a);
}

