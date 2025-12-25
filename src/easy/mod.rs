//! Easy API (Level C) - Simplified high-level graphics API
//!
//! This module provides the simplest possible API for graphics programming.
//! It inherits from graphics (Level B) which inherits from dx12 (Level A).
//!
//! Hierarchy:
//! - Level A (dx12): Raw DirectX12 wrappers
//! - Level B (graphics): Mid-level abstractions  
//! - Level C (easy): This module - simple, high-level API
//!
//! Example usage:
//! ```rust
//! use epicx::easy::*;
//!
//! fn main() {
//!     let mut app = EasyApp::new("My Game", 800, 600);
//!     
//!     app.run(|ctx| {
//!         ctx.clear(Color::CORNFLOWER_BLUE);
//!         ctx.draw_rect(100.0, 100.0, 200.0, 150.0, Color::RED);
//!         ctx.draw_text("Hello World!", 50.0, 50.0);
//!     });
//! }
//! ```

use crate::graphics::{Graphics, GraphicsConfig};
use crate::math::{Color, Rect, Vec2};
use crate::dx12::Dx12Result;

/// Simple 2D drawing context
pub struct DrawContext {
    width: f32,
    height: f32,
    clear_color: Color,
    commands: Vec<DrawCommand>,
}

/// Drawing commands
#[derive(Debug, Clone)]
pub enum DrawCommand {
    Clear(Color),
    Rect { x: f32, y: f32, width: f32, height: f32, color: Color },
    FilledRect { x: f32, y: f32, width: f32, height: f32, color: Color },
    Circle { x: f32, y: f32, radius: f32, color: Color },
    FilledCircle { x: f32, y: f32, radius: f32, color: Color },
    Line { x1: f32, y1: f32, x2: f32, y2: f32, color: Color, thickness: f32 },
    Text { text: String, x: f32, y: f32, color: Color, size: f32 },
    Image { path: String, x: f32, y: f32, width: f32, height: f32 },
}

impl DrawContext {
    /// Create a new draw context
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            clear_color: Color::BLACK,
            commands: Vec::new(),
        }
    }

    /// Get the screen width
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Get the screen height
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Get the screen size as Vec2
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    /// Get the screen bounds as Rect
    pub fn bounds(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width, self.height)
    }

    /// Clear the screen with a color
    pub fn clear(&mut self, color: Color) {
        self.clear_color = color;
        self.commands.push(DrawCommand::Clear(color));
    }

    /// Draw a rectangle outline
    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.commands.push(DrawCommand::Rect { x, y, width, height, color });
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.commands.push(DrawCommand::FilledRect { x, y, width, height, color });
    }

    /// Draw a circle outline
    pub fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        self.commands.push(DrawCommand::Circle { x, y, radius, color });
    }

    /// Draw a filled circle
    pub fn fill_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        self.commands.push(DrawCommand::FilledCircle { x, y, radius, color });
    }

    /// Draw a line
    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color) {
        self.commands.push(DrawCommand::Line { x1, y1, x2, y2, color, thickness: 1.0 });
    }

    /// Draw a line with thickness
    pub fn draw_line_thick(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color, thickness: f32) {
        self.commands.push(DrawCommand::Line { x1, y1, x2, y2, color, thickness });
    }

    /// Draw text
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32) {
        self.commands.push(DrawCommand::Text {
            text: text.to_string(),
            x,
            y,
            color: Color::WHITE,
            size: 16.0,
        });
    }

    /// Draw text with color
    pub fn draw_text_colored(&mut self, text: &str, x: f32, y: f32, color: Color) {
        self.commands.push(DrawCommand::Text {
            text: text.to_string(),
            x,
            y,
            color,
            size: 16.0,
        });
    }

    /// Draw text with color and size
    pub fn draw_text_styled(&mut self, text: &str, x: f32, y: f32, color: Color, size: f32) {
        self.commands.push(DrawCommand::Text {
            text: text.to_string(),
            x,
            y,
            color,
            size,
        });
    }

    /// Draw an image
    pub fn draw_image(&mut self, path: &str, x: f32, y: f32, width: f32, height: f32) {
        self.commands.push(DrawCommand::Image {
            path: path.to_string(),
            x,
            y,
            width,
            height,
        });
    }

    /// Get all draw commands
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Clear all commands
    pub fn reset(&mut self) {
        self.commands.clear();
    }
}

