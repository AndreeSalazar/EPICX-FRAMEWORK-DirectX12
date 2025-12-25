//! Component Demo Example
//!
//! Demonstrates the React-like component system with state management.

use epicx::prelude::*;
use epicx::components::{Button, ButtonProps, Container, ContainerProps, FlexDirection};
use std::any::Any;

fn main() {
    env_logger::init();
    
    let app = App::builder()
        .title("EPICX - Component Demo")
        .size(1024, 768)
        .clear_color(Color::from_hex(0x0f0f23))
        .build();

    if let Err(e) = app.run(|| CounterApp::new(())) {
        eprintln!("Error: {}", e);
    }
}

/// Counter application state
#[derive(Debug, Clone, Default)]
struct CounterState {
    count: i32,
    clicks: u32,
}

impl epicx::core::State for CounterState {}

/// Counter application component
struct CounterApp {
    state: CounterState,
}

impl Component for CounterApp {
    type Props = ();
    type State = CounterState;

    fn new(_props: Self::Props) -> Self {
        Self {
            state: CounterState::default(),
        }
    }

    fn props(&self) -> &Self::Props {
        &()
    }

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn state_mut(&mut self) -> &mut Self::State {
        &mut self.state
    }

    fn set_state<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut Self::State),
    {
        updater(&mut self.state);
    }

    fn render(&self, ctx: &mut RenderContext) -> Element {
        let width = ctx.width();
        let height = ctx.height();

        // Main container
        Element::group(vec![
            // Background gradient (simulated with rectangles)
            Element::rect(Rect::new(0.0, 0.0, width, height / 2.0))
                .fill(Color::from_hex(0x1a1a2e)),
            Element::rect(Rect::new(0.0, height / 2.0, width, height / 2.0))
                .fill(Color::from_hex(0x16213e)),

            // Title
            Element::text("EPICX Counter Demo", width / 2.0 - 100.0, 50.0),

            // Counter display
            Element::rect(Rect::new(width / 2.0 - 150.0, height / 2.0 - 100.0, 300.0, 80.0))
                .fill(Color::from_hex(0x0f3460))
                .stroke(Color::from_hex(0xe94560), 2.0),
            
            Element::text(
                format!("Count: {}", self.state.count),
                width / 2.0 - 50.0,
                height / 2.0 - 70.0,
            ),

            // Buttons row
            self.render_buttons(ctx),

            // Stats
            Element::text(
                format!("Total clicks: {}", self.state.clicks),
                width / 2.0 - 60.0,
                height - 100.0,
            ),

            // Instructions
            Element::text(
                "Click the buttons to increment/decrement the counter",
                width / 2.0 - 180.0,
                height - 50.0,
            ),
        ])
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl CounterApp {
    fn render_buttons(&self, ctx: &mut RenderContext) -> Element {
        let center_x = ctx.width() / 2.0;
        let center_y = ctx.height() / 2.0;

        Element::group(vec![
            // Decrement button
            Element::rect(Rect::new(center_x - 140.0, center_y + 20.0, 80.0, 40.0))
                .fill(Color::from_hex(0xe94560))
                .with_key("btn-dec"),
            Element::text("-", center_x - 105.0, center_y + 30.0),

            // Reset button
            Element::rect(Rect::new(center_x - 40.0, center_y + 20.0, 80.0, 40.0))
                .fill(Color::from_hex(0x533483))
                .with_key("btn-reset"),
            Element::text("Reset", center_x - 25.0, center_y + 30.0),

            // Increment button
            Element::rect(Rect::new(center_x + 60.0, center_y + 20.0, 80.0, 40.0))
                .fill(Color::from_hex(0x16c79a))
                .with_key("btn-inc"),
            Element::text("+", center_x + 95.0, center_y + 30.0),
        ])
    }

    /// Increment the counter
    pub fn increment(&mut self) {
        self.set_state(|s| {
            s.count += 1;
            s.clicks += 1;
        });
    }

    /// Decrement the counter
    pub fn decrement(&mut self) {
        self.set_state(|s| {
            s.count -= 1;
            s.clicks += 1;
        });
    }

    /// Reset the counter
    pub fn reset(&mut self) {
        self.set_state(|s| {
            s.count = 0;
            s.clicks += 1;
        });
    }
}

/// Using hooks for state management (alternative approach)
fn counter_with_hooks() {
    // This demonstrates how hooks would be used
    let count = use_state(0);
    let clicks = use_state(0u32);

    // Increment
    let increment = {
        let count = count.clone();
        let clicks = clicks.clone();
        move || {
            count.set(count.get() + 1);
            clicks.set(clicks.get() + 1);
        }
    };

    // Effect on mount
    use_effect(|| {
        println!("Counter component mounted!");
    }, None);

    // Memoized value
    let _doubled = use_memo(|| count.get() * 2, Some(vec![count.get() as u64]));
}
