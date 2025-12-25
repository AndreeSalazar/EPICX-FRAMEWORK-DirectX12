//! Vulkan Cube Style - Simple Rotating Cube
//! 
//! A simple rotating cube demo similar to vkcube from Vulkan SDK.
//! Features:
//! - Automatic GPU detection
//! - Simple 3D cube with colored faces
//! - Smooth rotation
//! - DirectX12 rendering
//!
//! Run with: cargo run --example vulkan_cube

use epicx::dx12::{detect_gpu, Device, CommandQueue, SwapChain, SwapChainConfig, CommandAllocator, CommandList};
use epicx::math::{Vec3, Vec2, Color};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D12::*;

// ============================================================================
// SIMPLE 3D CUBE RENDERER (SDF Ray Marching)
// ============================================================================

struct CubeRenderer {
    rotation: f32,
    camera_distance: f32,
}

impl CubeRenderer {
    fn new() -> Self {
        Self {
            rotation: 0.0,
            camera_distance: 4.0,
        }
    }
    
    fn update(&mut self, dt: f32) {
        self.rotation += dt * 1.0; // Rotate 1 radian per second
    }
    
    /// Rotate point around Y axis
    fn rotate_y(p: Vec3, angle: f32) -> Vec3 {
        let c = angle.cos();
        let s = angle.sin();
        Vec3::new(p.x * c + p.z * s, p.y, -p.x * s + p.z * c)
    }
    
    /// Rotate point around X axis
    fn rotate_x(p: Vec3, angle: f32) -> Vec3 {
        let c = angle.cos();
        let s = angle.sin();
        Vec3::new(p.x, p.y * c - p.z * s, p.y * s + p.z * c)
    }
    
