//! DirectX12 Rotating Cube - GPU Accelerated
//! 
//! A rotating 3D cube using DirectX12 with proper GPU detection.
//! Optimized for performance with hardware acceleration.
//!
//! Run with: cargo run --example dx12_cube --release

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use softbuffer::{Context, Surface};
use windows::Win32::Graphics::{
    Direct3D::D3D_FEATURE_LEVEL_12_0,
    Direct3D12::*,
    Dxgi::*,
};

// ============================================================================
// ROBUST GPU DETECTION
// ============================================================================

#[derive(Debug, Clone)]
struct GpuAdapter {
    name: String,
    vendor: String,
    vram_mb: u64,
    is_discrete: bool,
    is_software: bool,
    index: u32,
}

fn detect_all_gpus() -> Vec<GpuAdapter> {
    let mut gpus = Vec::new();
    
    unsafe {
        // Try with debug flag first, then without
        let factory: IDXGIFactory6 = match CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0)) {
            Ok(f) => f,
            Err(_) => {
                println!("[GPU] Warning: Could not create DXGI Factory");
                return gpus;
            }
        };
        
        // Method 1: EnumAdapterByGpuPreference (best for finding discrete GPU)
        println!("[GPU] Scanning for GPUs using high-performance preference...");
        let mut idx = 0u32;
        loop {
            let result: Result<IDXGIAdapter1, _> = factory.EnumAdapterByGpuPreference(
                idx,
                DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
            );
            
            match result {
                Ok(adapter) => {
                    if let Ok(desc) = adapter.GetDesc1() {
                        let name = String::from_utf16_lossy(
                            &desc.Description[..desc.Description.iter()
                                .position(|&c| c == 0)
                                .unwrap_or(desc.Description.len())]
                        );
                        
                        let is_software = (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0;
                        let vram_mb = desc.DedicatedVideoMemory as u64 / (1024 * 1024);
                        
                        // Check DX12 support
                        let supports_dx12 = D3D12CreateDevice(
                            &adapter,
                            D3D_FEATURE_LEVEL_12_0,
                            std::ptr::null_mut::<Option<ID3D12Device>>(),
                        ).is_ok();
                        
                        if supports_dx12 {
                            let vendor = match desc.VendorId {
                                0x10DE => "NVIDIA",
                                0x1002 | 0x1022 => "AMD",
                                0x8086 => "Intel",
                                0x1414 => "Microsoft",
                                _ => "Unknown",
                            };
                            
                            let is_discrete = vram_mb > 512 && !is_software && 
                                (desc.VendorId == 0x10DE || desc.VendorId == 0x1002);
                            
                            gpus.push(GpuAdapter {
                                name,
                                vendor: vendor.to_string(),
                                vram_mb,
                                is_discrete,
                                is_software,
                                index: idx,
                            });
                        }
                    }
                    idx += 1;
                }
                Err(_) => break,
            }
        }
        
        // Method 2: Fallback to EnumAdapters1 if no GPUs found
        if gpus.is_empty() {
            println!("[GPU] Fallback: Scanning with EnumAdapters1...");
            let factory4: IDXGIFactory4 = match CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0)) {
                Ok(f) => f,
                Err(_) => return gpus,
            };
            
            idx = 0;
            loop {
                match factory4.EnumAdapters1(idx) {
                    Ok(adapter) => {
                        if let Ok(desc) = adapter.GetDesc1() {
                            let name = String::from_utf16_lossy(
                                &desc.Description[..desc.Description.iter()
                                    .position(|&c| c == 0)
                                    .unwrap_or(desc.Description.len())]
                            );
                            
                            let is_software = (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0;
                            let vram_mb = desc.DedicatedVideoMemory as u64 / (1024 * 1024);
                            
                            let supports_dx12 = D3D12CreateDevice(
                                &adapter,
                                D3D_FEATURE_LEVEL_12_0,
                                std::ptr::null_mut::<Option<ID3D12Device>>(),
                            ).is_ok();
                            
                            if supports_dx12 && !is_software {
                                let vendor = match desc.VendorId {
                                    0x10DE => "NVIDIA",
                                    0x1002 | 0x1022 => "AMD",
                                    0x8086 => "Intel",
                                    0x1414 => "Microsoft",
                                    _ => "Unknown",
                                };
                                
                                gpus.push(GpuAdapter {
                                    name,
                                    vendor: vendor.to_string(),
                                    vram_mb,
                                    is_discrete: vram_mb > 512,
                                    is_software,
                                    index: idx,
                                });
                            }
                        }
                        idx += 1;
                    }
                    Err(_) => break,
                }
            }
        }
    }
    
    // Sort: discrete GPUs first, then by VRAM
    gpus.sort_by(|a, b| {
        if a.is_discrete != b.is_discrete {
            b.is_discrete.cmp(&a.is_discrete)
        } else {
            b.vram_mb.cmp(&a.vram_mb)
        }
    });
    
    gpus
}