/// Easy application - the simplest way to create a graphics app
pub struct EasyApp {
    title: String,
    width: u32,
    height: u32,
    graphics: Option<Graphics>,
    running: bool,
    frame_count: u64,
}

impl EasyApp {
    /// Create a new easy application
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        Self {
            title: title.to_string(),
            width,
            height,
            graphics: None,
            running: false,
            frame_count: 0,
        }
    }

    /// Initialize the graphics system (requires HWND from window)
    pub fn init_with_hwnd(&mut self, hwnd: windows::Win32::Foundation::HWND) -> Dx12Result<()> {
        let config = GraphicsConfig {
            width: self.width,
            height: self.height,
            debug: cfg!(debug_assertions),
            ..Default::default()
        };

        self.graphics = Some(Graphics::new(hwnd, config)?);
        self.running = true;
        Ok(())
    }

    /// Check if the app is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stop the application
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Get the current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Run the application with a draw callback
    /// Note: Requires window HWND - use run_with_window instead
    pub fn run_loop<F>(&mut self, mut draw_fn: F)
    where
        F: FnMut(&mut DrawContext),
    {
        while self.running {
            let mut ctx = DrawContext::new(self.width as f32, self.height as f32);
            draw_fn(&mut ctx);
            self.frame_count += 1;

            if self.frame_count > 1000 {
                self.running = false;
            }
        }
    }
    
    /// Get mutable reference to graphics (if initialized)
    pub fn graphics_mut(&mut self) -> Option<&mut Graphics> {
        self.graphics.as_mut()
    }
}

/// Quick function to run a simple graphics app (placeholder)
pub fn run_app<F>(title: &str, width: u32, height: u32, mut draw_fn: F) -> Dx12Result<()>
where
    F: FnMut(&mut DrawContext),
{
    let mut app = EasyApp::new(title, width, height);
    app.run_loop(draw_fn);
    Ok(())
}

/// Sprite for easy 2D rendering
#[derive(Debug, Clone)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub scale: f32,
    pub color: Color,
    pub visible: bool,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
            rotation: 0.0,
            scale: 1.0,
            color: Color::WHITE,
            visible: true,
        }
    }
}

impl Sprite {
    /// Create a new sprite
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            ..Default::default()
        }
    }

    /// Set position
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    /// Move by offset
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }

    /// Set rotation in radians
    pub fn set_rotation(&mut self, radians: f32) {
        self.rotation = radians;
    }

    /// Rotate by angle in radians
    pub fn rotate(&mut self, radians: f32) {
        self.rotation += radians;
    }

    /// Set scale
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    /// Get bounds as Rect
    pub fn bounds(&self) -> Rect {
        let w = self.width * self.scale;
        let h = self.height * self.scale;
        Rect::new(self.x - w / 2.0, self.y - h / 2.0, w, h)
    }

    /// Check collision with another sprite
    pub fn collides_with(&self, other: &Sprite) -> bool {
        self.bounds().intersects(&other.bounds())
    }

    /// Draw the sprite
    pub fn draw(&self, ctx: &mut DrawContext) {
        if self.visible {
            let bounds = self.bounds();
            ctx.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height, self.color);
        }
    }
}

/// Simple timer for game logic
#[derive(Debug, Clone)]
pub struct Timer {
    duration: f32,
    elapsed: f32,
    repeating: bool,
    finished: bool,
}

impl Timer {
    /// Create a new timer
    pub fn new(duration: f32, repeating: bool) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            repeating,
            finished: false,
        }
    }

    /// Create a one-shot timer
    pub fn once(duration: f32) -> Self {
        Self::new(duration, false)
    }

    /// Create a repeating timer
    pub fn repeating(duration: f32) -> Self {
        Self::new(duration, true)
    }

    /// Update the timer
    pub fn update(&mut self, delta: f32) -> bool {
        if self.finished && !self.repeating {
            return false;
        }

        self.elapsed += delta;

        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed -= self.duration;
            } else {
                self.finished = true;
            }
            return true;
        }

        false
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.finished = false;
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).min(1.0)
    }

    /// Check if finished
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}
