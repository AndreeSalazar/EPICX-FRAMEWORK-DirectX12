/**
 * ADead-GPU: SDF Anti-Aliasing Shader Library
 * Pure Mathematics for Perfect Anti-Aliasing
 * 
 * Include this file in your shaders to use SDF-based AA.
 * 
 * Usage:
 *   #include "sdf_antialiasing.hlsl"
 *   
 *   float4 PSMain(VSOutput input) : SV_TARGET {
 *       float sdf = sdBox(input.localPos, float3(0.5, 0.5, 0.5));
 *       float aa = computeSdfAA(sdf);
 *       return float4(color.rgb, aa);
 *   }
 */

#ifndef SDF_ANTIALIASING_HLSL
#define SDF_ANTIALIASING_HLSL

// ============================================================================
//                    SIGNED DISTANCE FUNCTIONS (SDFs)
// ============================================================================

/**
 * SDF de una esfera
 * @param p Punto en espacio local
 * @param r Radio
 */
float sdSphere(float3 p, float r) {
    return length(p) - r;
}

/**
 * SDF de un cubo (box)
 * La función matemática más elegante para un cubo.
 * 
 * @param p Punto en espacio local del cubo
 * @param b Half-size del cubo (mitad del tamaño en cada eje)
 */
float sdBox(float3 p, float3 b) {
    float3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

/**
 * SDF de un cubo redondeado
 * @param r Radio de redondeo de las esquinas
 */
float sdRoundBox(float3 p, float3 b, float r) {
    float3 q = abs(p) - b + r;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0) - r;
}

/**
 * SDF de un cilindro (eje Y)
 */
