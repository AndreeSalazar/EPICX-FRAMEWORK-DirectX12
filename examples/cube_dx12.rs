//! DirectX12 Cube - Maximum Performance
//! 
//! A rotating 3D cube using DirectX12 with VSync OFF for maximum FPS.
//! Uses software rasterization displayed via softbuffer for reliability.
//!
//! Run with: cargo run --example cube_dx12 --release

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use softbuffer::{Context, Surface};

// Cube vertices
const V: [[f32; 3]; 8] = [
    [-1.0, -1.0, -1.0], [1.0, -1.0, -1.0], [1.0, 1.0, -1.0], [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0], [1.0, -1.0, 1.0], [1.0, 1.0, 1.0], [-1.0, 1.0, 1.0],
];

// Faces: vertex indices + color
const FACES: [([usize; 4], u32); 6] = [
    ([0, 1, 2, 3], 0xFFE03030), // Back - Red
    ([5, 4, 7, 6], 0xFF30E030), // Front - Green  
    ([4, 0, 3, 7], 0xFF3030E0), // Left - Blue
    ([1, 5, 6, 2], 0xFFE0E030), // Right - Yellow
    ([3, 2, 6, 7], 0xFFE030E0), // Top - Magenta
    ([4, 5, 1, 0], 0xFF30E0E0), // Bottom - Cyan
];

struct Renderer {
    w: u32,
    h: u32,
    buf: Vec<u32>,
    zbuf: Vec<f32>,
    angle: f32,
}

impl Renderer {
    fn new(w: u32, h: u32) -> Self {
        let n = (w * h) as usize;
        Self { w, h, buf: vec![0; n], zbuf: vec![0.0; n], angle: 0.0 }
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.w = w; self.h = h;
        let n = (w * h) as usize;
        self.buf.resize(n, 0);
        self.zbuf.resize(n, 0.0);
    }

    #[inline]
    fn clear(&mut self) {
        self.buf.fill(0xFF101018);
        self.zbuf.fill(1e9);
    }

    fn render(&mut self, dt: f32) {
        self.angle += dt * 2.0;
        self.clear();

        let (sy, cy) = self.angle.sin_cos();
        let (sx, cx) = (self.angle * 0.7).sin_cos();

        // Transform vertices
        let mut tv = [[0.0f32; 4]; 8];
        for (i, v) in V.iter().enumerate() {
            let x1 = v[0] * cy + v[2] * sy;
            let z1 = -v[0] * sy + v[2] * cy;
            let y1 = v[1] * cx - z1 * sx;
            let z2 = v[1] * sx + z1 * cx + 5.0;
            let scale = 2.0 / z2;
            let asp = self.w as f32 / self.h as f32;
            tv[i] = [
                x1 * scale / asp * (self.w as f32 * 0.5) + self.w as f32 * 0.5,
                -y1 * scale * (self.h as f32 * 0.5) + self.h as f32 * 0.5,
                z2,
                0.0,
            ];
        }

        // Sort faces back to front
        let mut order: [(usize, f32); 6] = core::array::from_fn(|i| {
            let f = &FACES[i].0;
            (i, (tv[f[0]][2] + tv[f[1]][2] + tv[f[2]][2] + tv[f[3]][2]) * 0.25)
        });
        order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Draw faces
        for (fi, _) in order {
            let (f, col) = &FACES[fi];
            let (v0, v1, v2, v3) = (tv[f[0]], tv[f[1]], tv[f[2]], tv[f[3]]);
            
            // Normal for backface culling
            let nx = (v1[1] - v0[1]) * (v2[2] - v0[2]) - (v1[2] - v0[2]) * (v2[1] - v0[1]);
            let ny = (v1[2] - v0[2]) * (v2[0] - v0[0]) - (v1[0] - v0[0]) * (v2[2] - v0[2]);
            let nz = (v1[0] - v0[0]) * (v2[1] - v0[1]) - (v1[1] - v0[1]) * (v2[0] - v0[0]);
            
            // Simple lighting
            let light = (0.3 + 0.7 * (nx * 0.5 + ny * 0.7 + nz * 0.5).abs() / (nx*nx + ny*ny + nz*nz).sqrt().max(0.001)).min(1.0);
            
            let r = (((*col >> 16) & 0xFF) as f32 * light) as u32;
            let g = (((*col >> 8) & 0xFF) as f32 * light) as u32;
            let b = ((*col & 0xFF) as f32 * light) as u32;
            let lit_col = 0xFF000000 | (r << 16) | (g << 8) | b;

            self.tri(v0, v1, v2, lit_col);
            self.tri(v0, v2, v3, lit_col);
        }
    }

