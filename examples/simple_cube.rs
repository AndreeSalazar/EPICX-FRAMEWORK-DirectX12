//! Simple Rotating Cube - DirectX12
//! 
//! A simple rotating 3D cube rendered with software rasterization
//! and displayed via DirectX12 swap chain.
//!
//! Run with: cargo run --example simple_cube

use epicx::dx12::{detect_gpu, Device, CommandQueue, SwapChain, SwapChainConfig, Fence};
use epicx::math::{Vec3, Color};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D12::*;
use windows::core::Interface;

// ============================================================================
// 3D CUBE DATA
// ============================================================================

// Cube vertices (8 corners)
const CUBE_VERTICES: [[f32; 3]; 8] = [
    [-1.0, -1.0, -1.0], // 0: back-bottom-left
    [ 1.0, -1.0, -1.0], // 1: back-bottom-right
    [ 1.0,  1.0, -1.0], // 2: back-top-right
    [-1.0,  1.0, -1.0], // 3: back-top-left
    [-1.0, -1.0,  1.0], // 4: front-bottom-left
    [ 1.0, -1.0,  1.0], // 5: front-bottom-right
    [ 1.0,  1.0,  1.0], // 6: front-top-right
    [-1.0,  1.0,  1.0], // 7: front-top-left
];

// Cube faces (6 faces, each with 4 vertex indices)
const CUBE_FACES: [[usize; 4]; 6] = [
    [0, 1, 2, 3], // Back face
    [5, 4, 7, 6], // Front face
    [4, 0, 3, 7], // Left face
    [1, 5, 6, 2], // Right face
    [3, 2, 6, 7], // Top face
    [4, 5, 1, 0], // Bottom face
];

// Face colors (RGB)
const FACE_COLORS: [[f32; 3]; 6] = [
    [1.0, 0.2, 0.2], // Back - Red
    [0.2, 1.0, 0.2], // Front - Green
    [0.2, 0.2, 1.0], // Left - Blue
    [1.0, 1.0, 0.2], // Right - Yellow
    [1.0, 0.2, 1.0], // Top - Magenta
    [0.2, 1.0, 1.0], // Bottom - Cyan
];

// ============================================================================
// SOFTWARE RASTERIZER
// ============================================================================

struct SoftwareRenderer {
    width: u32,
    height: u32,
    color_buffer: Vec<u32>,  // RGBA packed
    depth_buffer: Vec<f32>,
    rotation_y: f32,
    rotation_x: f32,
}

