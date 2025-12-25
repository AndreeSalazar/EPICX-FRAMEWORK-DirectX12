//! 3D Renderer - Real GPU rendering with primitives
//!
//! Provides actual 3D rendering using DirectX12 with:
//! - Vertex/Index buffers
//! - Shaders (HLSL)
//! - Primitive meshes (cube, sphere, cylinder, plane)
//! - Camera and transforms
//! - Basic lighting

use crate::math::{Vec3, Mat4, Color};

/// Vertex format for 3D rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex3D {
    pub fn new(pos: Vec3, normal: Vec3, color: Color) -> Self {
        Self {
            position: [pos.x, pos.y, pos.z],
            normal: [normal.x, normal.y, normal.z],
            color: [color.r, color.g, color.b, color.a],
        }
    }
}

/// Constant buffer for transforms
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TransformConstants {
    pub world: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub light_dir: [f32; 4],
    pub camera_pos: [f32; 4],
    pub ambient_color: [f32; 4],
    pub light_color: [f32; 4],
}

impl Default for TransformConstants {
    fn default() -> Self {
        Self {
            world: Mat4::IDENTITY.to_cols_array_2d(),
            view: Mat4::IDENTITY.to_cols_array_2d(),
            projection: Mat4::IDENTITY.to_cols_array_2d(),
            light_dir: [0.5, 0.8, 0.3, 0.0],
            camera_pos: [0.0, 5.0, 10.0, 1.0],
            ambient_color: [0.2, 0.25, 0.3, 1.0],
            light_color: [1.0, 0.95, 0.9, 1.0],
        }
    }
}

/// A 3D mesh with vertices and indices
pub struct Mesh3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
}

impl Mesh3D {
    /// Create a cube mesh
    pub fn cube(size: f32, color: Color) -> Self {
        let s = size * 0.5;
        
        // 8 corners of the cube
        let positions = [
            Vec3::new(-s, -s, -s), // 0: back-bottom-left
            Vec3::new( s, -s, -s), // 1: back-bottom-right
            Vec3::new( s,  s, -s), // 2: back-top-right
            Vec3::new(-s,  s, -s), // 3: back-top-left
            Vec3::new(-s, -s,  s), // 4: front-bottom-left
            Vec3::new( s, -s,  s), // 5: front-bottom-right
            Vec3::new( s,  s,  s), // 6: front-top-right
            Vec3::new(-s,  s,  s), // 7: front-top-left
        ];
        
        // Face normals
        let normals = [
            Vec3::new( 0.0,  0.0, -1.0), // back
            Vec3::new( 0.0,  0.0,  1.0), // front
            Vec3::new(-1.0,  0.0,  0.0), // left
            Vec3::new( 1.0,  0.0,  0.0), // right
            Vec3::new( 0.0, -1.0,  0.0), // bottom
            Vec3::new( 0.0,  1.0,  0.0), // top
        ];
        
        // Create vertices for each face (24 vertices total - 4 per face)
        let mut vertices = Vec::with_capacity(24);
        
        // Back face (0, 1, 2, 3)
        vertices.push(Vertex3D::new(positions[0], normals[0], color));
        vertices.push(Vertex3D::new(positions[1], normals[0], color));
        vertices.push(Vertex3D::new(positions[2], normals[0], color));
        vertices.push(Vertex3D::new(positions[3], normals[0], color));
        
        // Front face (4, 5, 6, 7)
        vertices.push(Vertex3D::new(positions[4], normals[1], color));
        vertices.push(Vertex3D::new(positions[5], normals[1], color));
        vertices.push(Vertex3D::new(positions[6], normals[1], color));
        vertices.push(Vertex3D::new(positions[7], normals[1], color));
        
        // Left face (0, 3, 7, 4)
        vertices.push(Vertex3D::new(positions[0], normals[2], color));
        vertices.push(Vertex3D::new(positions[3], normals[2], color));
        vertices.push(Vertex3D::new(positions[7], normals[2], color));
        vertices.push(Vertex3D::new(positions[4], normals[2], color));
        
        // Right face (1, 5, 6, 2)
        vertices.push(Vertex3D::new(positions[1], normals[3], color));
        vertices.push(Vertex3D::new(positions[5], normals[3], color));
        vertices.push(Vertex3D::new(positions[6], normals[3], color));
        vertices.push(Vertex3D::new(positions[2], normals[3], color));
        
        // Bottom face (0, 4, 5, 1)
        vertices.push(Vertex3D::new(positions[0], normals[4], color));
        vertices.push(Vertex3D::new(positions[4], normals[4], color));
        vertices.push(Vertex3D::new(positions[5], normals[4], color));
        vertices.push(Vertex3D::new(positions[1], normals[4], color));
        
        // Top face (3, 2, 6, 7)
        vertices.push(Vertex3D::new(positions[3], normals[5], color));
        vertices.push(Vertex3D::new(positions[2], normals[5], color));
        vertices.push(Vertex3D::new(positions[6], normals[5], color));
        vertices.push(Vertex3D::new(positions[7], normals[5], color));
        
        // Indices for 6 faces (2 triangles per face)
        let indices = vec![
            // Back
            0, 2, 1, 0, 3, 2,
            // Front
            4, 5, 6, 4, 6, 7,
            // Left
            8, 9, 10, 8, 10, 11,
            // Right
            12, 14, 13, 12, 15, 14,
            // Bottom
            16, 17, 18, 16, 18, 19,
            // Top
            20, 22, 21, 20, 23, 22,
        ];
        
        Self { vertices, indices }
    }
    
