// =============================================================================
// ADead-GPU: Vector3D Demo - Ray Marching Scene
// Pure Mathematics Rendering - No Triangles!
// =============================================================================

#include "vector3d_raymarching.hlsl"

// =============================================================================
// CONSTANT BUFFER
// =============================================================================

cbuffer CB : register(b0) {
    row_major float4x4 invViewProj;
    float3 cameraPos;
    float time;
    float2 resolution;
    float2 _padding;
};

// =============================================================================
// ESCENA VECTORIAL (100% Matemáticas, 0% Triángulos)
// =============================================================================

float sceneSDF(float3 p) {
    // Piso
    float floor = sdPlane(p, float3(0, 1, 0), 0);
    
    // Esfera principal (animada)
    float3 spherePos = float3(0, 1.0 + sin(time) * 0.3, 0);
    float sphere = sdSphere(p - spherePos, 0.5);
    
    // Cubo rotando
    float3 cubePos = float3(2.0, 0.5, 0);
    float c = cos(time * 0.5);
    float s = sin(time * 0.5);
    float3 rotP = p - cubePos;
    rotP.xz = float2(c * rotP.x - s * rotP.z, s * rotP.x + c * rotP.z);
    float cube = sdBox(rotP, float3(0.4, 0.4, 0.4));
    
    // Toro (donut)
    float3 torusPos = float3(-2.0, 0.5, 0);
    float torus = sdTorus(p - torusPos, float2(0.4, 0.15));
    
    // Cápsula diagonal
    float capsule = sdCapsule(p, 
        float3(0, 0.2, 2), 
        float3(1, 1.2, 2), 
        0.15);
    
    // Cilindro
    float cylinder = sdCylinder(p - float3(-1, 0.5, -2), 0.5, 0.25);
    
    // Pirámide
    float pyramid = sdPyramid(p - float3(1.5, 0, -2), 1.0);
    
    // Combinación CSG: Esfera - Cubo pequeño (muestra sustracción)
    float3 csgPos = float3(0, 0.5, -1);
    float csgSphere = sdSphere(p - csgPos, 0.4);
    float csgCube = sdBox(p - csgPos, float3(0.25, 0.25, 0.25));
    float csgResult = opSmoothSubtraction(csgCube, csgSphere, 0.05);
    
    // Esferas repetidas (demostración de repetición)
    float3 repP = opRepeatLimited(p - float3(0, 0.3, 4), 1.0, float3(3, 0, 0));
    float repeatedSpheres = sdSphere(repP, 0.2);
    
    // Forma retorcida
    float3 twistP = opTwist(p - float3(3, 1, 2), 2.0);
    float twisted = sdBox(twistP, float3(0.2, 0.8, 0.2));
    
    // Combinar todo con unión
    float scene = floor;
    scene = opUnion(scene, sphere);
    scene = opUnion(scene, cube);
    scene = opUnion(scene, torus);
    scene = opUnion(scene, capsule);
    scene = opUnion(scene, cylinder);
    scene = opUnion(scene, pyramid);
    scene = opUnion(scene, csgResult);
    scene = opUnion(scene, repeatedSpheres);
    scene = opUnion(scene, twisted);
    
    return scene;
}

// Normal de la escena
float3 sceneNormal(float3 p) {
    float2 e = float2(0.001, 0);
    return normalize(float3(
        sceneSDF(p + e.xyy) - sceneSDF(p - e.xyy),
        sceneSDF(p + e.yxy) - sceneSDF(p - e.yxy),
        sceneSDF(p + e.yyx) - sceneSDF(p - e.yyx)
    ));
}

// Material basado en posición
float3 getMaterial(float3 p, float3 n) {
    // Piso con patrón de tablero
    if (p.y < 0.01) {
        float checker = fmod(floor(p.x) + floor(p.z), 2.0);
        return lerp(float3(0.3, 0.3, 0.3), float3(0.6, 0.6, 0.6), checker);
    }
    
    // Color basado en normal (como visualización de debug)
    return n * 0.5 + 0.5;
}

// =============================================================================
// RAY MARCHING PRINCIPAL
// =============================================================================

