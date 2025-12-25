//! Event system for EPICX

use crate::math::Vec2;
use std::collections::VecDeque;

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

/// Key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Special keys
    Escape, Tab, CapsLock, Shift, Control, Alt, Space,
    Enter, Backspace, Delete, Insert, Home, End, PageUp, PageDown,
    Left, Right, Up, Down,
    // Other
    Unknown,
}

/// Mouse event data
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub position: Vec2,
    pub button: Option<MouseButton>,
    pub delta: Vec2,
    pub scroll_delta: f32,
}

impl Default for MouseEvent {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            button: None,
            delta: Vec2::ZERO,
            scroll_delta: 0.0,
        }
    }
}

/// Keyboard event data
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: KeyCode,
    pub pressed: bool,
    pub repeat: bool,
    pub modifiers: Modifiers,
}

/// Modifier keys state
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

/// Event types
#[derive(Debug, Clone)]
pub enum Event {
    // Window events
    WindowClose,
    WindowResize { width: u32, height: u32 },
    WindowFocus(bool),
    
    // Mouse events
    MouseMove(MouseEvent),
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    MouseScroll(MouseEvent),
    MouseEnter,
    MouseLeave,
    
    // Keyboard events
    KeyDown(KeyEvent),
    KeyUp(KeyEvent),
    CharInput(char),
    
    // Touch events (for future use)
    TouchStart { id: u64, position: Vec2 },
    TouchMove { id: u64, position: Vec2 },
    TouchEnd { id: u64, position: Vec2 },
    
    // Custom events
    Custom(String),
}

/// Event handler trait
pub trait EventHandler {
    fn on_event(&mut self, event: &Event) -> bool;
}

/// Event loop for processing events
pub struct EventLoop {
    events: VecDeque<Event>,
    running: bool,
}

impl EventLoop {
    /// Create a new event loop
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            running: true,
        }
    }

    /// Push an event to the queue
    pub fn push(&mut self, event: Event) {
        self.events.push_back(event);
    }

    /// Pop an event from the queue
    pub fn pop(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    /// Check if there are pending events
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }

    /// Check if the loop is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stop the event loop
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Process all pending events with a handler
    pub fn process<H: EventHandler>(&mut self, handler: &mut H) {
        while let Some(event) = self.pop() {
            if matches!(event, Event::WindowClose) {
                self.running = false;
            }
            handler.on_event(&event);
        }
    }

    /// Clear all pending events
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}