    /// Create a plane mesh
    pub fn plane(width: f32, depth: f32, color: Color) -> Self {
        let hw = width * 0.5;
        let hd = depth * 0.5;
        let normal = Vec3::Y;
        
        let vertices = vec![
            Vertex3D::new(Vec3::new(-hw, 0.0, -hd), normal, color),
            Vertex3D::new(Vec3::new( hw, 0.0, -hd), normal, color),
            Vertex3D::new(Vec3::new( hw, 0.0,  hd), normal, color),
            Vertex3D::new(Vec3::new(-hw, 0.0,  hd), normal, color),
        ];
        
        let indices = vec![0, 2, 1, 0, 3, 2];
        
        Self { vertices, indices }
    }
    
    /// Create a cylinder mesh
    pub fn cylinder(radius: f32, height: f32, segments: u32, color: Color) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let half_height = height * 0.5;
        
        // Generate vertices for the sides
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let normal = Vec3::new(angle.cos(), 0.0, angle.sin());
            
            // Bottom vertex
            vertices.push(Vertex3D::new(Vec3::new(x, -half_height, z), normal, color));
            // Top vertex
            vertices.push(Vertex3D::new(Vec3::new(x, half_height, z), normal, color));
        }
        
        // Generate indices for the sides
        for i in 0..segments {
            let base = i * 2;
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 3);
            indices.push(base);
            indices.push(base + 3);
            indices.push(base + 2);
        }
        
        // Top cap center
        let top_center_idx = vertices.len() as u32;
        vertices.push(Vertex3D::new(Vec3::new(0.0, half_height, 0.0), Vec3::Y, color));
        
        // Bottom cap center
        let bottom_center_idx = vertices.len() as u32;
        vertices.push(Vertex3D::new(Vec3::new(0.0, -half_height, 0.0), -Vec3::Y, color));
        
        // Top cap vertices
        let top_start = vertices.len() as u32;
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            vertices.push(Vertex3D::new(Vec3::new(x, half_height, z), Vec3::Y, color));
        }
        
        // Bottom cap vertices
        let bottom_start = vertices.len() as u32;
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            vertices.push(Vertex3D::new(Vec3::new(x, -half_height, z), -Vec3::Y, color));
        }
        
        // Top cap indices
        for i in 0..segments {
            indices.push(top_center_idx);
            indices.push(top_start + i);
            indices.push(top_start + i + 1);
        }
        
        // Bottom cap indices
        for i in 0..segments {
            indices.push(bottom_center_idx);
            indices.push(bottom_start + i + 1);
            indices.push(bottom_start + i);
        }
        
        Self { vertices, indices }
    }
    
    /// Create a sphere mesh
    pub fn sphere(radius: f32, rings: u32, segments: u32, color: Color) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Generate vertices
        for ring in 0..=rings {
            let phi = (ring as f32 / rings as f32) * std::f32::consts::PI;
            let y = phi.cos() * radius;
            let ring_radius = phi.sin() * radius;
            
            for seg in 0..=segments {
                let theta = (seg as f32 / segments as f32) * std::f32::consts::TAU;
                let x = theta.cos() * ring_radius;
                let z = theta.sin() * ring_radius;
                
                let pos = Vec3::new(x, y, z);
                let normal = pos.normalize();
                
                vertices.push(Vertex3D::new(pos, normal, color));
            }
        }
        
        // Generate indices
        for ring in 0..rings {
            for seg in 0..segments {
                let current = ring * (segments + 1) + seg;
                let next = current + segments + 1;
                
                indices.push(current);
                indices.push(next);
                indices.push(current + 1);
                
                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }
        
        Self { vertices, indices }
    }
    
    /// Create a pyramid mesh
    pub fn pyramid(base_size: f32, height: f32, color: Color) -> Self {
        let s = base_size * 0.5;
        let h = height;
        
        // Apex and base corners
        let apex = Vec3::new(0.0, h, 0.0);
        let bl = Vec3::new(-s, 0.0, -s);
        let br = Vec3::new( s, 0.0, -s);
        let fr = Vec3::new( s, 0.0,  s);
        let fl = Vec3::new(-s, 0.0,  s);
        
        let mut vertices = Vec::new();
        
        // Front face
        let n_front = (fr - apex).cross(fl - apex).normalize();
        vertices.push(Vertex3D::new(apex, n_front, color));
        vertices.push(Vertex3D::new(fl, n_front, color));
        vertices.push(Vertex3D::new(fr, n_front, color));
        
        // Right face
        let n_right = (br - apex).cross(fr - apex).normalize();
        vertices.push(Vertex3D::new(apex, n_right, color));
        vertices.push(Vertex3D::new(fr, n_right, color));
        vertices.push(Vertex3D::new(br, n_right, color));
        
        // Back face
        let n_back = (bl - apex).cross(br - apex).normalize();
        vertices.push(Vertex3D::new(apex, n_back, color));
        vertices.push(Vertex3D::new(br, n_back, color));
        vertices.push(Vertex3D::new(bl, n_back, color));
        
        // Left face
        let n_left = (fl - apex).cross(bl - apex).normalize();
        vertices.push(Vertex3D::new(apex, n_left, color));
        vertices.push(Vertex3D::new(bl, n_left, color));
        vertices.push(Vertex3D::new(fl, n_left, color));
        
        // Base
        let n_base = -Vec3::Y;
        vertices.push(Vertex3D::new(bl, n_base, color));
        vertices.push(Vertex3D::new(br, n_base, color));
        vertices.push(Vertex3D::new(fr, n_base, color));
        vertices.push(Vertex3D::new(fl, n_base, color));
        
        let indices = vec![
            0, 1, 2,     // Front
            3, 4, 5,     // Right
            6, 7, 8,     // Back
            9, 10, 11,   // Left
            12, 13, 14, 12, 14, 15, // Base
        ];
        
        Self { vertices, indices }
    }
}

