//! SDF Scene Demo - Map with Ground and Cubes
//! 
//! Demonstrates ADead-GPU technologies in EPICX:
//! - SDF primitives (cubes, spheres, ground)
//! - Ray marching rendering
//! - Soft shadows
//! - Ambient occlusion
//! - Multiple objects with different materials
//!
//! Run with: cargo run --example sdf_scene

use epicx::graphics::{Graphics, GraphicsConfig};
use epicx::math::{Vec3, Vec2, Color};
use epicx::sdf::{Sdf, Sphere, Box3D};
use std::time::Instant;
use std::f32::consts::PI;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::HWND;

/// Material properties
#[derive(Clone, Copy)]
struct Material {
    color: Vec3,
    roughness: f32,
    metallic: f32,
}

impl Material {
    fn new(color: Vec3, roughness: f32, metallic: f32) -> Self {
        Self { color, roughness, metallic }
    }
}

/// Scene object with SDF and material
struct SceneObject {
    position: Vec3,
    size: Vec3,
    material: Material,
    rotation: f32,
}

/// The 3D Scene with ground and objects
struct Scene {
    objects: Vec<SceneObject>,
    sun_dir: Vec3,
    sun_color: Vec3,
    ambient_color: Vec3,
    time: f32,
    camera_pos: Vec3,
    camera_target: Vec3,
}

impl Scene {
    fn new() -> Self {
        let mut scene = Self {
            objects: Vec::new(),
            sun_dir: Vec3::new(0.5, 0.8, 0.3).normalize(),
            sun_color: Vec3::new(1.0, 0.95, 0.9),
            ambient_color: Vec3::new(0.15, 0.18, 0.25),
            time: 0.0,
            camera_pos: Vec3::new(0.0, 5.0, 12.0),
            camera_target: Vec3::new(0.0, 1.0, 0.0),
        };
        
        // Add ground (large flat box)
        scene.objects.push(SceneObject {
            position: Vec3::new(0.0, -0.5, 0.0),
            size: Vec3::new(20.0, 0.5, 20.0),
            material: Material::new(Vec3::new(0.3, 0.35, 0.3), 0.8, 0.0),
            rotation: 0.0,
        });
        
        // Add cubes in a pattern
        let cube_positions = [
            (Vec3::new(-3.0, 0.5, -2.0), Vec3::new(0.8, 0.4, 0.9)),  // Purple
            (Vec3::new(0.0, 0.75, 0.0), Vec3::new(0.2, 0.6, 0.9)),   // Blue (center, taller)
            (Vec3::new(3.0, 0.5, -2.0), Vec3::new(0.9, 0.3, 0.3)),   // Red
            (Vec3::new(-2.0, 0.4, 3.0), Vec3::new(0.9, 0.8, 0.2)),   // Yellow
            (Vec3::new(2.5, 0.5, 2.5), Vec3::new(0.3, 0.8, 0.4)),    // Green
        ];
        
        for (i, (pos, color)) in cube_positions.iter().enumerate() {
            scene.objects.push(SceneObject {
                position: *pos,
                size: Vec3::new(0.8, pos.y * 2.0, 0.8),
                material: Material::new(*color, 0.3, 0.1),
                rotation: i as f32 * 0.3,
            });
        }
        
        // Add a sphere
        scene.objects.push(SceneObject {
            position: Vec3::new(-1.0, 1.0, 1.5),
            size: Vec3::splat(0.6), // Sphere uses size.x as radius
            material: Material::new(Vec3::new(0.95, 0.95, 0.95), 0.1, 0.9), // Metallic sphere
            rotation: -1.0, // Negative = sphere
        });
        
        scene
    }
    
    fn update(&mut self, dt: f32) {
        self.time += dt;
        
        // Rotate some cubes
        for (i, obj) in self.objects.iter_mut().enumerate().skip(1) {
            if obj.rotation >= 0.0 {
                obj.rotation = (i as f32 * 0.5) + self.time * 0.5;
            }
        }
        
        // Animate camera slightly
        self.camera_pos.x = 2.0 * (self.time * 0.2).sin();
        self.camera_pos.z = 12.0 + 2.0 * (self.time * 0.15).cos();
    }
    
