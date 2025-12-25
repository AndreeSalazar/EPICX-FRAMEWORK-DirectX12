//! Component trait - React-inspired component system

use crate::core::{Element, RenderContext, Props, State};
use std::any::Any;
use uuid::Uuid;

/// Unique identifier for a component instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub Uuid);

impl ComponentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self::new()
    }
}

/// Lifecycle events for components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    /// Component is about to mount
    WillMount,
    /// Component has mounted
    DidMount,
    /// Component is about to update
    WillUpdate,
    /// Component has updated
    DidUpdate,
    /// Component is about to unmount
    WillUnmount,
}

/// The core Component trait - similar to React components
///
/// Components are the building blocks of EPICX applications.
/// They encapsulate state, props, and rendering logic.
pub trait Component: Send + Sync + 'static {
    /// The props type for this component
    type Props: Props;
    
    /// The state type for this component
    type State: State;

    /// Create a new component with the given props
    fn new(props: Self::Props) -> Self where Self: Sized;

    /// Get the component's unique identifier
    fn id(&self) -> ComponentId {
        ComponentId::new()
    }

    /// Get the current props
    fn props(&self) -> &Self::Props;

    /// Get the current state
    fn state(&self) -> &Self::State;

    /// Get mutable access to state
    fn state_mut(&mut self) -> &mut Self::State;

    /// Set new state and trigger a re-render
    fn set_state<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut Self::State);

    /// Render the component to an Element tree
    fn render(&self, ctx: &mut RenderContext) -> Element;

    /// Called when the component is about to mount
    fn will_mount(&mut self) {}

    /// Called after the component has mounted
    fn did_mount(&mut self) {}

    /// Called before the component updates
    fn will_update(&mut self, _next_props: &Self::Props, _next_state: &Self::State) -> bool {
        true // Return true to allow update, false to skip
    }

    /// Called after the component has updated
    fn did_update(&mut self, _prev_props: &Self::Props, _prev_state: &Self::State) {}

    /// Called before the component unmounts
    fn will_unmount(&mut self) {}

    /// Handle lifecycle events
    fn on_lifecycle(&mut self, event: Lifecycle) {
        match event {
            Lifecycle::WillMount => self.will_mount(),
            Lifecycle::DidMount => self.did_mount(),
            Lifecycle::WillUpdate => {}
            Lifecycle::DidUpdate => {}
            Lifecycle::WillUnmount => self.will_unmount(),
        }
    }

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A boxed component for dynamic dispatch
pub type BoxedComponent = Box<dyn ComponentDyn>;

/// Dynamic component trait for type-erased components
pub trait ComponentDyn: Send + Sync {
    fn id(&self) -> ComponentId;
    fn render(&self, ctx: &mut RenderContext) -> Element;
    fn will_mount(&mut self);
    fn did_mount(&mut self);
    fn will_unmount(&mut self);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A simple functional component
pub struct FunctionalComponent<F>
where
    F: Fn(&mut RenderContext) -> Element + Send + Sync + 'static,
{
    id: ComponentId,
    render_fn: F,
}

impl<F> FunctionalComponent<F>
where
    F: Fn(&mut RenderContext) -> Element + Send + Sync + 'static,
{
    pub fn new(render_fn: F) -> Self {
        Self {
            id: ComponentId::new(),
            render_fn,
        }
    }

    pub fn render(&self, ctx: &mut RenderContext) -> Element {
        (self.render_fn)(ctx)
    }
}
