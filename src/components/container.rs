//! Container and layout components

use crate::core::{Element, RenderContext, Props};
use crate::math::{Color, Rect};

/// Flex direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

/// Container props
#[derive(Debug, Clone)]
pub struct ContainerProps {
    pub bounds: Rect,
    pub background: Option<Color>,
    pub padding: f32,
    pub gap: f32,
    pub direction: FlexDirection,
}

impl Default for ContainerProps {
    fn default() -> Self {
        Self {
            bounds: Rect::zero(),
            background: None,
            padding: 0.0,
            gap: 0.0,
            direction: FlexDirection::Row,
        }
    }
}

impl Props for ContainerProps {
    fn props_eq(&self, other: &Self) -> bool {
        self.bounds == other.bounds
            && self.background == other.background
            && self.padding == other.padding
            && self.gap == other.gap
            && self.direction == other.direction
    }
}

/// Container component
pub struct Container {
    props: ContainerProps,
    children: Vec<Element>,
}

impl Container {
    pub fn new(props: ContainerProps) -> Self {
        Self {
            props,
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<Element>) -> Self {
        self.children = children;
        self
    }

    pub fn add_child(&mut self, child: Element) {
        self.children.push(child);
    }

    pub fn render(&self, _ctx: &mut RenderContext) -> Element {
        let mut container = Element::rect(self.props.bounds);

        if let Some(bg) = self.props.background {
            container = container.fill(bg);
        }

        // Layout children based on direction
        let mut offset = self.props.padding;
        let laid_out_children: Vec<Element> = self.children.iter().map(|child| {
            let mut positioned = child.clone();
            match self.props.direction {
                FlexDirection::Row | FlexDirection::RowReverse => {
                    positioned.bounds.x = self.props.bounds.x + offset;
                    positioned.bounds.y = self.props.bounds.y + self.props.padding;
                    offset += positioned.bounds.width + self.props.gap;
                }
                FlexDirection::Column | FlexDirection::ColumnReverse => {
                    positioned.bounds.x = self.props.bounds.x + self.props.padding;
                    positioned.bounds.y = self.props.bounds.y + offset;
                    offset += positioned.bounds.height + self.props.gap;
                }
            }
            positioned
        }).collect();

        container.children(laid_out_children)
    }
}

/// Flex container helper
pub struct Flex;

impl Flex {
    /// Create a horizontal flex container
    pub fn row(bounds: Rect, gap: f32, children: Vec<Element>) -> Element {
        Container::new(ContainerProps {
            bounds,
            gap,
            direction: FlexDirection::Row,
            ..Default::default()
        })
        .with_children(children)
        .render(&mut RenderContext::new(&crate::core::Context::new(), bounds))
    }

    /// Create a vertical flex container
    pub fn column(bounds: Rect, gap: f32, children: Vec<Element>) -> Element {
        Container::new(ContainerProps {
            bounds,
            gap,
            direction: FlexDirection::Column,
            ..Default::default()
        })
        .with_children(children)
        .render(&mut RenderContext::new(&crate::core::Context::new(), bounds))
    }
}