RayMarchResult rayMarch(float3 ro, float3 rd) {
    RayMarchResult result;
    result.hit = false;
    result.dist = 0;
    result.steps = 0;
    result.ao = 1.0;
    
    float t = 0;
    
    [loop]
    for (int i = 0; i < MAX_STEPS; i++) {
        float3 p = ro + rd * t;
        float d = sceneSDF(p);
        
        if (d < SURF_DIST) {
            result.hit = true;
            result.dist = t;
            result.pos = p;
            result.normal = sceneNormal(p);
            result.steps = i;
            result.ao = 1.0 - float(i) / float(MAX_STEPS); // AO simple
            return result;
        }
        
        if (t > MAX_DIST) break;
        
        t += d;
        result.steps = i;
    }
    
    result.dist = MAX_DIST;
    return result;
}

// =============================================================================
// ILUMINACIÓN
// =============================================================================

float3 shade(RayMarchResult hit, float3 rd) {
    if (!hit.hit) {
        // Cielo con gradiente
        float t = 0.5 * (rd.y + 1.0);
        return lerp(float3(0.5, 0.7, 1.0), float3(0.1, 0.2, 0.4), t);
    }
    
    // Direcciones de luz
    float3 lightDir = normalize(float3(1, 2, -1));
    float3 lightColor = float3(1.0, 0.95, 0.9);
    
    // Material
    float3 albedo = getMaterial(hit.pos, hit.normal);
    
    // Difuso
    float diff = max(0.0, dot(hit.normal, lightDir));
    
    // Especular
    float3 viewDir = -rd;
    float3 halfVec = normalize(lightDir + viewDir);
    float spec = pow(max(0.0, dot(hit.normal, halfVec)), 32.0);
    
    // Sombras suaves (ray march hacia la luz)
    float shadow = 1.0;
    float3 shadowOrigin = hit.pos + hit.normal * 0.02;
    float t = 0.02;
    
    [loop]
    for (int i = 0; i < 32; i++) {
        float3 p = shadowOrigin + lightDir * t;
        float d = sceneSDF(p);
        
        if (d < 0.001) {
            shadow = 0.0;
            break;
        }
        
        shadow = min(shadow, 16.0 * d / t);
        t += d;
        
        if (t > 10.0) break;
    }
    shadow = clamp(shadow, 0.0, 1.0);
    
    // Ambient
    float3 ambient = float3(0.1, 0.15, 0.2) * hit.ao;
    
    // Combinar
    float3 color = albedo * (ambient + lightColor * diff * shadow);
    color += lightColor * spec * shadow * 0.5;
    
    // Fog
    float fog = 1.0 - exp(-hit.dist * 0.05);
    float3 fogColor = float3(0.5, 0.6, 0.7);
    color = lerp(color, fogColor, fog);
    
    return color;
}

// =============================================================================
// VERTEX / PIXEL SHADERS
// =============================================================================

struct VSInput {
    float3 pos : POSITION;
    float2 uv : TEXCOORD0;
};

struct PSInput {
    float4 pos : SV_POSITION;
    float2 uv : TEXCOORD0;
};

PSInput VSMain(VSInput input) {
    PSInput output;
    output.pos = float4(input.pos, 1.0);
    output.uv = input.uv;
    return output;
}

float4 PSMain(PSInput input) : SV_TARGET {
    // Coordenadas normalizadas [-1, 1]
    float2 uv = input.uv * 2.0 - 1.0;
    uv.x *= resolution.x / resolution.y;
    
    // Construir rayo
    float3 ro = cameraPos;
    float3 rd = normalize(float3(uv.x, uv.y, 2.0));  // FOV ~53°
    
    // Rotar dirección del rayo con la cámara (simplificado)
    // En producción: usar matriz invViewProj
    
    // Ray march
    RayMarchResult hit = rayMarch(ro, rd);
    
    // Sombrear
    float3 color = shade(hit, rd);
    
    // Anti-aliasing basado en SDF (en los bordes)
    if (hit.hit) {
        float sdf = sceneSDF(hit.pos);
        float aa = vectorAA(sdf);
        // Suavizar bordes de la silueta
    }
    
    // Tone mapping simple
    color = color / (color + 1.0);
    
    // Gamma
    color = pow(color, 1.0 / 2.2);
    
    return float4(color, 1.0);
}

