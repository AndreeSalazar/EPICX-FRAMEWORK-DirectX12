//! Props system for EPICX components

use std::any::Any;
use std::fmt::Debug;

/// Trait for component props
///
/// Props are immutable data passed from parent to child components.
pub trait Props: Clone + Debug + Send + Sync + 'static {
    /// Check if props are equal (for optimization)
    fn props_eq(&self, other: &Self) -> bool;
}

/// Default implementation for unit type (no props)
impl Props for () {
    fn props_eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Macro to derive Props for simple structs
#[macro_export]
macro_rules! derive_props {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $field: $ty),*
        }

        impl Props for $name {
            fn props_eq(&self, other: &Self) -> bool {
                $(self.$field == other.$field)&&*
            }
        }
    };
}

/// A dynamic props container for type-erased props
#[derive(Debug, Clone)]
pub struct DynamicProps {
    inner: Box<dyn DynamicPropsInner>,
}

trait DynamicPropsInner: Debug + Send + Sync {
    fn clone_box(&self) -> Box<dyn DynamicPropsInner>;
    fn as_any(&self) -> &dyn Any;
    fn props_eq(&self, other: &dyn DynamicPropsInner) -> bool;
}

impl<T: Props> DynamicPropsInner for T {
    fn clone_box(&self) -> Box<dyn DynamicPropsInner> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn props_eq(&self, other: &dyn DynamicPropsInner) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            Props::props_eq(self, other)
        } else {
            false
        }
    }
}

impl Clone for Box<dyn DynamicPropsInner> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl DynamicProps {
    /// Create new dynamic props from a Props value
    pub fn new<T: Props>(props: T) -> Self {
        Self {
            inner: Box::new(props),
        }
    }

    /// Try to downcast to a specific props type
    pub fn downcast_ref<T: Props>(&self) -> Option<&T> {
        self.inner.as_any().downcast_ref()
    }

    /// Check if props are equal
    pub fn props_eq(&self, other: &DynamicProps) -> bool {
        self.inner.props_eq(other.inner.as_ref())
    }
}