float sdCylinder(float3 p, float h, float r) {
    float2 d = abs(float2(length(p.xz), p.y)) - float2(r, h);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

/**
 * SDF de un toro (en plano XZ)
 * @param t.x Radio mayor (del centro al tubo)
 * @param t.y Radio menor (del tubo)
 */
float sdTorus(float3 p, float2 t) {
    float2 q = float2(length(p.xz) - t.x, p.y);
    return length(q) - t.y;
}

/**
 * SDF de un cono
 */
float sdCone(float3 p, float2 c, float h) {
    float2 q = h * float2(c.x / c.y, -1.0);
    float2 w = float2(length(p.xz), p.y);
    float2 a = w - q * clamp(dot(w, q) / dot(q, q), 0.0, 1.0);
    float2 b = w - q * float2(clamp(w.x / q.x, 0.0, 1.0), 1.0);
    float k = sign(q.y);
    float d = min(dot(a, a), dot(b, b));
    float s = max(k * (w.x * q.y - w.y * q.x), k * (w.y - q.y));
    return sqrt(d) * sign(s);
}

/**
 * SDF de un plano infinito
 * @param n Normal del plano (debe estar normalizada)
 * @param h Altura/offset del plano
 */
float sdPlane(float3 p, float3 n, float h) {
    return dot(p, n) + h;
}

/**
 * SDF de una cápsula (línea con radio)
 */
float sdCapsule(float3 p, float3 a, float3 b, float r) {
    float3 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

// ============================================================================
//                    OPERACIONES BOOLEANAS
// ============================================================================

/** Unión: A OR B */
float opUnion(float d1, float d2) { 
    return min(d1, d2); 
}

/** Intersección: A AND B */
float opIntersection(float d1, float d2) { 
    return max(d1, d2); 
}

/** Sustracción: A - B */
float opSubtraction(float d1, float d2) { 
    return max(d1, -d2); 
}

/** Unión suave - mezcla gradual entre formas */
float opSmoothUnion(float d1, float d2, float k) {
    float h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return lerp(d2, d1, h) - k * h * (1.0 - h);
}

/** Intersección suave */
float opSmoothIntersection(float d1, float d2, float k) {
    float h = clamp(0.5 - 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return lerp(d2, d1, h) + k * h * (1.0 - h);
}

/** Sustracción suave */
float opSmoothSubtraction(float d1, float d2, float k) {
    float h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0.0, 1.0);
    return lerp(d1, -d2, h) + k * h * (1.0 - h);
}

// ============================================================================
//                    TRANSFORMACIONES
// ============================================================================

/** Repetición infinita en un eje */
float3 opRepeat(float3 p, float3 c) {
    return fmod(abs(p) + 0.5 * c, c) - 0.5 * c;
}

/** Repetición limitada */
float3 opRepeatLimited(float3 p, float c, float3 l) {
    return p - c * clamp(round(p / c), -l, l);
}

/** Simetría en un eje */
float3 opSymX(float3 p) { return float3(abs(p.x), p.y, p.z); }
float3 opSymY(float3 p) { return float3(p.x, abs(p.y), p.z); }
float3 opSymZ(float3 p) { return float3(p.x, p.y, abs(p.z)); }

// ============================================================================
//                    ANTI-ALIASING CORE
// ============================================================================

/**
 * La función CLAVE de anti-aliasing SDF.
 * 
 * Convierte una distancia firmada en un factor de alpha suave.
 * Usa fwidth() para calcular automáticamente el ancho del pixel.
 * 
 * @param sdf Distancia firmada al borde (negativo = dentro)
 * @return Factor de anti-aliasing [0, 1]
 */
float computeSdfAA(float sdf) {
    // fwidth = abs(ddx(sdf)) + abs(ddy(sdf))
    // Representa cuánto cambia el SDF por pixel
    float pixelWidth = fwidth(sdf);
    
    // smoothstep crea transición suave
    // -sdf porque queremos: dentro=1, fuera=0
    return smoothstep(pixelWidth, -pixelWidth, sdf);
}

/**
 * Anti-aliasing con control de suavidad
 * @param softness Multiplicador de suavizado (1.0 = normal, 2.0 = más suave)
 */
float computeSdfAA_Soft(float sdf, float softness) {
    float pixelWidth = fwidth(sdf) * softness;
    return smoothstep(pixelWidth, -pixelWidth, sdf);
}

/**
 * Anti-aliasing para siluetas (solo bordes externos)
 * Útil para personajes y objetos donde solo importa el contorno.
 */
float computeSilhouetteAA(float sdf, float width) {
    float pixelWidth = fwidth(sdf);
    float edge = smoothstep(pixelWidth, -pixelWidth, sdf);
    float innerMask = smoothstep(-width * pixelWidth, 0, sdf);
    return edge * innerMask;
}

// ============================================================================
//                    COORDENADAS BARICÉNTRICAS
// ============================================================================

/**
 * Calcula la distancia al borde más cercano del triángulo
 * usando coordenadas baricéntricas.
 * 
 * @param bary Coordenadas baricéntricas (u, v, w) donde u+v+w=1
 * @return Distancia al borde más cercano
 */
float barycentricEdgeDist(float3 bary) {
    return min(min(bary.x, bary.y), bary.z);
}

/**
 * Anti-aliasing de bordes de triángulo usando baricéntricas
 */
float computeBarycentricAA(float3 bary) {
    float dist = barycentricEdgeDist(bary);
    float pixelWidth = fwidth(dist);
    return smoothstep(0, pixelWidth * 2, dist);
}

/**
 * Wireframe usando coordenadas baricéntricas
 * @param thickness Grosor del wireframe en pixels
 */
float computeWireframe(float3 bary, float thickness) {
    float dist = barycentricEdgeDist(bary);
    float pixelWidth = fwidth(dist);
    return 1.0 - smoothstep(0, pixelWidth * thickness, dist);
}

// ============================================================================
//                    UTILIDADES
// ============================================================================

/**
 * Calcula el gradiente (normal) del SDF
 * Útil para iluminación basada en SDF
 */
float3 calcSDFNormal(float3 p, float sdf) {
    float2 e = float2(0.001, 0);
    return normalize(float3(
        sdf - sdBox(p - e.xyy, float3(0.5, 0.5, 0.5)),
        sdf - sdBox(p - e.yxy, float3(0.5, 0.5, 0.5)),
        sdf - sdBox(p - e.yyx, float3(0.5, 0.5, 0.5))
    ));
}

/**
 * Ambient Occlusion aproximado usando SDF
 */
float calcSDFAO(float3 p, float3 n, float maxDist, int steps) {
    float ao = 0.0;
    float weight = 1.0;
    
    for (int i = 1; i <= steps; i++) {
        float dist = maxDist * float(i) / float(steps);
        float3 samplePos = p + n * dist;
        // Aquí deberías llamar a tu función SDF de la escena
        float sdf = sdBox(samplePos, float3(0.5, 0.5, 0.5));
        ao += weight * (dist - sdf);
        weight *= 0.5;
    }
    
    return saturate(1.0 - ao);
}

// ============================================================================
//                    CONSTANTES
// ============================================================================

static const float SDF_AA_DEFAULT_SOFTNESS = 1.0;
static const float SDF_AA_SILHOUETTE_WIDTH = 2.0;
static const float SDF_EPSILON = 0.0001;

#endif // SDF_ANTIALIASING_HLSL

