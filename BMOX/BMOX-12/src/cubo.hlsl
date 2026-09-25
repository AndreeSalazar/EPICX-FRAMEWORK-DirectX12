// BMOX-12 · Shader Model 5.0 (vs_5_0 / ps_5_0, compilado con FXC)
//
// El HLSL es el mismo que en BMOX-10/11. Lo que cambia en D3D12 es CÓMO llega
// el cbuffer al shader: a través de la "root signature" (aquí, un root CBV en b0).
// Para SM 6.x (wave ops, mesh shaders, raytracing) hace falta DXC en lugar de FXC.

cbuffer Constantes : register(b0)
{
    float4x4 WVP;
    float4x4 World;
};

struct VSIn  { float3 pos : POSITION; float3 nrm : NORMAL; float4 col : COLOR; };
struct PSIn  { float4 pos : SV_POSITION; float3 nrm : NORMAL; float4 col : COLOR; };

PSIn VSMain(VSIn i)
{
    PSIn o;
    o.pos = mul(WVP, float4(i.pos, 1.0));
    o.nrm = mul((float3x3)World, i.nrm);
    o.col = i.col;
    return o;
}

float4 PSMain(PSIn i) : SV_TARGET
{
    float3 luz = normalize(float3(0.4, 0.8, -0.5));
    float d = saturate(dot(normalize(i.nrm), luz));
    return float4(i.col.rgb * (0.25 + 0.75 * d), i.col.a);
}