    /// SDF for a box
    fn sd_box(p: Vec3, b: Vec3) -> f32 {
        let q = p.abs() - b;
        q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0)
    }
    
    /// Get face color based on normal direction
    fn get_face_color(normal: Vec3) -> Vec3 {
        // Different color for each face
        if normal.x > 0.5 { return Vec3::new(1.0, 0.2, 0.2); }      // Right - Red
        if normal.x < -0.5 { return Vec3::new(0.2, 1.0, 0.2); }     // Left - Green
        if normal.y > 0.5 { return Vec3::new(0.2, 0.2, 1.0); }      // Top - Blue
        if normal.y < -0.5 { return Vec3::new(1.0, 1.0, 0.2); }     // Bottom - Yellow
        if normal.z > 0.5 { return Vec3::new(1.0, 0.2, 1.0); }      // Front - Magenta
        if normal.z < -0.5 { return Vec3::new(0.2, 1.0, 1.0); }     // Back - Cyan
        Vec3::new(0.5, 0.5, 0.5)
    }
    
    /// Calculate normal at point
    fn calc_normal(&self, p: Vec3) -> Vec3 {
        let eps = 0.001;
        let rotated = Self::rotate_x(Self::rotate_y(p, self.rotation), self.rotation * 0.7);
        let d = Self::sd_box(rotated, Vec3::splat(0.8));
        
        let dx = Self::sd_box(Self::rotate_x(Self::rotate_y(p + Vec3::new(eps, 0.0, 0.0), self.rotation), self.rotation * 0.7), Vec3::splat(0.8)) - d;
        let dy = Self::sd_box(Self::rotate_x(Self::rotate_y(p + Vec3::new(0.0, eps, 0.0), self.rotation), self.rotation * 0.7), Vec3::splat(0.8)) - d;
        let dz = Self::sd_box(Self::rotate_x(Self::rotate_y(p + Vec3::new(0.0, 0.0, eps), self.rotation), self.rotation * 0.7), Vec3::splat(0.8)) - d;
        
        Vec3::new(dx, dy, dz).normalize()
    }
    
    /// Render a single pixel
    fn render_pixel(&self, uv: Vec2, aspect: f32) -> Color {
        // Camera
        let camera_pos = Vec3::new(0.0, 0.0, self.camera_distance);
        let fov = 1.5;
        let rd = Vec3::new(uv.x * aspect * fov, uv.y * fov, -1.0).normalize();
        
        // Light direction
        let light_dir = Vec3::new(0.5, 0.8, 0.6).normalize();
        
        // Background gradient
        let bg_top = Vec3::new(0.1, 0.1, 0.15);
        let bg_bottom = Vec3::new(0.2, 0.2, 0.25);
        let bg = bg_bottom.lerp(bg_top, (uv.y * 0.5 + 0.5).clamp(0.0, 1.0));
        
        // Ray march
        let mut t = 0.0f32;
        let max_t = 20.0;
        
        for _ in 0..64 {
            let p = camera_pos + rd * t;
            
            // Rotate the point (inverse rotation of cube)
            let rotated = Self::rotate_x(Self::rotate_y(p, self.rotation), self.rotation * 0.7);
            let d = Self::sd_box(rotated, Vec3::splat(0.8));
            
            if d < 0.001 {
                // Hit the cube!
                let normal = self.calc_normal(p);
                
                // Get face color
                let rotated_normal = Self::rotate_x(Self::rotate_y(normal, -self.rotation), -self.rotation * 0.7);
                let base_color = Self::get_face_color(rotated_normal);
                
                // Lighting
                let n_dot_l = normal.dot(light_dir).max(0.0);
                let ambient = 0.3;
                let diffuse = n_dot_l * 0.7;
                
                // Specular
                let view_dir = -rd;
                let half_vec = (light_dir + view_dir).normalize();
                let spec = normal.dot(half_vec).max(0.0).powf(32.0) * 0.5;
                
                let color = base_color * (ambient + diffuse) + Vec3::splat(spec);
                
                return Color::new(
                    color.x.clamp(0.0, 1.0),
                    color.y.clamp(0.0, 1.0),
                    color.z.clamp(0.0, 1.0),
                    1.0,
                );
            }
            
            t += d;
            if t > max_t { break; }
        }
        
        // Background
        Color::new(bg.x, bg.y, bg.z, 1.0)
    }
    
    /// Get average color for the frame (for DirectX clear)
    fn get_frame_color(&self) -> Color {
        // Sample a few pixels to get average
        let samples = [
            self.render_pixel(Vec2::new(0.0, 0.0), 1.0),
            self.render_pixel(Vec2::new(0.3, 0.3), 1.0),
            self.render_pixel(Vec2::new(-0.3, -0.3), 1.0),
            self.render_pixel(Vec2::new(0.3, -0.3), 1.0),
        ];
        
        let r = (samples[0].r + samples[1].r + samples[2].r + samples[3].r) / 4.0;
        let g = (samples[0].g + samples[1].g + samples[2].g + samples[3].g) / 4.0;
        let b = (samples[0].b + samples[1].b + samples[2].b + samples[3].b) / 4.0;
        
        Color::new(r, g, b, 1.0)
    }
    
    /// Print ASCII representation
    fn print_ascii(&self) {
        let chars = " .:-=+*#%@";
        let w = 60;
        let h = 25;
        let aspect = w as f32 / h as f32 / 2.0;
        
        println!();
        for y in 0..h {
            for x in 0..w {
                let uv = Vec2::new(
                    (x as f32 / w as f32) * 2.0 - 1.0,
                    1.0 - (y as f32 / h as f32) * 2.0,
                );
                let color = self.render_pixel(uv, aspect);
                let brightness = (color.r * 0.299 + color.g * 0.587 + color.b * 0.114).clamp(0.0, 0.999);
                let idx = (brightness * chars.len() as f32) as usize;
                print!("{}", chars.chars().nth(idx).unwrap_or(' '));
            }
            println!();
        }
    }
}

// ============================================================================
// APPLICATION
// ============================================================================