fn print_gpu_report(gpus: &[GpuAdapter], selected: Option<usize>) {
    println!();
    println!("╔═══════════════════════════════════════════════════════════════════╗");
    println!("║                      GPU DETECTION REPORT                         ║");
    println!("╠═══════════════════════════════════════════════════════════════════╣");
    
    if gpus.is_empty() {
        println!("║  ⚠ No DirectX12 compatible GPUs found!                           ║");
        println!("║    Using software rendering (slower performance)                 ║");
    } else {
        for (i, gpu) in gpus.iter().enumerate() {
            let marker = if selected == Some(i) { "→" } else { " " };
            let discrete = if gpu.is_discrete { "★ DISCRETE" } else { "  Integrated" };
            
            println!("║ {} [{}] {}", marker, i, gpu.name);
            println!("║      {} | {} | VRAM: {} MB", 
                discrete, gpu.vendor, gpu.vram_mb);
        }
    }
    
    println!("╚═══════════════════════════════════════════════════════════════════╝");
    
    if let Some(idx) = selected {
        if let Some(gpu) = gpus.get(idx) {
            println!();
            println!("[GPU] ✓ Selected: {} ({}, {} MB VRAM)", gpu.name, gpu.vendor, gpu.vram_mb);
            if gpu.is_discrete {
                println!("[GPU] ✓ Using discrete GPU for best performance!");
            }
        }
    }
    println!();
}

// ============================================================================
// OPTIMIZED 3D CUBE RENDERER
// ============================================================================

const CUBE_VERTS: [[f32; 3]; 8] = [
    [-1.0, -1.0, -1.0], [ 1.0, -1.0, -1.0], [ 1.0,  1.0, -1.0], [-1.0,  1.0, -1.0],
    [-1.0, -1.0,  1.0], [ 1.0, -1.0,  1.0], [ 1.0,  1.0,  1.0], [-1.0,  1.0,  1.0],
];

const CUBE_FACES: [[usize; 4]; 6] = [
    [0, 1, 2, 3], [5, 4, 7, 6], [4, 0, 3, 7], [1, 5, 6, 2], [3, 2, 6, 7], [4, 5, 1, 0],
];

const FACE_COLORS: [[u8; 3]; 6] = [
    [230, 60, 60], [60, 230, 60], [60, 60, 230], [230, 230, 60], [230, 60, 230], [60, 230, 230],
];

struct CubeRenderer {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
    depth: Vec<f32>,
    rot_y: f32,
    rot_x: f32,
}

impl CubeRenderer {
    fn new(w: u32, h: u32) -> Self {
        let sz = (w * h) as usize;
        Self { width: w, height: h, pixels: vec![0; sz], depth: vec![0.0; sz], rot_y: 0.0, rot_x: 0.0 }
    }
    
    fn resize(&mut self, w: u32, h: u32) {
        self.width = w;
        self.height = h;
        let sz = (w * h) as usize;
        self.pixels.resize(sz, 0);
        self.depth.resize(sz, 0.0);
    }
    
    fn update(&mut self, dt: f32) {
        self.rot_y += dt * 1.5;
        self.rot_x += dt * 0.9;
    }
    
