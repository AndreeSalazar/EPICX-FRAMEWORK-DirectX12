//! Button component

use crate::core::{Element, RenderContext, Props, State};
use crate::math::{Color, Rect};

/// Button props
#[derive(Debug, Clone)]
pub struct ButtonProps {
    pub label: String,
    pub bounds: Rect,
    pub background: Color,
    pub hover_background: Color,
    pub pressed_background: Color,
    pub text_color: Color,
    pub disabled: bool,
}

impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            label: "Button".to_string(),
            bounds: Rect::new(0.0, 0.0, 100.0, 40.0),
            background: Color::from_hex(0x4A90D9),
            hover_background: Color::from_hex(0x5BA0E9),
            pressed_background: Color::from_hex(0x3A80C9),
            text_color: Color::WHITE,
            disabled: false,
        }
    }
}

impl Props for ButtonProps {
    fn props_eq(&self, other: &Self) -> bool {
        self.label == other.label
            && self.bounds == other.bounds
            && self.disabled == other.disabled
    }
}

/// Button state
#[derive(Debug, Clone, Default)]
pub struct ButtonState {
    pub hovered: bool,
    pub pressed: bool,
}

impl State for ButtonState {}

/// Button component
pub struct Button {
    props: ButtonProps,
    state: ButtonState,
}

impl Button {
    pub fn new(props: ButtonProps) -> Self {
        Self {
            props,
            state: ButtonState::default(),
        }
    }

    pub fn render(&self, _ctx: &mut RenderContext) -> Element {
        let bg_color = if self.props.disabled {
            Color::from_hex(0x888888)
        } else if self.state.pressed {
            self.props.pressed_background
        } else if self.state.hovered {
            self.props.hover_background
        } else {
            self.props.background
        };

        Element::rect(self.props.bounds)
            .fill(bg_color)
            .child(
                Element::text(&self.props.label, self.props.bounds.x + 10.0, self.props.bounds.y + 10.0)
            )
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.state.hovered = hovered;
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.state.pressed = pressed;
    }
}
