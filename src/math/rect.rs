//! Rectangle type for EPICX

use serde::{Deserialize, Serialize};
use glam::Vec2;

/// A 2D rectangle defined by position and size
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle
    #[inline]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Create a rectangle from position and size vectors
    #[inline]
    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self::new(pos.x, pos.y, size.x, size.y)
    }

    /// Create a rectangle from two corner points
    pub fn from_corners(min: Vec2, max: Vec2) -> Self {
        Self::new(min.x, min.y, max.x - min.x, max.y - min.y)
    }

    /// Create a zero-sized rectangle at origin
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Get the position as a Vec2
    #[inline]
    pub fn position(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Get the size as a Vec2
    #[inline]
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    /// Get the center point
    #[inline]
    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Get the minimum corner (top-left)
    #[inline]
    pub fn min(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Get the maximum corner (bottom-right)
    #[inline]
    pub fn max(&self) -> Vec2 {
        Vec2::new(self.x + self.width, self.y + self.height)
    }

    /// Check if a point is inside the rectangle
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Get the intersection of two rectangles
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }

        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);

        Some(Rect::new(x, y, right - x, bottom - y))
    }

    /// Get the union of two rectangles (bounding box)
    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = (self.x + self.width).max(other.x + other.width);
        let bottom = (self.y + self.height).max(other.y + other.height);

        Rect::new(x, y, right - x, bottom - y)
    }

    /// Expand the rectangle by a given amount on all sides
    pub fn expand(&self, amount: f32) -> Rect {
        Rect::new(
            self.x - amount,
            self.y - amount,
            self.width + amount * 2.0,
            self.height + amount * 2.0,
        )
    }

    /// Translate the rectangle by a given offset
    pub fn translate(&self, offset: Vec2) -> Rect {
        Rect::new(self.x + offset.x, self.y + offset.y, self.width, self.height)
    }

    /// Scale the rectangle by a factor
    pub fn scale(&self, factor: f32) -> Rect {
        Rect::new(
            self.x * factor,
            self.y * factor,
            self.width * factor,
            self.height * factor,
        )
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::zero()
    }
}