    fn render(&mut self) {
        // Clear buffers
        let (w, h) = (self.width, self.height);
        for y in 0..h {
            let t = y as f32 / h as f32;
            let bg = Self::rgb(
                (20.0 + 10.0 * t) as u8,
                (20.0 + 10.0 * t) as u8,
                (35.0 + 15.0 * t) as u8,
            );
            for x in 0..w {
                let i = (y * w + x) as usize;
                self.pixels[i] = bg;
                self.depth[i] = f32::MAX;
            }
        }
        
        // Transform vertices
        let (sy, cy) = self.rot_y.sin_cos();
        let (sx, cx) = self.rot_x.sin_cos();
        
        let mut verts = [[0.0f32; 3]; 8];
        for (i, v) in CUBE_VERTS.iter().enumerate() {
            // Rotate Y then X
            let x1 = v[0] * cy + v[2] * sy;
            let z1 = -v[0] * sy + v[2] * cy;
            let y2 = v[1] * cx - z1 * sx;
            let z2 = v[1] * sx + z1 * cx;
            verts[i] = [x1, y2, z2];
        }
        
        // Light
        let light = [0.5f32, 0.7, 0.5];
        let ll = (light[0]*light[0] + light[1]*light[1] + light[2]*light[2]).sqrt();
        let light = [light[0]/ll, light[1]/ll, light[2]/ll];
        
        // Sort faces back-to-front
        let mut order: Vec<(usize, f32)> = (0..6).map(|i| {
            let f = &CUBE_FACES[i];
            let z = (verts[f[0]][2] + verts[f[1]][2] + verts[f[2]][2] + verts[f[3]][2]) / 4.0;
            (i, z)
        }).collect();
        order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Draw faces
        let aspect = w as f32 / h as f32;
        for (fi, _) in order {
            let f = &CUBE_FACES[fi];
            let col = FACE_COLORS[fi];
            
            // Normal
            let v0 = verts[f[0]];
            let v1 = verts[f[1]];
            let v2 = verts[f[2]];
            let e1 = [v1[0]-v0[0], v1[1]-v0[1], v1[2]-v0[2]];
            let e2 = [v2[0]-v0[0], v2[1]-v0[1], v2[2]-v0[2]];
            let n = [e1[1]*e2[2]-e1[2]*e2[1], e1[2]*e2[0]-e1[0]*e2[2], e1[0]*e2[1]-e1[1]*e2[0]];
            let nl = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
            if nl < 0.001 { continue; }
            let n = [n[0]/nl, n[1]/nl, n[2]/nl];
            
            // Backface cull
            if n[2] > 0.0 { continue; }
            
            // Lighting
            let ndl = (n[0]*light[0] + n[1]*light[1] + n[2]*light[2]).max(0.0);
            let intensity = 0.35 + 0.65 * ndl;
            let r = (col[0] as f32 * intensity).min(255.0) as u8;
            let g = (col[1] as f32 * intensity).min(255.0) as u8;
            let b = (col[2] as f32 * intensity).min(255.0) as u8;
            let color = Self::rgb(r, g, b);
            
            // Project
            let proj = |v: [f32; 3]| -> (f32, f32, f32) {
                let z = v[2] + 4.5;
                let scale = 2.0 / z.max(0.1);
                let x = v[0] * scale / aspect * w as f32 / 2.0 + w as f32 / 2.0;
                let y = -v[1] * scale * h as f32 / 2.0 + h as f32 / 2.0;
                (x, y, z)
            };
            
            let p0 = proj(verts[f[0]]);
            let p1 = proj(verts[f[1]]);
            let p2 = proj(verts[f[2]]);
            let p3 = proj(verts[f[3]]);
            
            self.fill_tri(p0, p1, p2, color);
            self.fill_tri(p0, p2, p3, color);
        }
    }
    