struct App {
    window: Option<Window>,
    device: Option<Device>,
    command_queue: Option<CommandQueue>,
    swap_chain: Option<SwapChain>,
    allocator: Option<CommandAllocator>,
    cube: CubeRenderer,
    last_frame: Instant,
    frame_count: u64,
    last_fps_time: Instant,
    fps: f32,
    last_ascii_time: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            device: None,
            command_queue: None,
            swap_chain: None,
            allocator: None,
            cube: CubeRenderer::new(),
            last_frame: Instant::now(),
            frame_count: 0,
            last_fps_time: Instant::now(),
            fps: 0.0,
            last_ascii_time: Instant::now(),
        }
    }
    
    fn render(&mut self) {
        let Some(device) = &self.device else { return };
        let Some(command_queue) = &mut self.command_queue else { return };
        let Some(swap_chain) = &mut self.swap_chain else { return };
        let Some(allocator) = &self.allocator else { return };
        let Some(window) = &self.window else { return };
        
        // Delta time
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        
        // Update cube rotation
        self.cube.update(dt);
        
        // FPS
        self.frame_count += 1;
        let fps_elapsed = self.last_fps_time.elapsed().as_secs_f32();
        if fps_elapsed >= 1.0 {
            self.fps = self.frame_count as f32 / fps_elapsed;
            self.frame_count = 0;
            self.last_fps_time = Instant::now();
            
            window.set_title(&format!(
                "EPICX Cube | FPS: {:.0} | Rotation: {:.1}°",
                self.fps, self.cube.rotation.to_degrees() % 360.0
            ));
        }
        
        // Print ASCII every 2 seconds
        if self.last_ascii_time.elapsed().as_secs_f32() > 2.0 {
            self.cube.print_ascii();
            self.last_ascii_time = Instant::now();
        }
        
        // Get color from cube renderer
        let clear_color = self.cube.get_frame_color();
        
        // Reset allocator
        if let Err(e) = allocator.reset() {
            eprintln!("Allocator reset error: {:?}", e);
            return;
        }
        
        // Create command list
        let cmd_list = match CommandList::new(device, allocator, None) {
            Ok(list) => list,
            Err(e) => {
                eprintln!("Command list error: {:?}", e);
                return;
            }
        };
        
        // Get current back buffer and RTV
        let back_buffer = swap_chain.current_back_buffer();
        let rtv = swap_chain.current_rtv();
        
        unsafe {
            // Transition to render target
            let barrier = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: std::mem::transmute_copy(back_buffer),
                        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: D3D12_RESOURCE_STATE_PRESENT,
                        StateAfter: D3D12_RESOURCE_STATE_RENDER_TARGET,
                    }),
                },
            };
            cmd_list.raw().ResourceBarrier(&[barrier]);
            
            // Clear with cube color
            let color = [clear_color.r, clear_color.g, clear_color.b, 1.0];
            cmd_list.raw().ClearRenderTargetView(rtv, &color, None);
            
            // Transition back to present
            let barrier = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: std::mem::transmute_copy(back_buffer),
                        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: D3D12_RESOURCE_STATE_RENDER_TARGET,
                        StateAfter: D3D12_RESOURCE_STATE_PRESENT,
                    }),
                },
            };
            cmd_list.raw().ResourceBarrier(&[barrier]);
        }
        
        // Close and execute
        if let Err(e) = cmd_list.close() {
            eprintln!("Command list close error: {:?}", e);
            return;
        }
        
        command_queue.execute(&[&cmd_list]);
        
        // Present
        if let Err(e) = swap_chain.present() {
            eprintln!("Present error: {:?}", e);
            return;
        }
        
        // Wait for GPU
        if let Err(e) = command_queue.flush() {
            eprintln!("Flush error: {:?}", e);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!();
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║           EPICX - Vulkan Cube Style Demo                     ║");
        println!("║                                                              ║");
        println!("║  A simple rotating cube like vkcube from Vulkan SDK          ║");
        println!("║  but using DirectX12 (Xbox compatible!)                      ║");
        println!("║                                                              ║");
        println!("║  Controls: ESC to exit                                       ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        
        // Detect GPU
        let gpu = detect_gpu();
        
        if gpu.is_none() {
            eprintln!("[ERROR] No compatible GPU found!");
            event_loop.exit();
            return;
        }
        
        println!();
        
        // Create window
        let window_attrs = Window::default_attributes()
            .with_title("EPICX Cube")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
        
        let window = event_loop.create_window(window_attrs).expect("Failed to create window");
        let size = window.inner_size();
        
        // Get HWND
        let hwnd = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get() as *mut std::ffi::c_void),
            _ => panic!("Unsupported platform"),
        };
        
        // Initialize DirectX12
        println!("[INIT] Creating DirectX12 device...");
        let device = Device::new(cfg!(debug_assertions)).expect("Failed to create device");
        
        println!("[INIT] Creating command queue...");
        let command_queue = CommandQueue::graphics(&device).expect("Failed to create command queue");
        
        println!("[INIT] Creating swap chain ({}x{})...", size.width, size.height);
        let config = SwapChainConfig {
            width: size.width,
            height: size.height,
            buffer_count: 2,
            vsync: true,
            ..Default::default()
        };
        let swap_chain = SwapChain::new(&device, &command_queue, hwnd, config)
            .expect("Failed to create swap chain");
        
        println!("[INIT] Creating command allocator...");
        let allocator = CommandAllocator::new(&device, D3D12_COMMAND_LIST_TYPE_DIRECT)
            .expect("Failed to create allocator");
        
        println!("[INIT] Ready! Starting render loop...");
        println!();
        println!("Watch the ASCII cube rotate below (updates every 2 seconds):");
        
        // Print initial cube
        self.cube.print_ascii();
        
        self.window = Some(window);
        self.device = Some(device);
        self.command_queue = Some(command_queue);
        self.swap_chain = Some(swap_chain);
        self.allocator = Some(allocator);
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("\n[EXIT] Window closed");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) {
                    println!("\n[EXIT] ESC pressed");
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    if let (Some(device), Some(swap_chain), Some(queue)) = 
                        (&self.device, &mut self.swap_chain, &mut self.command_queue) {
                        let _ = queue.flush();
                        let _ = swap_chain.resize(device, new_size.width, new_size.height);
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