    /// Rotate point around Y axis
    fn rotate_y(p: Vec3, angle: f32) -> Vec3 {
        let c = angle.cos();
        let s = angle.sin();
        Vec3::new(p.x * c + p.z * s, p.y, -p.x * s + p.z * c)
    }
    
    /// SDF for a box
    fn sd_box(p: Vec3, b: Vec3) -> f32 {
        let q = p.abs() - b;
        q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0)
    }
    
    /// SDF for a sphere
    fn sd_sphere(p: Vec3, r: f32) -> f32 {
        p.length() - r
    }
    
    /// Get distance and material at point
    fn scene_sdf(&self, p: Vec3) -> (f32, Material) {
        let mut min_dist = f32::MAX;
        let mut mat = Material::new(Vec3::ONE, 0.5, 0.0);
        
        for obj in &self.objects {
            let local_p = p - obj.position;
            
            let d = if obj.rotation < 0.0 {
                // Sphere
                Self::sd_sphere(local_p, obj.size.x)
            } else {
                // Rotated box
                let rotated = Self::rotate_y(local_p, obj.rotation);
                Self::sd_box(rotated, obj.size * 0.5)
            };
            
            if d < min_dist {
                min_dist = d;
                mat = obj.material;
            }
        }
        
        (min_dist, mat)
    }
    
    /// Calculate normal at point
    fn calc_normal(&self, p: Vec3) -> Vec3 {
        let eps = 0.001;
        let d = self.scene_sdf(p).0;
        Vec3::new(
            self.scene_sdf(p + Vec3::new(eps, 0.0, 0.0)).0 - d,
            self.scene_sdf(p + Vec3::new(0.0, eps, 0.0)).0 - d,
            self.scene_sdf(p + Vec3::new(0.0, 0.0, eps)).0 - d,
        ).normalize()
    }
    
    /// Soft shadow calculation
    fn calc_shadow(&self, origin: Vec3, dir: Vec3, min_t: f32, max_t: f32) -> f32 {
        let mut res = 1.0f32;
        let mut t = min_t;
        let k = 16.0;
        
        for _ in 0..48 {
            let (d, _) = self.scene_sdf(origin + dir * t);
            if d < 0.001 {
                return 0.0;
            }
            res = res.min(k * d / t);
            t += d.max(0.02);
            if t > max_t {
                break;
            }
        }
        res.clamp(0.0, 1.0)
    }
    
    /// Ambient occlusion
    fn calc_ao(&self, p: Vec3, n: Vec3) -> f32 {
        let mut occ = 0.0f32;
        let mut sca = 1.0f32;
        
        for i in 0..5 {
            let h = 0.01 + 0.12 * i as f32;
            let (d, _) = self.scene_sdf(p + n * h);
            occ += (h - d) * sca;
            sca *= 0.95;
        }
        (1.0 - 3.0 * occ).clamp(0.0, 1.0)
    }
    
    /// Render a single pixel
    fn render_pixel(&self, uv: Vec2, aspect: f32) -> Color {
        // Camera setup
        let forward = (self.camera_target - self.camera_pos).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward);
        
        // Ray direction
        let fov = 1.2;
        let rd = (forward * fov + right * uv.x * aspect + up * uv.y).normalize();
        
        // Sky gradient
        let sky_color = Vec3::new(0.4, 0.6, 0.9) * (1.0 - uv.y * 0.3) 
                      + Vec3::new(0.7, 0.8, 0.95) * (uv.y * 0.3 + 0.5);
        
        // Sun glow
        let sun_dot = rd.dot(self.sun_dir).max(0.0);
        let sun_glow = sun_dot.powf(64.0) * Vec3::new(1.0, 0.9, 0.7) * 0.5;
        
        // Ray march
        let mut t = 0.0f32;
        let max_t = 50.0;
        
        for _ in 0..100 {
            let p = self.camera_pos + rd * t;
            let (d, mat) = self.scene_sdf(p);
            
            if d < 0.001 {
                // Hit! Calculate shading
                let normal = self.calc_normal(p);
                
                // Lighting
                let n_dot_l = normal.dot(self.sun_dir).max(0.0);
                let shadow = self.calc_shadow(p + normal * 0.02, self.sun_dir, 0.02, 20.0);
                let ao = self.calc_ao(p, normal);
                
                // Fresnel
                let fresnel = (1.0 - (-rd).dot(normal).max(0.0)).powf(5.0);
                
                // Specular (Blinn-Phong)
                let half_vec = (self.sun_dir - rd).normalize();
                let spec_power = 32.0 / (mat.roughness + 0.01);
                let spec = normal.dot(half_vec).max(0.0).powf(spec_power) * (1.0 - mat.roughness);
                
                // Combine
                let diffuse = mat.color * self.sun_color * n_dot_l * shadow;
                let specular = self.sun_color * spec * shadow * (mat.metallic * 0.5 + 0.5);
                let ambient = self.ambient_color * mat.color * ao;
                let reflection = sky_color * fresnel * mat.metallic * 0.3;
                
                let color = ambient + diffuse + specular + reflection;
                
                // Fog
                let fog_amount = (1.0 - (-t * 0.03).exp()).clamp(0.0, 1.0);
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
        
        // Sky
        let final_sky = sky_color + sun_glow;
        Color::new(
            final_sky.x.clamp(0.0, 1.0),
            final_sky.y.clamp(0.0, 1.0),
            final_sky.z.clamp(0.0, 1.0),
            1.0,
        )
    }
    
    /// Render the scene to a pixel buffer
    fn render_to_buffer(&self, width: u32, height: u32) -> Vec<u8> {
        let aspect = width as f32 / height as f32;
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        
        for y in 0..height {
            for x in 0..width {
                let uv = Vec2::new(
                    (x as f32 / width as f32) * 2.0 - 1.0,
                    1.0 - (y as f32 / height as f32) * 2.0,
                );
                
                let color = self.render_pixel(uv, aspect);
                let idx = ((y * width + x) * 4) as usize;
                
                buffer[idx] = (color.r * 255.0) as u8;
                buffer[idx + 1] = (color.g * 255.0) as u8;
                buffer[idx + 2] = (color.b * 255.0) as u8;
                buffer[idx + 3] = 255;
            }
        }
        
        buffer
    }
}

