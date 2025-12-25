/**
 * ADead-GPU: Cube with SDF Anti-Aliasing
 * 
 * Este shader demuestra anti-aliasing matemáticamente perfecto
 * usando Signed Distance Fields en lugar de MSAA.
 * 
 * Resultado: Bordes perfectos con 0% memoria extra.
 */

// Incluir biblioteca SDF
#include "sdf_antialiasing.hlsl"

// ============================================================================
//                    CONSTANT BUFFERS
// ============================================================================

cbuffer TransformBuffer : register(b0) {
    row_major float4x4 WorldViewProj;
    row_major float4x4 World;
    float3 CameraPos;
    float Time;
};

// ============================================================================
//                    ESTRUCTURAS
// ============================================================================

struct VSInput {
    float3 position : POSITION;
    float4 color : COLOR;
    float3 normal : NORMAL;
};

struct PSInput {
    float4 clipPos : SV_POSITION;
    float4 color : COLOR;
    float3 worldPos : TEXCOORD0;
    float3 localPos : TEXCOORD1;
    float3 normal : TEXCOORD2;
    float3 barycentricHint : TEXCOORD3;  // Para edge AA
};

// ============================================================================
//                    VERTEX SHADER
// ============================================================================

PSInput VSMain(VSInput input, uint vertexID : SV_VertexID) {
    PSInput output;
    
    // Transformaciones
    output.clipPos = mul(float4(input.position, 1.0), WorldViewProj);
    output.worldPos = mul(float4(input.position, 1.0), World).xyz;
    output.localPos = input.position;  // Posición local para SDF
    output.normal = mul(float4(input.normal, 0.0), World).xyz;
    output.color = input.color;
    
    // Coordenadas baricéntricas aproximadas para edge AA
    // En un cubo, usamos la posición local como hint
    output.barycentricHint = frac(input.position + 0.5);
    
    return output;
}

// ============================================================================
//                    PIXEL SHADER - SDF Anti-Aliasing
// ============================================================================

float4 PSMain(PSInput input) : SV_TARGET {
    // ========================================
    // 1. COLOR BASE
    // ========================================
    float4 baseColor = input.color;
    
    // ========================================
    // 2. SDF DEL CUBO
    // ========================================
    // El cubo tiene half-size de 0.5 en cada eje
    float sdf = sdBox(input.localPos, float3(0.5, 0.5, 0.5));
    
    // ========================================
    // 3. ANTI-ALIASING MATEMÁTICO
    // ========================================
    // Esta es la magia: smoothstep convierte distancia en alpha suave
    float aa = computeSdfAA(sdf);
    
    // ========================================
    // 4. EDGE ENHANCEMENT (opcional)
    // ========================================
    // Detectar bordes usando el gradiente del SDF
    float edgeDist = abs(sdf);
    float edgeWidth = fwidth(sdf) * 2.0;
    float edgeFactor = 1.0 - smoothstep(0, edgeWidth, edgeDist);
    
    // Oscurecer ligeramente los bordes para definición
    float3 edgeColor = baseColor.rgb * 0.7;
    baseColor.rgb = lerp(baseColor.rgb, edgeColor, edgeFactor * 0.3);
    
    // ========================================
    // 5. ILUMINACIÓN SIMPLE (opcional)
    // ========================================
    float3 lightDir = normalize(float3(1, 1, -1));
    float3 normal = normalize(input.normal);
    float ndotl = max(0.3, dot(normal, lightDir));
    
    // ========================================
    // 6. OUTPUT FINAL
    // ========================================
    float3 finalColor = baseColor.rgb * ndotl;
    
    return float4(finalColor, aa);
}

// ============================================================================
//                    PIXEL SHADER - Solo SDF (sin iluminación)
// ============================================================================

float4 PSMain_PureSDF(PSInput input) : SV_TARGET {
    // SDF del cubo
    float sdf = sdBox(input.localPos, float3(0.5, 0.5, 0.5));
    
    // Anti-aliasing puro
    float aa = computeSdfAA(sdf);
    
    // Color con AA aplicado
    return float4(input.color.rgb, aa);
}

// ============================================================================
//                    PIXEL SHADER - Wireframe SDF
// ============================================================================

float4 PSMain_Wireframe(PSInput input) : SV_TARGET {
    // SDF del cubo
    float sdf = sdBox(input.localPos, float3(0.5, 0.5, 0.5));
    
    // Detectar bordes (donde SDF está cerca de 0)
    float edgeDist = abs(sdf);
    float wireWidth = fwidth(sdf) * 3.0;  // 3 pixels de grosor
    float wire = 1.0 - smoothstep(0, wireWidth, edgeDist);
    
    // Color del wireframe
    float3 wireColor = float3(0.0, 1.0, 0.5);  // Verde neón
    float3 fillColor = input.color.rgb * 0.1;   // Color tenue de relleno
    
    float3 finalColor = lerp(fillColor, wireColor, wire);
    float alpha = max(wire, 0.1);  // Siempre un poco visible
    
    return float4(finalColor, alpha);
}

// ============================================================================
//                    PIXEL SHADER - Visualización de SDF
// ============================================================================

float4 PSMain_VisualizeSDF(PSInput input) : SV_TARGET {
    // SDF del cubo
    float sdf = sdBox(input.localPos, float3(0.5, 0.5, 0.5));
    
    // Visualizar SDF como color
    // Azul = dentro (negativo), Rojo = fuera (positivo)
    float3 insideColor = float3(0.0, 0.3, 1.0);   // Azul
    float3 outsideColor = float3(1.0, 0.3, 0.0);  // Rojo
    float3 edgeColor = float3(1.0, 1.0, 1.0);     // Blanco
    
    // Normalizar SDF para visualización
    float normalizedSDF = sdf * 2.0;  // Escalar para mejor visualización
    
    // Color basado en distancia
    float3 color;
    if (abs(sdf) < 0.02) {
        // Cerca del borde: blanco
        color = edgeColor;
    } else if (sdf < 0) {
        // Dentro: azul
        color = insideColor * (1.0 + normalizedSDF);
    } else {
        // Fuera: rojo
        color = outsideColor * (1.0 - normalizedSDF);
    }
    
    // Añadir líneas de contorno
    float contour = frac(abs(sdf) * 10.0);
    contour = smoothstep(0.0, 0.1, contour) * smoothstep(0.2, 0.1, contour);
    color = lerp(color, float3(1,1,1), contour * 0.3);
    
    return float4(color, 1.0);
}

// ============================================================================
//                    PIXEL SHADER - Comparación Split-Screen
// ============================================================================

float4 PSMain_Compare(PSInput input) : SV_TARGET {
    // SDF del cubo
    float sdf = sdBox(input.localPos, float3(0.5, 0.5, 0.5));
    
    // Dividir pantalla: izquierda = sin AA, derecha = con AA
    bool rightSide = input.clipPos.x > 640;  // Asume 1280 de ancho
    
    float4 color = input.color;
    
    if (rightSide) {
        // CON SDF Anti-Aliasing
        float aa = computeSdfAA(sdf);
        color.a = aa;
    } else {
        // SIN Anti-Aliasing (hard edge)
        color.a = sdf < 0 ? 1.0 : 0.0;
    }
    
    // Línea divisoria
    if (abs(input.clipPos.x - 640) < 2) {
        return float4(1, 1, 0, 1);  // Línea amarilla
    }
    
    return color;
}