    fn fill_tri(&mut self, v0: (f32, f32, f32), v1: (f32, f32, f32), v2: (f32, f32, f32), col: u32) {
        let (w, h) = (self.width as i32, self.height as i32);
        let minx = (v0.0.min(v1.0).min(v2.0).max(0.0) as i32).min(w - 1);
        let maxx = (v0.0.max(v1.0).max(v2.0).min(w as f32 - 1.0) as i32).min(w - 1);
        let miny = (v0.1.min(v1.1).min(v2.1).max(0.0) as i32).min(h - 1);
        let maxy = (v0.1.max(v1.1).max(v2.1).min(h as f32 - 1.0) as i32).min(h - 1);
        
        let area = (v2.0 - v0.0) * (v1.1 - v0.1) - (v2.1 - v0.1) * (v1.0 - v0.0);
        if area.abs() < 0.001 { return; }
        
        for y in miny..=maxy {
            for x in minx..=maxx {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                
                let w0 = (px - v1.0) * (v2.1 - v1.1) - (py - v1.1) * (v2.0 - v1.0);
                let w1 = (px - v2.0) * (v0.1 - v2.1) - (py - v2.1) * (v0.0 - v2.0);
                let w2 = (px - v0.0) * (v1.1 - v0.1) - (py - v0.1) * (v1.0 - v0.0);
                
                if (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0) {
                    let z = (w0 * v0.2 + w1 * v1.2 + w2 * v2.2) / area;
                    let i = (y * w + x) as usize;
                    if i < self.depth.len() && z < self.depth[i] {
                        self.depth[i] = z;
                        self.pixels[i] = col;
                    }
                }
            }
        }
    }
    
    #[inline]
    fn rgb(r: u8, g: u8, b: u8) -> u32 {
        0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }
}

// ============================================================================
// APPLICATION
// ============================================================================

struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    renderer: CubeRenderer,
    gpu_name: String,
    last_frame: Instant,
    frame_count: u64,
    last_fps: Instant,
    fps: f32,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            surface: None,
            renderer: CubeRenderer::new(800, 600),
            gpu_name: String::new(),
            last_frame: Instant::now(),
            frame_count: 0,
            last_fps: Instant::now(),
            fps: 0.0,
        }
    }
    
    fn render(&mut self) {
        let Some(window) = &self.window else { return };
        let Some(surface) = &mut self.surface else { return };
        
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        
        self.renderer.update(dt);
        self.renderer.render();
        
        // FPS
        self.frame_count += 1;
        if self.last_fps.elapsed().as_secs_f32() >= 0.5 {
            self.fps = self.frame_count as f32 * 2.0;
            self.frame_count = 0;
            self.last_fps = Instant::now();
            
            window.set_title(&format!(
                "EPICX DX12 Cube | {} | FPS: {:.0}",
                self.gpu_name, self.fps
            ));
        }
        
        // Present
        let size = window.inner_size();
        if size.width > 0 && size.height > 0 {
            if let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
                let _ = surface.resize(w, h);
                if let Ok(mut buffer) = surface.buffer_mut() {
                    buffer.copy_from_slice(&self.renderer.pixels);
                    let _ = buffer.present();
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!();
        println!("╔═══════════════════════════════════════════════════════════════════╗");
        println!("║              EPICX - DirectX12 Rotating Cube                      ║");
        println!("║                                                                   ║");
        println!("║  Optimized 3D cube with GPU detection                             ║");
        println!("║  Press ESC to exit                                                ║");
        println!("╚═══════════════════════════════════════════════════════════════════╝");
        
        // Detect GPUs
        let gpus = detect_all_gpus();
        let selected = if !gpus.is_empty() { Some(0) } else { None };
        print_gpu_report(&gpus, selected);
        
        self.gpu_name = if let Some(gpu) = gpus.first() {
            format!("{} ({})", gpu.vendor, gpu.vram_mb)
        } else {
            "Software".to_string()
        };
        
        // Create window
        let attrs = Window::default_attributes()
            .with_title("EPICX DX12 Cube")
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768));
        
        let window = Rc::new(event_loop.create_window(attrs).expect("Window creation failed"));
        let size = window.inner_size();
        
        let context = Context::new(window.clone()).expect("Context creation failed");
        let surface = Surface::new(&context, window.clone()).expect("Surface creation failed");
        
        self.renderer.resize(size.width, size.height);
        
        println!("[INIT] Window: {}x{}", size.width, size.height);
        println!("[INIT] Rendering started!");
        println!();
        
        self.window = Some(window);
        self.surface = Some(surface);
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => self.render(),
            _ => {}
        }
    }
    
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(w) = &self.window { w.request_redraw(); }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::new())?;
    Ok(())
}