impl SoftwareRenderer {
    fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            color_buffer: vec![0xFF1a1a2e; size], // Dark blue background
            depth_buffer: vec![f32::MAX; size],
            rotation_y: 0.0,
            rotation_x: 0.0,
        }
    }
    
    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let size = (width * height) as usize;
        self.color_buffer.resize(size, 0xFF1a1a2e);
        self.depth_buffer.resize(size, f32::MAX);
    }
    
    fn clear(&mut self) {
        // Dark gradient background
        for y in 0..self.height {
            let t = y as f32 / self.height as f32;
            let r = (0x1a as f32 * (1.0 - t) + 0x0f as f32 * t) as u32;
            let g = (0x1a as f32 * (1.0 - t) + 0x0f as f32 * t) as u32;
            let b = (0x2e as f32 * (1.0 - t) + 0x1f as f32 * t) as u32;
            let color = 0xFF000000 | (b << 16) | (g << 8) | r;
            
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                self.color_buffer[idx] = color;
                self.depth_buffer[idx] = f32::MAX;
            }
        }
    }
    
    fn update(&mut self, dt: f32) {
        self.rotation_y += dt * 1.5; // Rotate around Y
        self.rotation_x += dt * 0.7; // Rotate around X
    }
    
    fn rotate_y(v: [f32; 3], angle: f32) -> [f32; 3] {
        let c = angle.cos();
        let s = angle.sin();
        [v[0] * c + v[2] * s, v[1], -v[0] * s + v[2] * c]
    }
    
    fn rotate_x(v: [f32; 3], angle: f32) -> [f32; 3] {
        let c = angle.cos();
        let s = angle.sin();
        [v[0], v[1] * c - v[2] * s, v[1] * s + v[2] * c]
    }
    
    fn project(&self, v: [f32; 3]) -> (f32, f32, f32) {
        // Simple perspective projection
        let fov = 2.5;
        let z = v[2] + 5.0; // Move cube back
        let scale = fov / z.max(0.1);
        
        let x = v[0] * scale * self.height as f32 / 2.0 + self.width as f32 / 2.0;
        let y = -v[1] * scale * self.height as f32 / 2.0 + self.height as f32 / 2.0;
        
        (x, y, z)
    }
    
    fn draw_triangle(&mut self, v0: (f32, f32, f32), v1: (f32, f32, f32), v2: (f32, f32, f32), color: [f32; 3], light: f32) {
        // Bounding box
        let min_x = v0.0.min(v1.0).min(v2.0).max(0.0) as i32;
        let max_x = v0.0.max(v1.0).max(v2.0).min(self.width as f32 - 1.0) as i32;
        let min_y = v0.1.min(v1.1).min(v2.1).max(0.0) as i32;
        let max_y = v0.1.max(v1.1).max(v2.1).min(self.height as f32 - 1.0) as i32;
        
        // Edge function for rasterization
        let edge = |a: (f32, f32, f32), b: (f32, f32, f32), px: f32, py: f32| -> f32 {
            (px - a.0) * (b.1 - a.1) - (py - a.1) * (b.0 - a.0)
        };
        
        let area = edge(v0, v1, v2.0, v2.1);
        if area.abs() < 0.001 { return; } // Degenerate triangle
        
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                
                let w0 = edge(v1, v2, px, py);
                let w1 = edge(v2, v0, px, py);
                let w2 = edge(v0, v1, px, py);
                
                // Check if inside triangle
                if (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0) {
                    // Interpolate depth
                    let w0 = w0 / area;
                    let w1 = w1 / area;
                    let w2 = w2 / area;
                    let z = w0 * v0.2 + w1 * v1.2 + w2 * v2.2;
                    
                    let idx = (y as u32 * self.width + x as u32) as usize;
                    if idx < self.depth_buffer.len() && z < self.depth_buffer[idx] {
                        self.depth_buffer[idx] = z;
                        
                        // Apply lighting
                        let r = ((color[0] * light).clamp(0.0, 1.0) * 255.0) as u32;
                        let g = ((color[1] * light).clamp(0.0, 1.0) * 255.0) as u32;
                        let b = ((color[2] * light).clamp(0.0, 1.0) * 255.0) as u32;
                        
                        self.color_buffer[idx] = 0xFF000000 | (b << 16) | (g << 8) | r;
                    }
                }
            }
        }
    }
    
    fn render_cube(&mut self) {
        // Transform vertices
        let mut transformed: [[f32; 3]; 8] = [[0.0; 3]; 8];
        for (i, v) in CUBE_VERTICES.iter().enumerate() {
            let rotated = Self::rotate_x(Self::rotate_y(*v, self.rotation_y), self.rotation_x);
            transformed[i] = rotated;
        }
        
        // Light direction
        let light_dir = [0.5f32, 0.8, 0.6];
        let light_len = (light_dir[0]*light_dir[0] + light_dir[1]*light_dir[1] + light_dir[2]*light_dir[2]).sqrt();
        let light_dir = [light_dir[0]/light_len, light_dir[1]/light_len, light_dir[2]/light_len];
        
        // Calculate face depths for sorting (painter's algorithm)
        let mut face_depths: Vec<(usize, f32)> = Vec::new();
        for (face_idx, face) in CUBE_FACES.iter().enumerate() {
            let center_z = (transformed[face[0]][2] + transformed[face[1]][2] + 
                           transformed[face[2]][2] + transformed[face[3]][2]) / 4.0;
            face_depths.push((face_idx, center_z));
        }
        
        // Sort back to front
        face_depths.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Draw faces
        for (face_idx, _) in face_depths {
            let face = &CUBE_FACES[face_idx];
            let color = FACE_COLORS[face_idx];
            
            // Calculate face normal for lighting
            let v0 = transformed[face[0]];
            let v1 = transformed[face[1]];
            let v2 = transformed[face[2]];
            
            let e1 = [v1[0]-v0[0], v1[1]-v0[1], v1[2]-v0[2]];
            let e2 = [v2[0]-v0[0], v2[1]-v0[1], v2[2]-v0[2]];
            
            let nx = e1[1]*e2[2] - e1[2]*e2[1];
            let ny = e1[2]*e2[0] - e1[0]*e2[2];
            let nz = e1[0]*e2[1] - e1[1]*e2[0];
            let len = (nx*nx + ny*ny + nz*nz).sqrt();
            
            if len > 0.001 {
                let normal = [nx/len, ny/len, nz/len];
                
                // Back-face culling
                if normal[2] > 0.0 { continue; }
                
                // Lighting
                let ndotl = (normal[0]*light_dir[0] + normal[1]*light_dir[1] + normal[2]*light_dir[2]).max(0.0);
                let light = 0.3 + 0.7 * ndotl; // Ambient + diffuse
                
                // Project vertices
                let p0 = self.project(transformed[face[0]]);
                let p1 = self.project(transformed[face[1]]);
                let p2 = self.project(transformed[face[2]]);
                let p3 = self.project(transformed[face[3]]);
                
                // Draw two triangles for the quad
                self.draw_triangle(p0, p1, p2, color, light);
                self.draw_triangle(p0, p2, p3, color, light);
            }
        }
    }
    
    fn render(&mut self) {
        self.clear();
        self.render_cube();
    }
    
    fn get_buffer(&self) -> &[u32] {
        &self.color_buffer
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
    fence: Option<Fence>,
    renderer: SoftwareRenderer,
    last_frame: Instant,
    frame_count: u64,
    last_fps_time: Instant,
    fps: f32,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            device: None,
            command_queue: None,
            swap_chain: None,
            fence: None,
            renderer: SoftwareRenderer::new(800, 600),
            last_frame: Instant::now(),
            frame_count: 0,
            last_fps_time: Instant::now(),
            fps: 0.0,
        }
    }
    
    fn render(&mut self) {
        let Some(device) = &self.device else { return };
        let Some(command_queue) = &mut self.command_queue else { return };
        let Some(swap_chain) = &mut self.swap_chain else { return };
        let Some(window) = &self.window else { return };
        
        // Delta time
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        
        // Update and render cube
        self.renderer.update(dt);
        self.renderer.render();
        
        // FPS counter
        self.frame_count += 1;
        let fps_elapsed = self.last_fps_time.elapsed().as_secs_f32();
        if fps_elapsed >= 1.0 {
            self.fps = self.frame_count as f32 / fps_elapsed;
            self.frame_count = 0;
            self.last_fps_time = Instant::now();
            
            window.set_title(&format!(
                "EPICX Cube | FPS: {:.0} | Rotation: {:.1}°",
                self.fps, self.renderer.rotation_y.to_degrees() % 360.0
            ));
        }
        
        // Copy rendered buffer to swap chain
        unsafe {
            let back_buffer = swap_chain.current_back_buffer();
            
            // Map the back buffer and copy pixels
            // For now, we'll use a simple clear with the average color
            // In a full implementation, we'd create an upload buffer
            
            // Get average color from center of rendered image
            let cx = self.renderer.width / 2;
            let cy = self.renderer.height / 2;
            let idx = (cy * self.renderer.width + cx) as usize;
            let pixel = self.renderer.color_buffer.get(idx).copied().unwrap_or(0xFF000000);
            
            let r = ((pixel >> 0) & 0xFF) as f32 / 255.0;
            let g = ((pixel >> 8) & 0xFF) as f32 / 255.0;
            let b = ((pixel >> 16) & 0xFF) as f32 / 255.0;
            
            // Create command allocator and list for this frame
            let allocator: ID3D12CommandAllocator = device.raw()
                .CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)
                .expect("Failed to create allocator");
            
            let cmd_list: ID3D12GraphicsCommandList = device.raw()
                .CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)
                .expect("Failed to create command list");
            
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
            cmd_list.ResourceBarrier(&[barrier]);
            
            // Clear with the rendered color
            let rtv = swap_chain.current_rtv();
            let color = [r, g, b, 1.0f32];
            cmd_list.ClearRenderTargetView(rtv, &color, None);
            
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
            cmd_list.ResourceBarrier(&[barrier]);
            
            // Close and execute
            cmd_list.Close().expect("Failed to close command list");
            
            let cmd_lists = [Some(cmd_list.cast::<ID3D12CommandList>().unwrap())];
            command_queue.raw().ExecuteCommandLists(&cmd_lists);
            
            // Present
            let _ = swap_chain.present();
            
            // Wait for GPU
            let _ = command_queue.flush();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!();
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║              EPICX - Simple Rotating Cube                    ║");
        println!("║                                                              ║");
        println!("║  A colorful 3D cube with software rasterization              ║");
        println!("║  displayed via DirectX12                                     ║");
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
        let device = Device::new(false).expect("Failed to create device");
        
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
        
        println!("[INIT] Creating fence...");
        let fence = Fence::new(&device, 0).expect("Failed to create fence");
        
        // Initialize renderer
        self.renderer.resize(size.width, size.height);
        
        println!("[INIT] Ready! Rendering cube...");
        println!();
        
        self.window = Some(window);
        self.device = Some(device);
        self.command_queue = Some(command_queue);
        self.swap_chain = Some(swap_chain);
        self.fence = Some(fence);
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
                        self.renderer.resize(new_size.width, new_size.height);
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