/// Camera for 3D rendering
pub struct Camera3D {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera3D {
    pub fn new(position: Vec3, target: Vec3, aspect: f32) -> Self {
        Self {
            position,
            target,
            up: Vec3::Y,
            fov: 60.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 100.0,
        }
    }
    
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }
    
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }
}

/// Transform for 3D objects
#[derive(Clone, Copy, Debug)]
pub struct Transform3D {
    pub position: Vec3,
    pub rotation: Vec3, // Euler angles in radians
    pub scale: Vec3,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}

impl Transform3D {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }
    
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }
    
    pub fn with_rotation(mut self, rotation: Vec3) -> Self {
        self.rotation = rotation;
        self
    }
    
    pub fn matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(self.position);
        let rotation = Mat4::from_euler(glam::EulerRot::YXZ, 
            self.rotation.y, self.rotation.x, self.rotation.z);
        let scale = Mat4::from_scale(self.scale);
        translation * rotation * scale
    }
}

/// A renderable 3D object
pub struct Object3D {
    pub mesh: Mesh3D,
    pub transform: Transform3D,
}

impl Object3D {
    pub fn new(mesh: Mesh3D, transform: Transform3D) -> Self {
        Self { mesh, transform }
    }
    
    pub fn cube(size: f32, color: Color, position: Vec3) -> Self {
        Self::new(
            Mesh3D::cube(size, color),
            Transform3D::new(position),
        )
    }
    