    #[inline]
    fn tri(&mut self, a: [f32; 4], b: [f32; 4], c: [f32; 4], col: u32) {
        let (w, h) = (self.w as i32, self.h as i32);
        let x0 = (a[0].min(b[0]).min(c[0]).max(0.0) as i32).min(w - 1);
        let x1 = (a[0].max(b[0]).max(c[0]) as i32 + 1).min(w);
        let y0 = (a[1].min(b[1]).min(c[1]).max(0.0) as i32).min(h - 1);
        let y1 = (a[1].max(b[1]).max(c[1]) as i32 + 1).min(h);

        let area = (c[0] - a[0]) * (b[1] - a[1]) - (c[1] - a[1]) * (b[0] - a[0]);
        if area.abs() < 0.5 { return; }
        let inv = 1.0 / area;

        for y in y0..y1 {
            for x in x0..x1 {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                
                let w0 = ((px - b[0]) * (c[1] - b[1]) - (py - b[1]) * (c[0] - b[0])) * inv;
                let w1 = ((px - c[0]) * (a[1] - c[1]) - (py - c[1]) * (a[0] - c[0])) * inv;
                let w2 = 1.0 - w0 - w1;

                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    let z = w0 * a[2] + w1 * b[2] + w2 * c[2];
                    let i = (y * w + x) as usize;
                    if z < self.zbuf[i] {
                        self.zbuf[i] = z;
                        self.buf[i] = col;
                    }
                }
            }
        }
    }
}

struct App {
    win: Option<Rc<Window>>,
    surf: Option<Surface<Rc<Window>, Rc<Window>>>,
    ren: Renderer,
    t: Instant,
    frames: u64,
    fps_t: Instant,
    fps: f32,
    total_frames: u64,
    start: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            win: None, surf: None, ren: Renderer::new(1280, 720),
            t: Instant::now(), frames: 0, fps_t: Instant::now(), fps: 0.0,
            total_frames: 0, start: Instant::now(),
        }
    }

    fn draw(&mut self) {
        let Some(win) = &self.win else { return };
        let Some(surf) = &mut self.surf else { return };

        let now = Instant::now();
        let dt = (now - self.t).as_secs_f32();
        self.t = now;

        self.ren.render(dt);
        self.frames += 1;
        self.total_frames += 1;

        // FPS every 0.25s for faster updates
        if self.fps_t.elapsed().as_secs_f32() >= 0.25 {
            self.fps = self.frames as f32 * 4.0;
            self.frames = 0;
            self.fps_t = Instant::now();
            
            let avg = self.total_frames as f32 / self.start.elapsed().as_secs_f32();
            win.set_title(&format!("EPICX Cube | FPS: {:.0} | Avg: {:.0} | Frame: {}", self.fps, avg, self.total_frames));
        }

        let sz = win.inner_size();
        if sz.width > 0 && sz.height > 0 {
            if let (Some(w), Some(h)) = (NonZeroU32::new(sz.width), NonZeroU32::new(sz.height)) {
                let _ = surf.resize(w, h);
                if let Ok(mut buf) = surf.buffer_mut() {
                    buf.copy_from_slice(&self.ren.buf);
                    let _ = buf.present();
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        println!("\n╔═══════════════════════════════════════════════════════════════╗");
        println!("║         EPICX - High Performance Rotating Cube               ║");
        println!("║                                                               ║");
        println!("║  Optimized software rasterizer - NO VSYNC                     ║");
        println!("║  Press ESC to exit                                            ║");
        println!("╚═══════════════════════════════════════════════════════════════╝\n");

        // GPU Detection
        unsafe {
            use windows::Win32::Graphics::Dxgi::*;
            if let Ok(factory) = CreateDXGIFactory2::<IDXGIFactory6>(DXGI_CREATE_FACTORY_FLAGS(0)) {
                if let Ok(adapter) = factory.EnumAdapterByGpuPreference::<IDXGIAdapter1>(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE) {
                    if let Ok(desc) = adapter.GetDesc1() {
                        let name = String::from_utf16_lossy(&desc.Description[..desc.Description.iter().position(|&c| c == 0).unwrap_or(128)]);
                        println!("[GPU] Detected: {}", name);
                        println!("[GPU] VRAM: {} MB\n", desc.DedicatedVideoMemory / (1024 * 1024));
                    }
                }
            }
        }

        let attrs = Window::default_attributes()
            .with_title("EPICX Cube")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        let win = Rc::new(el.create_window(attrs).unwrap());
        let sz = win.inner_size();
        
        let ctx = Context::new(win.clone()).unwrap();
        let surf = Surface::new(&ctx, win.clone()).unwrap();

        self.ren.resize(sz.width, sz.height);
        self.start = Instant::now();
        
        println!("[INIT] Window: {}x{}", sz.width, sz.height);
        println!("[INIT] Rendering at maximum speed (no VSync)!\n");

        self.win = Some(win);
        self.surf = Some(surf);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => {
                let elapsed = self.start.elapsed().as_secs_f32();
                println!("\n[STATS] Total frames: {}", self.total_frames);
                println!("[STATS] Total time: {:.2}s", elapsed);
                println!("[STATS] Average FPS: {:.1}", self.total_frames as f32 / elapsed);
                el.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) {
                    let elapsed = self.start.elapsed().as_secs_f32();
                    println!("\n[STATS] Total frames: {}", self.total_frames);
                    println!("[STATS] Total time: {:.2}s", elapsed);
                    println!("[STATS] Average FPS: {:.1}", self.total_frames as f32 / elapsed);
                    el.exit();
                }
            }
            WindowEvent::Resized(sz) => {
                if sz.width > 0 && sz.height > 0 {
                    self.ren.resize(sz.width, sz.height);
                }
            }
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(w) = &self.win { w.request_redraw(); }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let el = EventLoop::new()?;
    el.set_control_flow(ControlFlow::Poll); // Maximum speed, no waiting
    el.run_app(&mut App::new())?;
    Ok(())
}
