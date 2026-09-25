// BMOX-11 · Shader Model 5.0 (vs_5_0 / ps_5_0)
//
// Para un cubo el HLSL es idéntico al de BMOX-10: SM5 añade cosas que aquí no
// se usan (compute shaders, teselación, UAVs en todas las etapas).

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
