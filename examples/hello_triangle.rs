//! Hello Triangle Example
//!
//! A simple example demonstrating how to render a triangle using EPICX.

use epicx::prelude::*;
use std::any::Any;

fn main() {
    env_logger::init();
    
    let app = App::builder()
        .title("EPICX - Hello Triangle")
        .size(800, 600)
        .clear_color(Color::from_hex(0x1a1a2e))
        .build();

    if let Err(e) = app.run(|| HelloTriangle::new(())) {
        eprintln!("Error: {}", e);
    }
}

/// Simple triangle component
struct HelloTriangle {
    rotation: f32,
}

impl Component for HelloTriangle {
    type Props = ();
    type State = f32; // rotation angle

    fn new(_props: Self::Props) -> Self {
        Self { rotation: 0.0 }
    }

    fn props(&self) -> &Self::Props {
        &()
    }

    fn state(&self) -> &Self::State {
        &self.rotation
    }

    fn state_mut(&mut self) -> &mut Self::State {
        &mut self.rotation
    }

    fn set_state<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut Self::State),
    {
        updater(&mut self.rotation);
    }

    fn render(&self, ctx: &mut RenderContext) -> Element {
        let center_x = ctx.width() / 2.0;
        let center_y = ctx.height() / 2.0;
        let size = 100.0;

        // Create a simple triangle using three lines
        // In a full implementation, this would use actual triangle primitives
        Element::group(vec![
            // Background
            Element::rect(ctx.viewport).fill(Color::from_hex(0x1a1a2e)),
            
            // Triangle (represented as a colored rectangle for now)
            Element::rect(Rect::new(
                center_x - size / 2.0,
                center_y - size / 2.0,
                size,
                size,
            ))
            .fill(Color::from_hex(0xe94560))
            .transform(Transform::from_position(glam::Vec3::new(center_x, center_y, 0.0))),
            
            // Title text
            Element::text("Hello Triangle!", center_x - 60.0, 50.0),
            
            // FPS counter placeholder
            Element::text(
                format!("Frame: {}", ctx.frame),
                10.0,
                ctx.height() - 30.0,
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
