//! Rotating Cube with Ground and Sun
//! 
//! Demonstrates EPICX capabilities:
//! - SDF rendering (cube, plane)
//! - Ray marching
//! - Basic lighting (sun)
//! - Real-time rotation
//!
//! Run with: cargo run --example rotating_cube

use epicx::sdf::{Sdf, Sphere, Box3D, Plane, ray_march, RayMarchConfig};
use epicx::math::{Vec3, Vec2, Color};
use std::f32::consts::PI;
use std::time::Instant;

/// Scene containing all SDF objects
struct Scene {
    /// Cube rotation angle
    rotation: f32,
    /// Sun direction (normalized)
    sun_dir: Vec3,
}

impl Scene {
    fn new() -> Self {
        Self {
            rotation: 0.0,
            sun_dir: Vec3::new(0.5, 0.8, 0.3).normalize(),
        }
    }
    
    /// Rotate a point around Y axis
    fn rotate_y(&self, p: Vec3, angle: f32) -> Vec3 {
        let c = angle.cos();
        let s = angle.sin();
        Vec3::new(
            p.x * c + p.z * s,
            p.y,
            -p.x * s + p.z * c,
        )
    }
    
    /// SDF for the rotating cube
    fn cube_sdf(&self, p: Vec3) -> f32 {
        // Translate cube up and rotate
        let local = self.rotate_y(p - Vec3::new(0.0, 1.0, 0.0), self.rotation);
        
        // Box SDF
        let b = Vec3::splat(0.8);
        let q = local.abs() - b;
        q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0)
    }
    
    /// SDF for the ground plane
    fn ground_sdf(&self, p: Vec3) -> f32 {
        p.y // Ground at y=0
    }
    
    /// SDF for the sun (visual sphere in the sky)
    fn sun_sdf(&self, p: Vec3) -> f32 {
        let sun_pos = self.sun_dir * 50.0;
        (p - sun_pos).length() - 3.0
    }
    
    /// Combined scene SDF
    fn scene_sdf(&self, p: Vec3) -> f32 {
        let cube = self.cube_sdf(p);
        let ground = self.ground_sdf(p);
        cube.min(ground)
    }
    
    /// Calculate normal using gradient
    fn calc_normal(&self, p: Vec3) -> Vec3 {
        let eps = 0.001;
        let d = self.scene_sdf(p);
        Vec3::new(
            self.scene_sdf(p + Vec3::new(eps, 0.0, 0.0)) - d,
            self.scene_sdf(p + Vec3::new(0.0, eps, 0.0)) - d,
            self.scene_sdf(p + Vec3::new(0.0, 0.0, eps)) - d,
        ).normalize()
    }
    
    /// Calculate soft shadow
    fn calc_shadow(&self, origin: Vec3, dir: Vec3, min_t: f32, max_t: f32) -> f32 {
        let mut res = 1.0f32;
        let mut t = min_t;
        let k = 16.0; // Shadow softness
        
        for _ in 0..32 {
            let d = self.scene_sdf(origin + dir * t);
            if d < 0.001 {
                return 0.0;
            }
            res = res.min(k * d / t);
            t += d;
            if t > max_t {
                break;
            }
        }
        res.clamp(0.0, 1.0)
    }
    
    /// Calculate ambient occlusion
    fn calc_ao(&self, p: Vec3, n: Vec3) -> f32 {
        let mut occ = 0.0f32;
        let mut sca = 1.0f32;
        
        for i in 0..5 {
            let h = 0.01 + 0.12 * i as f32;
            let d = self.scene_sdf(p + n * h);
            occ += (h - d) * sca;
            sca *= 0.95;
        }
        (1.0 - 3.0 * occ).clamp(0.0, 1.0)
    }
    
    /// Get material color based on which object was hit
    fn get_material(&self, p: Vec3) -> (Vec3, f32) {
        let cube_d = self.cube_sdf(p);
        let ground_d = self.ground_sdf(p);
        
        if cube_d < ground_d {
            // Cube: blue-ish material
            (Vec3::new(0.2, 0.4, 0.8), 0.3)
        } else {
            // Ground: checkerboard pattern
            let checker = ((p.x * 2.0).floor() + (p.z * 2.0).floor()) as i32 % 2;
            if checker == 0 {
                (Vec3::new(0.4, 0.4, 0.4), 0.1)
            } else {
                (Vec3::new(0.6, 0.6, 0.6), 0.1)
            }
        }
    }
    
    /// Render a single pixel
    fn render_pixel(&self, uv: Vec2, aspect: f32) -> Color {
        // Camera setup
        let camera_pos = Vec3::new(0.0, 3.0, 8.0);
        let look_at = Vec3::new(0.0, 1.0, 0.0);
        
        // Camera basis
        let forward = (look_at - camera_pos).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward);
        
        // Ray direction
        let fov = 1.0;
        let rd = (forward * fov + right * uv.x * aspect + up * uv.y).normalize();
        
        // Sky color (gradient)
        let sky_color = Vec3::new(0.5, 0.7, 1.0) * (1.0 - uv.y * 0.5) + Vec3::new(0.8, 0.9, 1.0) * (uv.y * 0.5 + 0.5);
        
        // Sun glow
        let sun_dot = rd.dot(self.sun_dir).max(0.0);
        let sun_glow = sun_dot.powf(64.0) * Vec3::new(1.0, 0.9, 0.7);
        
        // Ray march
        let mut t = 0.0f32;
        let max_t = 100.0;
        
        for _ in 0..128 {
            let p = camera_pos + rd * t;
            let d = self.scene_sdf(p);
            
            if d < 0.001 {
                // Hit! Calculate shading
                let normal = self.calc_normal(p);
                let (albedo, roughness) = self.get_material(p);
                
                // Diffuse lighting
                let n_dot_l = normal.dot(self.sun_dir).max(0.0);
                
                // Shadow
                let shadow = self.calc_shadow(p + normal * 0.01, self.sun_dir, 0.01, 20.0);
                
                // Ambient occlusion
                let ao = self.calc_ao(p, normal);
                
                // Specular (Blinn-Phong)
                let half_vec = (self.sun_dir - rd).normalize();
                let spec = normal.dot(half_vec).max(0.0).powf(32.0 / (roughness + 0.01)) * (1.0 - roughness);
                
                // Combine lighting
                let sun_color = Vec3::new(1.0, 0.95, 0.9);
                let ambient = Vec3::new(0.2, 0.25, 0.3) * ao;
                
                let diffuse = albedo * sun_color * n_dot_l * shadow;
                let specular = sun_color * spec * shadow;
                
                let color = ambient * albedo + diffuse + specular;
                
                // Fog
                let fog_amount = (1.0 - (-t * 0.02).exp()).clamp(0.0, 1.0);
                let final_color = color * (1.0 - fog_amount) + sky_color * fog_amount;
                
                return Color::new(
                    final_color.x.clamp(0.0, 1.0),
                    final_color.y.clamp(0.0, 1.0),
                    final_color.z.clamp(0.0, 1.0),
                    1.0,
                );
            }
            
            t += d;
            if t > max_t {
                break;
            }
        }
        
        // Sky with sun
        let final_sky = sky_color + sun_glow;
        Color::new(
            final_sky.x.clamp(0.0, 1.0),
            final_sky.y.clamp(0.0, 1.0),
            final_sky.z.clamp(0.0, 1.0),
            1.0,
        )
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         EPICX - Rotating Cube Demo (SDF Ray Marching)        ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Features:                                                   ║");
    println!("║  - SDF Ray Marching                                          ║");
    println!("║  - Rotating cube with soft shadows                           ║");
    println!("║  - Checkerboard ground plane                                 ║");
    println!("║  - Sun lighting with glow                                    ║");
    println!("║  - Ambient occlusion                                         ║");
    println!("║  - Fog effect                                                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    
    // Render settings
    let width = 80;
    let height = 40;
    let aspect = width as f32 / height as f32;
    
    // ASCII brightness characters
    let chars = " .:-=+*#%@";
    
    let start = Instant::now();
    let mut scene = Scene::new();
    
    // Animate for a few frames
    for frame in 0..10 {
        scene.rotation = frame as f32 * PI / 5.0;
        
        println!("\n┌─ Frame {} (rotation: {:.1}°) ─┐", frame, scene.rotation.to_degrees());
        
        // Render frame
        let mut output = String::new();
        
        for y in 0..height {
            for x in 0..width {
                // Normalized coordinates (-1 to 1)
                let uv = Vec2::new(
                    (x as f32 / width as f32) * 2.0 - 1.0,
                    1.0 - (y as f32 / height as f32) * 2.0,
                );
                
                let color = scene.render_pixel(uv, aspect);
                
                // Convert to brightness
                let brightness = (color.r * 0.299 + color.g * 0.587 + color.b * 0.114).clamp(0.0, 0.999);
                let char_idx = (brightness * chars.len() as f32) as usize;
                output.push(chars.chars().nth(char_idx).unwrap_or(' '));
            }
            output.push('\n');
        }
        
        println!("{}", output);
        
        // Small delay between frames
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    let elapsed = start.elapsed();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  Rendering complete!                                         ║");
    println!("║  Total time: {:?}                                   ║", elapsed);
    println!("║  Resolution: {}x{} ASCII                                   ║", width, height);
    println!("║  Frames: 10                                                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("This demo uses pure SDF mathematics - no meshes, no textures!");
    println!("The same techniques can render at any resolution with perfect quality.");
}
