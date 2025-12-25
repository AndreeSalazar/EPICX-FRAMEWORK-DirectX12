//! Real-time 3D Cube with DirectX12 Window - SIMPLIFIED with Level B
//! 
//! Demonstrates EPICX hierarchy:
//! - Level A (dx12): Raw DirectX12 - NOT used directly here
//! - Level B (graphics): Graphics struct - USED HERE for simplicity
//! - Level C (easy): Even simpler API
//!
//! Run with: cargo run --example cube_window

use epicx::graphics::{Graphics, GraphicsConfig};
use epicx::math::Color;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::HWND;

/// Application state - MUCH SIMPLER with Level B!
struct App {
    window: Option<Window>,
    graphics: Option<Graphics>,  // Just ONE struct instead of 5!
    start_time: Instant,
    frame_count: u64,
    last_fps_time: Instant,
    fps: f32,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            graphics: None,
            start_time: Instant::now(),
            frame_count: 0,
            last_fps_time: Instant::now(),
            fps: 0.0,
        }
    }
    
    /// Render a frame - SIMPLIFIED!
    fn render(&mut self) {
        let Some(graphics) = &mut self.graphics else { return };
        let Some(window) = &self.window else { return };
        
        // Calculate time
        let elapsed = self.start_time.elapsed().as_secs_f32();
        
        // Calculate FPS
        self.frame_count += 1;
        let fps_elapsed = self.last_fps_time.elapsed().as_secs_f32();
        if fps_elapsed >= 1.0 {
            self.fps = self.frame_count as f32 / fps_elapsed;
            self.frame_count = 0;
            self.last_fps_time = Instant::now();
            
            window.set_title(&format!(
                "EPICX - Level B Demo | FPS: {:.1} | Time: {:.1}s",
                self.fps, elapsed
            ));
        }
        
        // Animated clear color
        let color = Color::new(
            0.1 + 0.1 * (elapsed * 0.5).sin(),
            0.15 + 0.1 * (elapsed * 0.7).cos(),
            0.2 + 0.1 * (elapsed * 0.3).sin(),
            1.0,
        );
        
        // BEGIN FRAME - Level B handles all the complexity!
        let frame = match graphics.begin_frame() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[EPICX] Begin frame error: {:?}", e);
                return;
            }
        };
        
        // CLEAR - Just one line!
        frame.clear(color);
        
        // END FRAME - Level B handles present, barriers, sync!
        if let Err(e) = graphics.end_frame(frame) {
            eprintln!("[EPICX] End frame error: {:?}", e);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║      EPICX - Level B Graphics Demo (SIMPLIFIED!)             ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  This example uses Level B (graphics module)                 ║");
        println!("║  which encapsulates ALL of Level A (dx12 module)             ║");
        println!("║                                                              ║");
        println!("║  Before: Device + CommandQueue + SwapChain + Allocator + ... ║");
        println!("║  After:  Just Graphics!                                      ║");
        println!("║                                                              ║");
        println!("║  Controls: ESC to exit                                       ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        
        // Create window
        let window_attrs = Window::default_attributes()
            .with_title("EPICX - Level B Demo")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
        
        let window = event_loop.create_window(window_attrs).expect("Failed to create window");
        let size = window.inner_size();
        
        // Get HWND
        let hwnd = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get() as *mut std::ffi::c_void),
            _ => panic!("Unsupported platform"),
        };
        
        // SIMPLIFIED: Just create Graphics with config!
        println!("[EPICX] Creating Graphics (Level B)...");
        let config = GraphicsConfig {
            width: size.width,
            height: size.height,
            ..Default::default()
        };
        
        let graphics = Graphics::new(hwnd, config).expect("Failed to create graphics");
        println!("[EPICX] Graphics created ({}x{}) - All DX12 resources encapsulated!", size.width, size.height);
        println!("[EPICX] Starting render loop...\n");
        
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
                    println!("[EPICX] Resizing to {}x{}", new_size.width, new_size.height);
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
