//! Element system for EPICX - the virtual DOM equivalent

use crate::math::{Color, Rect, Transform};
use crate::core::ComponentId;
use std::collections::HashMap;
use std::sync::Arc;

/// Element types that can be rendered
#[derive(Debug, Clone)]
pub enum ElementType {
    /// Empty element (renders nothing)
    Empty,
    /// A rectangle shape
    Rect,
    /// A circle shape
    Circle,
    /// A line
    Line,
    /// A path (complex shape)
    Path,
    /// Text element
    Text,
    /// Image/texture element
    Image,
    /// 3D mesh element
    Mesh,
    /// Container for grouping elements
    Group,
    /// Custom element type
    Custom(String),
}

/// Style properties for elements
#[derive(Debug, Clone, Default)]
pub struct Style {
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub stroke_width: f32,
    pub opacity: f32,
    pub transform: Transform,
    pub z_index: i32,
    pub visible: bool,
    pub clip: Option<Rect>,
}

impl Style {
    pub fn new() -> Self {
        Self {
            opacity: 1.0,
            visible: true,
            ..Default::default()
        }
    }

    pub fn with_fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn with_stroke(mut self, color: Color, width: f32) -> Self {
        self.stroke = Some(color);
        self.stroke_width = width;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}

/// An Element in the render tree (similar to React's virtual DOM)
#[derive(Debug, Clone)]
pub struct Element {
    /// Unique key for reconciliation
    pub key: Option<String>,
    /// The type of element
    pub element_type: ElementType,
    /// Bounding rectangle
    pub bounds: Rect,
    /// Style properties
    pub style: Style,
    /// Child elements
    pub children: Vec<Element>,
    /// Associated component ID (if any)
    pub component_id: Option<ComponentId>,
    /// Custom attributes
    pub attributes: HashMap<String, AttributeValue>,
}

/// Attribute values for elements
#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Bool(bool),
    Color(Color),
    Rect(Rect),
    Array(Vec<AttributeValue>),
}

impl Element {
    /// Create a new empty element
    pub fn empty() -> Self {
        Self {
            key: None,
            element_type: ElementType::Empty,
            bounds: Rect::zero(),
            style: Style::new(),
            children: Vec::new(),
            component_id: None,
            attributes: HashMap::new(),
        }
    }

    /// Create a rectangle element
    pub fn rect(bounds: Rect) -> Self {
        Self {
            element_type: ElementType::Rect,
            bounds,
            ..Self::empty()
        }
    }

    /// Create a circle element
    pub fn circle(center_x: f32, center_y: f32, radius: f32) -> Self {
        Self {
            element_type: ElementType::Circle,
            bounds: Rect::new(center_x - radius, center_y - radius, radius * 2.0, radius * 2.0),
            ..Self::empty()
        }
    }

    /// Create a text element
    pub fn text(content: impl Into<String>, x: f32, y: f32) -> Self {
        let mut element = Self {
            element_type: ElementType::Text,
            bounds: Rect::new(x, y, 0.0, 0.0),
            ..Self::empty()
        };
        element.attributes.insert("content".to_string(), AttributeValue::String(content.into()));
        element
    }

    /// Create a group element
    pub fn group(children: Vec<Element>) -> Self {
        Self {
            element_type: ElementType::Group,
            children,
            ..Self::empty()
        }
    }

    /// Create an image element
    pub fn image(path: impl Into<String>, bounds: Rect) -> Self {
        let mut element = Self {
            element_type: ElementType::Image,
            bounds,
            ..Self::empty()
        };
        element.attributes.insert("path".to_string(), AttributeValue::String(path.into()));
        element
    }

    /// Set the element key
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set fill color
    pub fn fill(mut self, color: Color) -> Self {
        self.style.fill = Some(color);
        self
    }

    /// Set stroke
    pub fn stroke(mut self, color: Color, width: f32) -> Self {
        self.style.stroke = Some(color);
        self.style.stroke_width = width;
        self
    }

    /// Set opacity
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.style.opacity = opacity;
        self
    }

    /// Set transform
    pub fn transform(mut self, transform: Transform) -> Self {
        self.style.transform = transform;
        self
    }

    /// Add a child element
    pub fn child(mut self, child: Element) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn children(mut self, children: impl IntoIterator<Item = Element>) -> Self {
        self.children.extend(children);
        self
    }

    /// Set an attribute
    pub fn attr(mut self, key: impl Into<String>, value: AttributeValue) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Set z-index
    pub fn z_index(mut self, z: i32) -> Self {
        self.style.z_index = z;
        self
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.style.visible = visible;
        self
    }
}

/// Builder pattern for creating elements
pub struct ElementBuilder {
    element: Element,
}

impl ElementBuilder {
    pub fn new(element_type: ElementType) -> Self {
        Self {
            element: Element {
                element_type,
                ..Element::empty()
            },
        }
    }

    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.element.bounds = bounds;
        self
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.element.key = Some(key.into());
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.element.style = style;
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.element.style.fill = Some(color);
        self
    }

    pub fn stroke(mut self, color: Color, width: f32) -> Self {
        self.element.style.stroke = Some(color);
        self.element.style.stroke_width = width;
        self
    }

    pub fn child(mut self, child: Element) -> Self {
        self.element.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Element>) -> Self {
        self.element.children.extend(children);
        self
    }

    pub fn build(self) -> Element {
        self.element
    }
}

/// Fragment - a way to return multiple elements without a wrapper
pub fn fragment(children: Vec<Element>) -> Element {
    Element::group(children)
}

/// Conditional rendering helper
pub fn when(condition: bool, element: impl FnOnce() -> Element) -> Element {
    if condition {
        element()
    } else {
        Element::empty()
    }
}

/// Map helper for rendering lists
pub fn map<T, F>(items: impl IntoIterator<Item = T>, f: F) -> Vec<Element>
where
    F: Fn(T, usize) -> Element,
{
    items.into_iter().enumerate().map(|(i, item)| f(item, i)).collect()
}
