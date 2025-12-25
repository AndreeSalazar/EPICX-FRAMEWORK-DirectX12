//! Easy API Demo
//!
//! Demonstrates the simplest way to use EPICX with the Level C (easy) API.

use epicx::easy::{EasyApp, Sprite, Timer};
use epicx::math::Color;

fn main() {
    println!("EPICX Easy API Demo");
    println!("==================");
    
    // Create a simple app
    let mut app = EasyApp::new("Easy Demo", 800, 600);
    
    // Create some sprites
    let mut player = Sprite::new(400.0, 300.0, 50.0, 50.0);
    player.color = Color::from_hex(0x4CAF50);
    
    let mut enemy = Sprite::new(200.0, 200.0, 40.0, 40.0);
    enemy.color = Color::from_hex(0xF44336);
    
    // Create a timer
    let mut _timer = Timer::repeating(1.0);
    
    // Simple run with just draw callback
    let result = app.run(|ctx| {
        // Clear screen
        ctx.clear(Color::from_hex(0x1a1a2e));
        
        // Draw some shapes
        ctx.fill_rect(50.0, 50.0, 100.0, 100.0, Color::from_hex(0x533483));
        ctx.draw_rect(200.0, 50.0, 100.0, 100.0, Color::from_hex(0xe94560));
        
        // Draw circles
        ctx.fill_circle(400.0, 100.0, 40.0, Color::from_hex(0x16c79a));
        ctx.draw_circle(500.0, 100.0, 40.0, Color::WHITE);
        
        // Draw lines
        ctx.draw_line(50.0, 200.0, 200.0, 250.0, Color::YELLOW);
        ctx.draw_line_thick(250.0, 200.0, 400.0, 250.0, Color::CYAN, 3.0);
        
        // Draw text
        ctx.draw_text("EPICX Easy API Demo", 50.0, 400.0);
        ctx.draw_text_colored("Colored Text!", 50.0, 430.0, Color::from_hex(0xff6b6b));
        ctx.draw_text_styled("Large Text", 50.0, 470.0, Color::WHITE, 24.0);
        
        // Draw player and enemy as rectangles
        ctx.fill_rect(player.x, player.y, player.width, player.height, player.color);
        ctx.fill_rect(enemy.x, enemy.y, enemy.width, enemy.height, enemy.color);
    });
    
    match result {
        Ok(()) => println!("App finished successfully!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
