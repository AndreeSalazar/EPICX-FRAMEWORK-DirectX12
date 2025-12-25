//! Built-in components for EPICX
//!
//! This module provides ready-to-use UI components inspired by React.

mod button;
mod container;
mod text_component;
mod image_component;
mod canvas;

pub use button::{Button, ButtonProps, ButtonState};
pub use container::{Container, ContainerProps, Flex, FlexDirection};
pub use text_component::{Text, TextProps};
pub use image_component::{Image, ImageProps};
pub use canvas::{Canvas, CanvasProps};

use crate::core::{Element, RenderContext};
use crate::math::{Color, Rect};

/// A simple box component
pub fn box_element(bounds: Rect, color: Color) -> Element {
    Element::rect(bounds).fill(color)
}

/// A horizontal line
pub fn h_line(x: f32, y: f32, width: f32, color: Color, thickness: f32) -> Element {
    Element::rect(Rect::new(x, y, width, thickness)).fill(color)
}

/// A vertical line
pub fn v_line(x: f32, y: f32, height: f32, color: Color, thickness: f32) -> Element {
    Element::rect(Rect::new(x, y, thickness, height)).fill(color)
}

/// Spacer component for layout
pub fn spacer(width: f32, height: f32) -> Element {
    Element::rect(Rect::new(0.0, 0.0, width, height))
}

/// Divider component
pub fn divider(horizontal: bool, size: f32, color: Color) -> Element {
    if horizontal {
        h_line(0.0, 0.0, size, color, 1.0)
    } else {
        v_line(0.0, 0.0, size, color, 1.0)
    }
}
