//! Real-time 3D Cube with DirectX12 Window
//! 
//! Demonstrates EPICX DirectX12 rendering:
//! - Window creation with winit
//! - DirectX12 device, swap chain, command queue
//! - Real-time rendering loop
//! - Animated clear color
//!
//! Run with: cargo run --example cube_window

use epicx::dx12::{Device, CommandQueue, SwapChain, SwapChainConfig, CommandAllocator, CommandList};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D12::*;

struct App {
    window: Option<Window>,
    device: Option<Device>,
    command_queue: Option<CommandQueue>,
    swap_chain: Option<SwapChain>,
    allocator: Option<CommandAllocator>,
    start_time: Instant,
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
            allocator: None,
            start_time: Instant::now(),
            frame_count: 0,
            last_fps_time: Instant::now(),
            fps: 0.0,
        }
    }
    
    fn render(&mut self) {
        let Some(device) = &self.device else { return };
        let Some(command_queue) = &mut self.command_queue else { return };
        let Some(swap_chain) = &mut self.swap_chain else { return };
        let Some(allocator) = &self.allocator else { return };
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
                "EPICX - Rotating Cube | FPS: {:.1} | Time: {:.1}s",
                self.fps, elapsed
            ));
        }
        
        // Animated clear color
        let clear_color = [
            0.1 + 0.1 * (elapsed * 0.5).sin(),
            0.15 + 0.1 * (elapsed * 0.7).cos(),
            0.2 + 0.1 * (elapsed * 0.3).sin(),
            1.0,
        ];
        
        // Reset allocator
        if let Err(e) = allocator.reset() {
            eprintln!("[EPICX] Allocator reset error: {:?}", e);
            return;
        }
        
        // Create command list
        let cmd_list = match CommandList::new(device, allocator, None) {
            Ok(list) => list,
            Err(e) => {
                eprintln!("[EPICX] Command list error: {:?}", e);
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
            
            // Clear render target
            cmd_list.raw().ClearRenderTargetView(rtv, &clear_color, None);
            
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
            eprintln!("[EPICX] Command list close error: {:?}", e);
            return;
        }
        
        command_queue.execute(&[&cmd_list]);
        
        // Present
        if let Err(e) = swap_chain.present() {
            eprintln!("[EPICX] Present error: {:?}", e);
            return;
        }
        
        // Wait for GPU
        if let Err(e) = command_queue.flush() {
            eprintln!("[EPICX] Flush error: {:?}", e);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║      EPICX - DirectX12 Real-time Renderer                    ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Controls:                                                   ║");
        println!("║  - ESC: Exit                                                 ║");
        println!("║  - Animated background color                                 ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        
        // Create window
        let window_attrs = Window::default_attributes()
            .with_title("EPICX - Rotating Cube")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
        
        let window = event_loop.create_window(window_attrs).expect("Failed to create window");
        let size = window.inner_size();
        
        // Get HWND
        let hwnd = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get() as *mut std::ffi::c_void),
            _ => panic!("Unsupported platform"),
        };
        
        // Initialize DirectX12
        println!("[EPICX] Initializing DirectX12...");
        let device = Device::new(true).expect("Failed to create device");
        println!("[EPICX] Device created");
        
        let command_queue = CommandQueue::graphics(&device).expect("Failed to create command queue");
        println!("[EPICX] Command queue created");
        
        let config = SwapChainConfig {
            width: size.width,
            height: size.height,
            buffer_count: 2,
            ..Default::default()
        };
        
        let swap_chain = SwapChain::new(&device, &command_queue, hwnd, config)
            .expect("Failed to create swap chain");
        println!("[EPICX] Swap chain created ({}x{})", size.width, size.height);
        
        let allocator = CommandAllocator::new(&device, D3D12_COMMAND_LIST_TYPE_DIRECT)
            .expect("Failed to create allocator");
        println!("[EPICX] Command allocator created");
        
        println!("[EPICX] Starting render loop...\n");
        
        self.window = Some(window);
        self.device = Some(device);
        self.command_queue = Some(command_queue);
        self.swap_chain = Some(swap_chain);
        self.allocator = Some(allocator);
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
                    if let (Some(device), Some(swap_chain)) = (&self.device, &mut self.swap_chain) {
                        if let Some(queue) = &mut self.command_queue {
                            let _ = queue.flush();
                        }
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
