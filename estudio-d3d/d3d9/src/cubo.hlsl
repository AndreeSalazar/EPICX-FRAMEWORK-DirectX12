// BMOX-9 · Shader Model 3.0 (vs_3_0 / ps_3_0)
//
// En D3D9 NO existen los constant buffers: las constantes son registros float4
// sueltos (c0, c1, ...) que se llenan con SetVertexShaderConstantF.
// Una matriz 4x4 ocupa 4 registros.

float4x4 WVP   : register(c0);   // c0..c3
float4x4 World : register(c4);   // c4..c7

struct VSIn  { float3 pos : POSITION; float3 nrm : NORMAL; float4 col : COLOR0; };
// SM3: la posición de salida usa la semántica POSITION (en SM4+ es SV_POSITION)
struct VSOut { float4 pos : POSITION; float3 nrm : TEXCOORD0; float4 col : COLOR0; };
// El pixel shader de SM3 no puede leer POSITION, por eso tiene su propia entrada
struct PSIn  { float3 nrm : TEXCOORD0; float4 col : COLOR0; };

VSOut VSMain(VSIn i)
{
    VSOut o;
    o.pos = mul(WVP, float4(i.pos, 1.0));
    o.nrm = mul((float3x3)World, i.nrm);
    o.col = i.col;
    return o;
}

// SM3: la salida del pixel shader es COLOR0 (en SM4+ es SV_TARGET)
float4 PSMain(PSIn i) : COLOR0
{
    float3 luz = normalize(float3(0.4, 0.8, -0.5));
    float d = saturate(dot(normalize(i.nrm), luz));
    return float4(i.col.rgb * (0.25 + 0.75 * d), i.col.a);
}
