//! Text component

use crate::core::{Element, RenderContext, Props};
use crate::math::Color;

/// Text props
#[derive(Debug, Clone)]
pub struct TextProps {
    pub content: String,
    pub x: f32,
    pub y: f32,
    pub color: Color,
    pub font_size: f32,
}

impl Default for TextProps {
    fn default() -> Self {
        Self {
            content: String::new(),
            x: 0.0,
            y: 0.0,
            color: Color::WHITE,
            font_size: 16.0,
        }
    }
}

impl Props for TextProps {
    fn props_eq(&self, other: &Self) -> bool {
        self.content == other.content
            && self.x == other.x
            && self.y == other.y
            && self.font_size == other.font_size
    }
}

/// Text component
pub struct Text {
    props: TextProps,
}

impl Text {
    pub fn new(props: TextProps) -> Self {
        Self { props }
    }

    pub fn from_str(content: &str, x: f32, y: f32) -> Self {
        Self::new(TextProps {
            content: content.to_string(),
            x,
            y,
            ..Default::default()
        })
    }

    pub fn render(&self, _ctx: &mut RenderContext) -> Element {
        Element::text(&self.props.content, self.props.x, self.props.y)
    }
}
