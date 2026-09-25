// Estudio D3D9 · Shader Model 3.0 (vs_3_0 / ps_3_0)
//
// En D3D9 NO existen los constant buffers: las constantes son registros float4
// sueltos. El vertex shader y el pixel shader tienen bancos SEPARADOS:
//   VS: c0..c3 = WVP, c4..c7 = World   (SetVertexShaderConstantF)
//   PS: c0     = Luz                   (SetPixelShaderConstantF)
// Los valores salen de cubo_neutro::constantes(), igual que en el rasterizador por CPU.

float4x4 WVP   : register(c0);
float4x4 World : register(c4);
// c8.xy = ajuste de medio píxel en NDC: (0,0), o (-1/ancho, +1/alto) con --corregir-medio-pixel
float4   Ajuste : register(c8);

struct VSIn  { float3 pos : POSITION; float3 nrm : NORMAL; float4 col : COLOR0; };
// SM3: la posición de salida usa la semántica POSITION (en SM4+ es SV_POSITION)
struct VSOut { float4 pos : POSITION; float3 nrm : TEXCOORD0; float4 col : TEXCOORD1; };

VSOut VSMain(VSIn i)
{
    VSOut o;
    o.pos = mul(WVP, float4(i.pos, 1.0));
    o.pos.xy += Ajuste.xy * o.pos.w;   // se multiplica por w porque luego se divide por w
    o.nrm = mul((float3x3)World, i.nrm);
    o.col = i.col;
    return o;
}

// xyz = dirección hacia la luz (normalizada), w = ambiente
float4 Luz : register(c0);

// El pixel shader de SM3 no puede leer POSITION, por eso tiene su propia entrada.
// El color viaja en TEXCOORD1 (float completo) y no en COLOR0, que en SM3 puede ser de baja precisión.
struct PSIn { float3 nrm : TEXCOORD0; float4 col : TEXCOORD1; };

// SM3: la salida del pixel shader es COLOR0 (en SM4+ es SV_TARGET)
float4 PSMain(PSIn i) : COLOR0
{
    float d = saturate(dot(normalize(i.nrm), Luz.xyz));
    return float4(i.col.rgb * (Luz.w + (1.0 - Luz.w) * d), i.col.a);
}
