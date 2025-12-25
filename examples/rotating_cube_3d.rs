//! Rotating 3D Cube - Software Rendered
//! 
//! A simple rotating 3D cube with colored faces using software rasterization
//! displayed via softbuffer (like Vulkan Cube demo).
//!
//! Run with: cargo run --example rotating_cube_3d

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use softbuffer::{Context, Surface};

// ============================================================================
// 3D CUBE DATA
// ============================================================================

const CUBE_VERTICES: [[f32; 3]; 8] = [
    [-1.0, -1.0, -1.0], // 0
    [ 1.0, -1.0, -1.0], // 1
    [ 1.0,  1.0, -1.0], // 2
    [-1.0,  1.0, -1.0], // 3
    [-1.0, -1.0,  1.0], // 4
    [ 1.0, -1.0,  1.0], // 5
    [ 1.0,  1.0,  1.0], // 6
    [-1.0,  1.0,  1.0], // 7
];

const CUBE_FACES: [[usize; 4]; 6] = [
    [0, 1, 2, 3], // Back
    [5, 4, 7, 6], // Front
    [4, 0, 3, 7], // Left
    [1, 5, 6, 2], // Right
    [3, 2, 6, 7], // Top
    [4, 5, 1, 0], // Bottom
];

const FACE_COLORS: [[u8; 3]; 6] = [
    [220, 50, 50],   // Back - Red
    [50, 220, 50],   // Front - Green
    [50, 50, 220],   // Left - Blue
    [220, 220, 50],  // Right - Yellow
    [220, 50, 220],  // Top - Magenta
    [50, 220, 220],  // Bottom - Cyan
];

// ============================================================================
// SOFTWARE RASTERIZER
// ============================================================================

struct Renderer {
    width: u32,
    height: u32,
    color_buffer: Vec<u32>,
    depth_buffer: Vec<f32>,
    rotation_y: f32,
    rotation_x: f32,
}

impl Renderer {
    fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            color_buffer: vec![0xFF1a1a2e; size],
            depth_buffer: vec![f32::MAX; size],
            rotation_y: 0.0,
            rotation_x: 0.0,
        }
    }
    
    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let size = (width * height) as usize;
        self.color_buffer.resize(size, 0);
        self.depth_buffer.resize(size, f32::MAX);
    }
    
    fn clear(&mut self) {
        for y in 0..self.height {
            let t = y as f32 / self.height as f32;
            let r = lerp(0x1a, 0x0a, t);
            let g = lerp(0x1a, 0x0a, t);
            let b = lerp(0x2e, 0x1a, t);
            let color = rgb(r, g, b);
            
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                self.color_buffer[idx] = color;
                self.depth_buffer[idx] = f32::MAX;
            }
        }
    }
    
    fn update(&mut self, dt: f32) {
        self.rotation_y += dt * 1.2;
        self.rotation_x += dt * 0.8;
    }
    
    fn rotate_y(v: [f32; 3], angle: f32) -> [f32; 3] {
        let (s, c) = angle.sin_cos();
        [v[0] * c + v[2] * s, v[1], -v[0] * s + v[2] * c]
    }
    
    fn rotate_x(v: [f32; 3], angle: f32) -> [f32; 3] {
        let (s, c) = angle.sin_cos();
        [v[0], v[1] * c - v[2] * s, v[1] * s + v[2] * c]
    }
    
    fn project(&self, v: [f32; 3]) -> (f32, f32, f32) {
        let z = v[2] + 4.5;
        let fov = 2.0;
        let scale = fov / z.max(0.1);
        let aspect = self.width as f32 / self.height as f32;
        
        let x = v[0] * scale / aspect * self.width as f32 / 2.0 + self.width as f32 / 2.0;
        let y = -v[1] * scale * self.height as f32 / 2.0 + self.height as f32 / 2.0;
        
        (x, y, z)
    }
    
    fn draw_triangle(&mut self, v0: (f32, f32, f32), v1: (f32, f32, f32), v2: (f32, f32, f32), color: u32) {
        let min_x = v0.0.min(v1.0).min(v2.0).max(0.0) as i32;
        let max_x = v0.0.max(v1.0).max(v2.0).min(self.width as f32 - 1.0) as i32;
        let min_y = v0.1.min(v1.1).min(v2.1).max(0.0) as i32;
        let max_y = v0.1.max(v1.1).max(v2.1).min(self.height as f32 - 1.0) as i32;
        
        let area = edge(v0, v1, v2.0, v2.1);
        if area.abs() < 0.001 { return; }
        
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                
                let w0 = edge(v1, v2, px, py);
                let w1 = edge(v2, v0, px, py);
                let w2 = edge(v0, v1, px, py);
                
                if (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0) {
                    let z = (w0 * v0.2 + w1 * v1.2 + w2 * v2.2) / area;
                    
                    let idx = (y as u32 * self.width + x as u32) as usize;
                    if idx < self.depth_buffer.len() && z < self.depth_buffer[idx] {
                        self.depth_buffer[idx] = z;
                        self.color_buffer[idx] = color;
                    }
                }
            }
        }
    }
    
    fn render(&mut self) {
        self.clear();
        
        // Transform vertices
        let mut transformed: [[f32; 3]; 8] = [[0.0; 3]; 8];
        for (i, v) in CUBE_VERTICES.iter().enumerate() {
            transformed[i] = Self::rotate_x(Self::rotate_y(*v, self.rotation_y), self.rotation_x);
        }
        
        // Light direction
        let light = normalize([0.5, 0.8, 0.6]);
        
        // Sort faces by depth (painter's algorithm)
        let mut face_order: Vec<(usize, f32)> = (0..6).map(|i| {
            let face = &CUBE_FACES[i];
            let z = (transformed[face[0]][2] + transformed[face[1]][2] + 
                    transformed[face[2]][2] + transformed[face[3]][2]) / 4.0;
            (i, z)
        }).collect();
        face_order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Draw faces
        for (face_idx, _) in face_order {
            let face = &CUBE_FACES[face_idx];
            let base_color = FACE_COLORS[face_idx];
            
            // Calculate normal
            let v0 = transformed[face[0]];
            let v1 = transformed[face[1]];
            let v2 = transformed[face[2]];
            
            let e1 = sub(v1, v0);
            let e2 = sub(v2, v0);
            let normal = normalize(cross(e1, e2));
            
            // Back-face culling
            if normal[2] > 0.0 { continue; }
            
            // Lighting
            let ndotl = dot(normal, light).max(0.0);
            let intensity = 0.3 + 0.7 * ndotl;
            
            let r = (base_color[0] as f32 * intensity).min(255.0) as u8;
            let g = (base_color[1] as f32 * intensity).min(255.0) as u8;
            let b = (base_color[2] as f32 * intensity).min(255.0) as u8;
            let color = rgb(r, g, b);
            
            // Project and draw
            let p0 = self.project(transformed[face[0]]);
            let p1 = self.project(transformed[face[1]]);
            let p2 = self.project(transformed[face[2]]);
            let p3 = self.project(transformed[face[3]]);
            
            self.draw_triangle(p0, p1, p2, color);
            self.draw_triangle(p0, p2, p3, color);
        }
    }
    
    fn buffer(&self) -> &[u32] {
        &self.color_buffer
    }
}

