//! Core module for EPICX
//!
//! Contains the fundamental types and traits for the component-based architecture.

mod app;
mod component;
pub mod element;
mod context;
mod state;
mod props;

pub use app::{App, AppBuilder};
pub use component::{Component, ComponentId, ComponentDyn, BoxedComponent, FunctionalComponent, Lifecycle};
pub use element::{Element, ElementBuilder, ElementType, Style, AttributeValue, fragment, when, map};
pub use context::{Context, RenderContext, Theme};
pub use state::{State, ReactiveState, Atom};
pub use props::{Props, DynamicProps};
