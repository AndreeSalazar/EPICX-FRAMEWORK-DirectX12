//! Image component

use crate::core::{Element, RenderContext, Props};
use crate::math::Rect;

/// Image props
#[derive(Debug, Clone)]
pub struct ImageProps {
    pub source: String,
    pub bounds: Rect,
    pub opacity: f32,
}

impl Default for ImageProps {
    fn default() -> Self {
        Self {
            source: String::new(),
            bounds: Rect::zero(),
            opacity: 1.0,
        }
    }
}

impl Props for ImageProps {
    fn props_eq(&self, other: &Self) -> bool {
        self.source == other.source && self.bounds == other.bounds
    }
}

/// Image component
pub struct Image {
    props: ImageProps,
}

impl Image {
    pub fn new(props: ImageProps) -> Self {
        Self { props }
    }

    pub fn from_path(path: &str, bounds: Rect) -> Self {
        Self::new(ImageProps {
            source: path.to_string(),
            bounds,
            ..Default::default()
        })
    }

    pub fn render(&self, _ctx: &mut RenderContext) -> Element {
        Element::image(&self.props.source, self.props.bounds)
            .opacity(self.props.opacity)
    }
}
