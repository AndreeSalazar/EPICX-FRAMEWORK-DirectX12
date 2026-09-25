// Estudio D3D10 · Shader Model 4.0 (vs_4_0 / ps_4_0)
//
// Novedades frente a SM3 (D3D9):
// - Las constantes se agrupan en un cbuffer (constant buffer) en el registro b0,
//   y el MISMO buffer se puede atar al VS y al PS.
// - Semánticas de sistema "SV_": SV_POSITION (salida del VS) y SV_TARGET (salida del PS).
// - El PS puede recibir la misma estructura que devuelve el VS.
// Los valores salen de cubo_neutro::constantes() (struct Constantes, 144 bytes).

cbuffer Constantes : register(b0)
{
    float4x4 WVP;
    float4x4 World;
    float4   Luz;    // xyz = dirección hacia la luz (normalizada), w = ambiente
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
    float d = saturate(dot(normalize(i.nrm), Luz.xyz));
    return float4(i.col.rgb * (Luz.w + (1.0 - Luz.w) * d), i.col.a);
}
