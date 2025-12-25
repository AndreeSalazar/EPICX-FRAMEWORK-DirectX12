//! Canvas component for custom drawing

use crate::core::{Element, RenderContext, Props};
use crate::math::{Color, Rect};
use std::sync::Arc;

/// Canvas props
#[derive(Clone)]
pub struct CanvasProps {
    pub bounds: Rect,
    pub background: Option<Color>,
    pub draw_fn: Option<Arc<dyn Fn(&mut RenderContext) -> Vec<Element> + Send + Sync>>,
}

impl std::fmt::Debug for CanvasProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanvasProps")
            .field("bounds", &self.bounds)
            .field("background", &self.background)
            .finish()
    }
}

impl Default for CanvasProps {
    fn default() -> Self {
        Self {
            bounds: Rect::zero(),
            background: None,
            draw_fn: None,
        }
    }
}

impl Props for CanvasProps {
    fn props_eq(&self, other: &Self) -> bool {
        self.bounds == other.bounds && self.background == other.background
    }
}

/// Canvas component for custom drawing
pub struct Canvas {
    props: CanvasProps,
}

impl Canvas {
    pub fn new(props: CanvasProps) -> Self {
        Self { props }
    }

    pub fn with_draw<F>(bounds: Rect, draw_fn: F) -> Self
    where
        F: Fn(&mut RenderContext) -> Vec<Element> + Send + Sync + 'static,
    {
        Self::new(CanvasProps {
            bounds,
            draw_fn: Some(Arc::new(draw_fn)),
            ..Default::default()
        })
    }

    pub fn render(&self, ctx: &mut RenderContext) -> Element {
        let mut canvas = Element::rect(self.props.bounds);

        if let Some(bg) = self.props.background {
            canvas = canvas.fill(bg);
        }

        if let Some(ref draw_fn) = self.props.draw_fn {
            let children = draw_fn(ctx);
            canvas = canvas.children(children);
        }

        canvas
    }
}
