//! # EPICX - React-inspired DirectX12 Framework
//!
//! EPICX is a Rust framework that encapsulates DirectX12 with a React-inspired
//! component architecture for building high-performance graphics applications.
//!
//! ## Architecture Hierarchy
//! 
//! EPICX provides three levels of abstraction:
//! 
//! - **Level A (dx12)**: Raw DirectX12 wrappers - full control
//! - **Level B (graphics)**: Mid-level abstractions - cleaner API
//! - **Level C (easy)**: Simple high-level API - easiest to use
//!
//! ## Features
//! - **Component-based architecture**: Build UIs and graphics using composable components
//! - **Declarative rendering**: Describe what you want, not how to render it
//! - **State management**: Reactive state updates trigger efficient re-renders
//! - **DirectX12 abstraction**: Full DX12 power without the complexity
//! - **React-style hooks**: use_state, use_effect, use_memo, use_ref
//!
//! ## Quick Start (Easy API)
//! ```rust,no_run
//! use epicx::easy::*;
//!
//! fn main() {
//!     let mut app = EasyApp::new("My Game", 800, 600);
//!     app.run(|ctx| {
//!         ctx.clear(Color::CORNFLOWER_BLUE);
//!         ctx.fill_rect(100.0, 100.0, 200.0, 150.0, Color::RED);
//!     });
//! }
//! ```
//!
//! ## Component-based (React-style)
//! ```rust,no_run
//! use epicx::prelude::*;
//!
//! struct MyApp {
//!     color: Color,
//! }
//!
//! // Implement Component trait for React-like behavior
//! ```

// Level A: Raw DirectX12 wrappers
pub mod dx12;

// Level B: Mid-level graphics abstractions
pub mod graphics;

// Level C: Simple high-level API
pub mod easy;

// Core component system (React-inspired)
pub mod core;

// Built-in components
pub mod components;

// Rendering system
pub mod renderer;

// Math utilities
pub mod math;

// Window management
pub mod window;

// Event system
pub mod events;

// React-style hooks
pub mod hooks;

// ADead-GPU Language (.gpu files)
pub mod lang;

// ADead-Vector3D: SDF primitives and operations
pub mod sdf;

// ADead-ISR: Intelligent Shading Rate
pub mod isr;

/// Prelude - commonly used types for component-based development
pub mod prelude {
    // Core types
    pub use crate::core::{
        App, AppBuilder, Component, Element, ElementBuilder,
        Props, State, Context, RenderContext,
    };
    
    // DirectX12 types
    pub use crate::dx12::{Device, CommandQueue, SwapChain, Pipeline};
    
    // Built-in components
    pub use crate::components::*;
    
    // Renderer
    pub use crate::renderer::{Renderer, RenderPass};
    
    // Math types
    pub use crate::math::{Color, Rect, Vec2, Vec3, Vec4, Mat4, Transform};
    
    // Window
    pub use crate::window::{Window, WindowConfig};
    
    // Events
    pub use crate::events::{Event, EventHandler, MouseEvent, KeyEvent};
    
    // Hooks
    pub use crate::hooks::{use_state, use_effect, use_memo, use_ref};
    
    // Graphics (Level B)
    pub use crate::graphics::{Graphics, GraphicsConfig, GraphicsContext};
}

pub use prelude::*;