// Helper functions
fn rgb(r: u8, g: u8, b: u8) -> u32 {
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 * (1.0 - t) + b as f32 * t) as u8
}

fn edge(a: (f32, f32, f32), b: (f32, f32, f32), px: f32, py: f32) -> f32 {
    (px - a.0) * (b.1 - a.1) - (py - a.1) * (b.0 - a.0)
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1]*b[2] - a[2]*b[1], a[2]*b[0] - a[0]*b[2], a[0]*b[1] - a[1]*b[0]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0]*b[0] + a[1]*b[1] + a[2]*b[2]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
    if len > 0.0 { [v[0]/len, v[1]/len, v[2]/len] } else { v }
}

// ============================================================================
// APPLICATION
// ============================================================================

struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    renderer: Renderer,
    last_frame: Instant,
    frame_count: u64,
    last_fps_time: Instant,
    fps: f32,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            surface: None,
            renderer: Renderer::new(800, 600),
            last_frame: Instant::now(),
            frame_count: 0,
            last_fps_time: Instant::now(),
            fps: 0.0,
        }
    }
    
    fn render(&mut self) {
        let Some(window) = &self.window else { return };
        let Some(surface) = &mut self.surface else { return };
        
        // Delta time
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        
        // Update and render
        self.renderer.update(dt);
        self.renderer.render();
        
        // FPS
        self.frame_count += 1;
        if self.last_fps_time.elapsed().as_secs_f32() >= 1.0 {
            self.fps = self.frame_count as f32;
            self.frame_count = 0;
            self.last_fps_time = Instant::now();
            
            window.set_title(&format!(
                "EPICX Cube | FPS: {:.0} | Rotation: {:.1}°",
                self.fps, self.renderer.rotation_y.to_degrees() % 360.0
            ));
        }
        
        // Display
        let size = window.inner_size();
        if size.width > 0 && size.height > 0 {
            let width = NonZeroU32::new(size.width).unwrap();
            let height = NonZeroU32::new(size.height).unwrap();
            
            surface.resize(width, height).expect("Failed to resize surface");
            
            let mut buffer = surface.buffer_mut().expect("Failed to get buffer");
            buffer.copy_from_slice(self.renderer.buffer());
            buffer.present().expect("Failed to present");
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!();
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║              EPICX - Rotating 3D Cube                        ║");
        println!("║                                                              ║");
        println!("║  A colorful rotating cube with software rasterization        ║");
        println!("║  Similar to Vulkan Cube demo                                 ║");
        println!("║                                                              ║");
        println!("║  Controls: ESC to exit                                       ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        
        let window_attrs = Window::default_attributes()
            .with_title("EPICX Cube")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
        
        let window = Rc::new(event_loop.create_window(window_attrs).expect("Failed to create window"));
        let size = window.inner_size();
        
        let context = Context::new(window.clone()).expect("Failed to create context");
        let surface = Surface::new(&context, window.clone()).expect("Failed to create surface");
        
        self.renderer.resize(size.width, size.height);
        
        println!("[INIT] Window created ({}x{})", size.width, size.height);
        println!("[INIT] Rendering cube...");
        println!();
        
        self.window = Some(window);
        self.surface = Some(surface);
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("\n[EXIT] Goodbye!");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    self.renderer.resize(new_size.width, new_size.height);
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