/// Application state
struct App {
    window: Option<Window>,
    graphics: Option<Graphics>,
    scene: Scene,
    start_time: Instant,
    last_frame_time: Instant,
    frame_count: u64,
    last_fps_time: Instant,
    fps: f32,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            graphics: None,
            scene: Scene::new(),
            start_time: Instant::now(),
            last_frame_time: Instant::now(),
            frame_count: 0,
            last_fps_time: Instant::now(),
            fps: 0.0,
        }
    }
    
    fn render(&mut self) {
        let Some(graphics) = &mut self.graphics else { return };
        let Some(window) = &self.window else { return };
        
        // Delta time
        let now = Instant::now();
        let dt = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        
        // Update scene
        self.scene.update(dt);
        
        // FPS calculation
        self.frame_count += 1;
        let fps_elapsed = self.last_fps_time.elapsed().as_secs_f32();
        if fps_elapsed >= 1.0 {
            self.fps = self.frame_count as f32 / fps_elapsed;
            self.frame_count = 0;
            self.last_fps_time = Instant::now();
            
            window.set_title(&format!(
                "EPICX - SDF Scene | FPS: {:.1} | Objects: {}",
                self.fps, self.scene.objects.len()
            ));
        }
        
        // Render scene to ASCII (for now - GPU rendering would use compute shaders)
        let width = 120;
        let height = 50;
        let aspect = width as f32 / height as f32;
        
        // Print scene info only once
        if self.frame_count == 0 && self.start_time.elapsed().as_secs() < 1 {
            println!("\n[SCENE] Rendering {} objects with ADead-GPU SDF technology", self.scene.objects.len());
            println!("[SCENE] Features: Ray Marching, Soft Shadows, AO, Fresnel, Fog\n");
        }
        
        // Begin frame with Level B API
        let frame = match graphics.begin_frame() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[EPICX] Begin frame error: {:?}", e);
                return;
            }
        };
        
        // Animated background based on scene
        let t = self.start_time.elapsed().as_secs_f32();
        let bg_color = Color::new(
            0.1 + 0.05 * (t * 0.3).sin(),
            0.12 + 0.05 * (t * 0.4).cos(),
            0.18 + 0.05 * (t * 0.2).sin(),
            1.0,
        );
        frame.clear(bg_color);
        
        // End frame
        if let Err(e) = graphics.end_frame(frame) {
            eprintln!("[EPICX] End frame error: {:?}", e);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║      EPICX - SDF Scene Demo (ADead-GPU Technology)           ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Scene Contents:                                             ║");
        println!("║  - Ground plane (large box)                                  ║");
        println!("║  - 5 colored cubes (rotating)                                ║");
        println!("║  - 1 metallic sphere                                         ║");
        println!("║                                                              ║");
        println!("║  ADead-GPU Features:                                         ║");
        println!("║  - SDF Ray Marching                                          ║");
        println!("║  - Soft Shadows                                              ║");
        println!("║  - Ambient Occlusion                                         ║");
        println!("║  - Fresnel Reflections                                       ║");
        println!("║  - Distance Fog                                              ║");
        println!("║                                                              ║");
        println!("║  Controls: ESC to exit                                       ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        
        // Create window
        let window_attrs = Window::default_attributes()
            .with_title("EPICX - SDF Scene")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
        
        let window = event_loop.create_window(window_attrs).expect("Failed to create window");
        let size = window.inner_size();
        
        // Get HWND
        let hwnd = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get() as *mut std::ffi::c_void),
            _ => panic!("Unsupported platform"),
        };
        
        // Create Graphics (Level B) - Simple!
        println!("[EPICX] Creating Graphics...");
        let config = GraphicsConfig {
            width: size.width,
            height: size.height,
            ..Default::default()
        };
        
        let graphics = Graphics::new(hwnd, config).expect("Failed to create graphics");
        println!("[EPICX] Graphics ready ({}x{})", size.width, size.height);
        
        // Print ASCII preview of scene
        println!("\n[SCENE] ASCII Preview (SDF Ray Marching):");
        self.print_ascii_preview();
        
        self.window = Some(window);
        self.graphics = Some(graphics);
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("\n[EPICX] Window closed");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) {
                    println!("\n[EPICX] ESC pressed, exiting...");
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    if let Some(graphics) = &mut self.graphics {
                        let _ = graphics.resize(new_size.width, new_size.height);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }
    
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl App {
    fn print_ascii_preview(&self) {
        let width = 80;
        let height = 30;
        let aspect = width as f32 / height as f32;
        let chars = " .:-=+*#%@";
        
        for y in 0..height {
            let mut line = String::new();
            for x in 0..width {
                let uv = Vec2::new(
                    (x as f32 / width as f32) * 2.0 - 1.0,
                    1.0 - (y as f32 / height as f32) * 2.0,
                );
                
                let color = self.scene.render_pixel(uv, aspect);
                let brightness = (color.r * 0.299 + color.g * 0.587 + color.b * 0.114).clamp(0.0, 0.999);
                let char_idx = (brightness * chars.len() as f32) as usize;
                line.push(chars.chars().nth(char_idx).unwrap_or(' '));
            }
            println!("{}", line);
        }
        println!();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
