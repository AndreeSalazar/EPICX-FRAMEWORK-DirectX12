// =============================================================================
// ADead-GPU: Vector3D Ray Marching Library
// Inspired by Adobe Illustrator - Pure Mathematics in 3D
// =============================================================================

#ifndef ADEAD_VECTOR3D_RAYMARCHING_HLSL
#define ADEAD_VECTOR3D_RAYMARCHING_HLSL

// =============================================================================
// CONSTANTES
// =============================================================================

#define MAX_STEPS 128
#define MAX_DIST 100.0
#define SURF_DIST 0.001
#define PI 3.14159265359

// =============================================================================
// PRIMITIVAS SDF (Signed Distance Functions)
// =============================================================================

// Esfera: la primitiva más simple
float sdSphere(float3 p, float radius) {
    return length(p) - radius;
}

// Cubo (Box)
float sdBox(float3 p, float3 size) {
    float3 q = abs(p) - size;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Cubo redondeado
float sdRoundBox(float3 p, float3 size, float radius) {
    float3 q = abs(p) - size;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0) - radius;
}

// Cilindro
float sdCylinder(float3 p, float height, float radius) {
    float2 d = abs(float2(length(p.xz), p.y)) - float2(radius, height);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

// Toro (Donut)
float sdTorus(float3 p, float2 t) {
    float2 q = float2(length(p.xz) - t.x, p.y);
    return length(q) - t.y;
}

// Cápsula (línea con grosor)
float sdCapsule(float3 p, float3 a, float3 b, float radius) {
    float3 ab = b - a;
    float3 ap = p - a;
    float t = saturate(dot(ap, ab) / dot(ab, ab));
    float3 c = a + t * ab;
    return length(p - c) - radius;
}

// Cono
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

// Plano infinito
float sdPlane(float3 p, float3 n, float h) {
    return dot(p, normalize(n)) + h;
}

// Elipsoide
float sdEllipsoid(float3 p, float3 r) {
    float k0 = length(p / r);
    float k1 = length(p / (r * r));
    return k0 * (k0 - 1.0) / k1;
}

// Pirámide
float sdPyramid(float3 p, float h) {
    float m2 = h * h + 0.25;
    p.xz = abs(p.xz);
    p.xz = (p.z > p.x) ? p.zx : p.xz;
    p.xz -= 0.5;
    
    float3 q = float3(p.z, h * p.y - 0.5 * p.x, h * p.x + 0.5 * p.y);
    float s = max(-q.x, 0.0);
    float t = clamp((q.y - 0.5 * p.z) / (m2 + 0.25), 0.0, 1.0);
    
    float a = m2 * (q.x + s) * (q.x + s) + q.y * q.y;
    float b = m2 * (q.x + 0.5 * t) * (q.x + 0.5 * t) + (q.y - m2 * t) * (q.y - m2 * t);
    
    float d2 = min(q.y, -q.x * m2 - q.y * 0.5) > 0.0 ? 0.0 : min(a, b);
    return sqrt((d2 + q.z * q.z) / m2) * sign(max(q.z, -p.y));
}

// =============================================================================
// CURVAS BÉZIER 3D
// =============================================================================

// Punto en curva Bézier cuadrática
float3 bezierQuadratic(float3 P0, float3 P1, float3 P2, float t) {
    float u = 1.0 - t;
    return u * u * P0 + 2.0 * u * t * P1 + t * t * P2;
}

// Punto en curva Bézier cúbica
float3 bezierCubic(float3 P0, float3 P1, float3 P2, float3 P3, float t) {
    float u = 1.0 - t;
    float u2 = u * u;
    float u3 = u2 * u;
    float t2 = t * t;
    float t3 = t2 * t;
    return u3 * P0 + 3.0 * u2 * t * P1 + 3.0 * u * t2 * P2 + t3 * P3;
}

// Tangente de curva Bézier cúbica
float3 bezierCubicTangent(float3 P0, float3 P1, float3 P2, float3 P3, float t) {
    float u = 1.0 - t;
    return 3.0 * u * u * (P1 - P0) + 6.0 * u * t * (P2 - P1) + 3.0 * t * t * (P3 - P2);
}

// Distancia aproximada a curva Bézier
float sdBezierCubic(float3 p, float3 P0, float3 P1, float3 P2, float3 P3, float radius) {
    float minDist = 1e10;
    
    [unroll]
    for (int i = 0; i <= 16; i++) {
        float t = float(i) / 16.0;
        float3 curvePoint = bezierCubic(P0, P1, P2, P3, t);
        float dist = length(p - curvePoint);
        minDist = min(minDist, dist);
    }
    
    return minDist - radius;
}

// =============================================================================
// OPERACIONES CSG (Constructive Solid Geometry)
// =============================================================================

// Unión
float opUnion(float d1, float d2) {
    return min(d1, d2);
}

// Intersección
float opIntersection(float d1, float d2) {
    return max(d1, d2);
}

// Sustracción
float opSubtraction(float d1, float d2) {
    return max(d1, -d2);
}

// Unión suave
float opSmoothUnion(float d1, float d2, float k) {
    float h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return lerp(d2, d1, h) - k * h * (1.0 - h);
}

// Intersección suave
float opSmoothIntersection(float d1, float d2, float k) {
    float h = clamp(0.5 - 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return lerp(d2, d1, h) + k * h * (1.0 - h);
}

// Sustracción suave
float opSmoothSubtraction(float d1, float d2, float k) {
    float h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0.0, 1.0);
    return lerp(d1, -d2, h) + k * h * (1.0 - h);
}

// =============================================================================
// TRANSFORMACIONES
// =============================================================================

// Traslación
float3 opTranslate(float3 p, float3 offset) {
    return p - offset;
}

// Simetría
float3 opSymX(float3 p) { return float3(abs(p.x), p.y, p.z); }
float3 opSymY(float3 p) { return float3(p.x, abs(p.y), p.z); }
float3 opSymZ(float3 p) { return float3(p.x, p.y, abs(p.z)); }

// Repetición infinita
float3 opRepeat(float3 p, float3 spacing) {
    return fmod(abs(p) + spacing * 0.5, spacing) - spacing * 0.5;
}

// Repetición limitada
float3 opRepeatLimited(float3 p, float spacing, float3 limit) {
    return p - spacing * clamp(round(p / spacing), -limit, limit);
}

// Torsión
float3 opTwist(float3 p, float amount) {
    float c = cos(amount * p.y);
    float s = sin(amount * p.y);
    float2 xz = float2(c * p.x - s * p.z, s * p.x + c * p.z);
    return float3(xz.x, p.y, xz.y);
}

// Doblado
float3 opBend(float3 p, float amount) {
    float c = cos(amount * p.x);
    float s = sin(amount * p.x);
    float2 xy = float2(c * p.x - s * p.y, s * p.x + c * p.y);
    return float3(xy.x, xy.y, p.z);
}

// Redondeo
float opRound(float d, float radius) {
    return d - radius;
}

// Engrosamiento (crear cascarón)
float opOnion(float d, float thickness) {
    return abs(d) - thickness;
}

// =============================================================================
// RAY MARCHING
// =============================================================================

struct RayMarchResult {
    bool hit;
    float dist;
    float3 pos;
    float3 normal;
    int steps;
    float ao;  // Ambient occlusion aproximado
};

// Calcular normal en punto del SDF
float3 calcNormal(float3 p, float sdfValue) {
    float2 e = float2(0.0001, 0);
    // Nota: sceneSDF debe ser definido externamente
    return normalize(float3(
        sdfValue - sdSphere(p - e.xyy, 0), // Placeholder - reemplazar con sceneSDF real
        sdfValue - sdSphere(p - e.yxy, 0),
        sdfValue - sdSphere(p - e.yyx, 0)
    ));
}

// =============================================================================
// ILUMINACIÓN
// =============================================================================

// Luz direccional simple
float3 directionalLight(float3 normal, float3 lightDir, float3 lightColor) {
    float diff = max(0.0, dot(normal, normalize(lightDir)));
    return lightColor * diff;
}

// Luz especular (Blinn-Phong)
float3 specularLight(float3 normal, float3 lightDir, float3 viewDir, float3 lightColor, float shininess) {
    float3 halfVec = normalize(lightDir + viewDir);
    float spec = pow(max(0.0, dot(normal, halfVec)), shininess);
    return lightColor * spec;
}

// Ambient occlusion aproximado
float calcAO(float3 pos, float3 normal) {
    float occ = 0.0;
    float sca = 1.0;
    
    [unroll]
    for (int i = 0; i < 5; i++) {
        float h = 0.01 + 0.12 * float(i) / 4.0;
        // Nota: Requiere sceneSDF definido externamente
        float d = sdSphere(pos + h * normal, 1.0); // Placeholder
        occ += (h - d) * sca;
        sca *= 0.95;
    }
    
    return clamp(1.0 - 3.0 * occ, 0.0, 1.0);
}

// =============================================================================
// ANTI-ALIASING VECTORIAL
// =============================================================================

// Anti-aliasing perfecto basado en SDF
float vectorAA(float sdf) {
    float w = fwidth(sdf);
    return 1.0 - smoothstep(-w, w, sdf);
}

// Anti-aliasing con control de suavidad
float vectorAA_smooth(float sdf, float smoothness) {
    float w = fwidth(sdf) * smoothness;
    return 1.0 - smoothstep(-w, w, sdf);
}

#endif // ADEAD_VECTOR3D_RAYMARCHING_HLSL

