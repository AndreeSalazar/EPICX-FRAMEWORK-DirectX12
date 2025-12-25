//! Context system for EPICX - similar to React Context

use crate::math::{Color, Rect, Vec2};
use crate::dx12::Device;
use parking_lot::RwLock;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Application-wide context that can be accessed by any component
pub struct Context {
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Provide a value to the context
    pub fn provide<T: Any + Send + Sync + 'static>(&mut self, value: T) {
        self.values.insert(TypeId::of::<T>(), Arc::new(value));
    }

    /// Get a value from the context
    pub fn get<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.values
            .get(&TypeId::of::<T>())
            .and_then(|v| v.clone().downcast::<T>().ok())
    }

    /// Check if a value exists in the context
    pub fn has<T: Any + Send + Sync + 'static>(&self) -> bool {
        self.values.contains_key(&TypeId::of::<T>())
    }

    /// Remove a value from the context
    pub fn remove<T: Any + Send + Sync + 'static>(&mut self) -> Option<Arc<T>> {
        self.values
            .remove(&TypeId::of::<T>())
            .and_then(|v| v.downcast::<T>().ok())
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// Render context passed to components during rendering
pub struct RenderContext<'a> {
    /// The application context
    pub context: &'a Context,
    /// Current viewport size
    pub viewport: Rect,
    /// Current mouse position
    pub mouse_position: Vec2,
    /// Delta time since last frame
    pub delta_time: f32,
    /// Total elapsed time
    pub elapsed_time: f32,
    /// Frame number
    pub frame: u64,
    /// Device reference for GPU operations
    device: Option<&'a Device>,
}

impl<'a> RenderContext<'a> {
    /// Create a new render context
    pub fn new(context: &'a Context, viewport: Rect) -> Self {
        Self {
            context,
            viewport,
            mouse_position: Vec2::ZERO,
            delta_time: 0.0,
            elapsed_time: 0.0,
            frame: 0,
            device: None,
        }
    }

    /// Set the device reference
    pub fn with_device(mut self, device: &'a Device) -> Self {
        self.device = Some(device);
        self
    }

    /// Get the device reference
    pub fn device(&self) -> Option<&Device> {
        self.device
    }

    /// Get a value from the context
    pub fn use_context<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.context.get::<T>()
    }

    /// Get the viewport width
    pub fn width(&self) -> f32 {
        self.viewport.width
    }

    /// Get the viewport height
    pub fn height(&self) -> f32 {
        self.viewport.height
    }

    /// Get the viewport aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        self.viewport.width / self.viewport.height
    }

    /// Check if a point is within the viewport
    pub fn is_in_viewport(&self, x: f32, y: f32) -> bool {
        self.viewport.contains(Vec2::new(x, y))
    }
}

/// Theme context for styling
#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub surface: Color,
    pub error: Color,
    pub on_primary: Color,
    pub on_secondary: Color,
    pub on_background: Color,
    pub on_surface: Color,
    pub on_error: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::from_hex(0x6200EE),
            secondary: Color::from_hex(0x03DAC6),
            background: Color::from_hex(0x121212),
            surface: Color::from_hex(0x1E1E1E),
            error: Color::from_hex(0xCF6679),
            on_primary: Color::WHITE,
            on_secondary: Color::BLACK,
            on_background: Color::WHITE,
            on_surface: Color::WHITE,
            on_error: Color::BLACK,
        }
    }
}

impl Theme {
    /// Create a light theme
    pub fn light() -> Self {
        Self {
            primary: Color::from_hex(0x6200EE),
            secondary: Color::from_hex(0x03DAC6),
            background: Color::WHITE,
            surface: Color::from_hex(0xF5F5F5),
            error: Color::from_hex(0xB00020),
            on_primary: Color::WHITE,
            on_secondary: Color::BLACK,
            on_background: Color::BLACK,
            on_surface: Color::BLACK,
            on_error: Color::WHITE,
        }
    }

    /// Create a dark theme
    pub fn dark() -> Self {
        Self::default()
    }
}
