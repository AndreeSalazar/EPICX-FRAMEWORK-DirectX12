//! Math utilities for EPICX
//!
//! Provides common math types and operations for graphics programming.

mod color;
mod rect;
mod transform;

pub use color::Color;
pub use rect::Rect;
pub use transform::Transform;

pub use glam::{Vec2, Vec3, Vec4, Mat4, Quat};