    pub fn sphere(radius: f32, color: Color, position: Vec3) -> Self {
        Self::new(
            Mesh3D::sphere(radius, 16, 32, color),
            Transform3D::new(position),
        )
    }
    
    pub fn cylinder(radius: f32, height: f32, color: Color, position: Vec3) -> Self {
        Self::new(
            Mesh3D::cylinder(radius, height, 24, color),
            Transform3D::new(position),
        )
    }
    
    pub fn plane(width: f32, depth: f32, color: Color, position: Vec3) -> Self {
        Self::new(
            Mesh3D::plane(width, depth, color),
            Transform3D::new(position),
        )
    }
    
    pub fn pyramid(base: f32, height: f32, color: Color, position: Vec3) -> Self {
        Self::new(
            Mesh3D::pyramid(base, height, color),
            Transform3D::new(position),
        )
    }
}

/// HLSL Shader source for 3D rendering
pub mod shaders {
    pub const VERTEX_SHADER_3D: &str = r#"
cbuffer TransformConstants : register(b0)
{
    float4x4 World;
    float4x4 View;
    float4x4 Projection;
    float4 LightDir;
    float4 CameraPos;
    float4 AmbientColor;
    float4 LightColor;
};

struct VSInput
{
    float3 Position : POSITION;
    float3 Normal : NORMAL;
    float4 Color : COLOR;
};

struct PSInput
{
    float4 Position : SV_POSITION;
    float3 WorldPos : TEXCOORD0;
    float3 Normal : TEXCOORD1;
    float4 Color : COLOR;
};

PSInput VSMain(VSInput input)
{
    PSInput output;
    
    float4 worldPos = mul(float4(input.Position, 1.0), World);
    float4 viewPos = mul(worldPos, View);
    output.Position = mul(viewPos, Projection);
    
    output.WorldPos = worldPos.xyz;
    output.Normal = normalize(mul(float4(input.Normal, 0.0), World).xyz);
    output.Color = input.Color;
    
    return output;
}
"#;

    pub const PIXEL_SHADER_3D: &str = r#"
cbuffer TransformConstants : register(b0)
{
    float4x4 World;
    float4x4 View;
    float4x4 Projection;
    float4 LightDir;
    float4 CameraPos;
    float4 AmbientColor;
    float4 LightColor;
};

struct PSInput
{
    float4 Position : SV_POSITION;
    float3 WorldPos : TEXCOORD0;
    float3 Normal : TEXCOORD1;
    float4 Color : COLOR;
};

float4 PSMain(PSInput input) : SV_TARGET
{
    float3 normal = normalize(input.Normal);
    float3 lightDir = normalize(LightDir.xyz);
    
    // Diffuse lighting
    float NdotL = max(dot(normal, lightDir), 0.0);
    float3 diffuse = input.Color.rgb * LightColor.rgb * NdotL;
    
    // Ambient
    float3 ambient = input.Color.rgb * AmbientColor.rgb;
    
    // Specular (Blinn-Phong)
    float3 viewDir = normalize(CameraPos.xyz - input.WorldPos);
    float3 halfVec = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfVec), 0.0), 32.0);
    float3 specular = LightColor.rgb * spec * 0.3;
    
    float3 finalColor = ambient + diffuse + specular;
    
    return float4(finalColor, input.Color.a);
}
"#;
}
