//! SDF Primitives - Basic shapes as Signed Distance Functions

use super::Sdf;
use crate::math::Vec3;

/// Sphere SDF
#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
    
    pub fn unit() -> Self {
        Self::new(Vec3::ZERO, 1.0)
    }
}

impl Sdf for Sphere {
    fn distance(&self, p: Vec3) -> f32 {
        (p - self.center).length() - self.radius
    }
    
    fn bounds(&self) -> (Vec3, Vec3) {
        let r = Vec3::splat(self.radius);
        (self.center - r, self.center + r)
    }
}

/// Box SDF
#[derive(Debug, Clone)]
pub struct Box3D {
    pub center: Vec3,
    pub half_extents: Vec3,
}

impl Box3D {
    pub fn new(center: Vec3, half_extents: Vec3) -> Self {
        Self { center, half_extents }
    }
    
    pub fn unit() -> Self {
        Self::new(Vec3::ZERO, Vec3::splat(0.5))
    }
    
    pub fn cube(center: Vec3, size: f32) -> Self {
        Self::new(center, Vec3::splat(size * 0.5))
    }
}

impl Sdf for Box3D {
    fn distance(&self, p: Vec3) -> f32 {
        let q = (p - self.center).abs() - self.half_extents;
        q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0)
    }
    
    fn bounds(&self) -> (Vec3, Vec3) {
        (self.center - self.half_extents, self.center + self.half_extents)
    }
}

/// Cylinder SDF
#[derive(Debug, Clone)]
pub struct Cylinder {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
}

impl Cylinder {
    pub fn new(center: Vec3, radius: f32, height: f32) -> Self {
        Self { center, radius, height }
    }
}

impl Sdf for Cylinder {
    fn distance(&self, p: Vec3) -> f32 {
        let local = p - self.center;
        let d_xz = (local.x * local.x + local.z * local.z).sqrt() - self.radius;
        let d_y = local.y.abs() - self.height * 0.5;
        d_xz.max(d_y).min(0.0) + (d_xz.max(0.0).powi(2) + d_y.max(0.0).powi(2)).sqrt()
    }
}

/// Torus SDF
#[derive(Debug, Clone)]
pub struct Torus {
    pub center: Vec3,
    pub major_radius: f32,
    pub minor_radius: f32,
}

impl Torus {
    pub fn new(center: Vec3, major_radius: f32, minor_radius: f32) -> Self {
        Self { center, major_radius, minor_radius }
    }
}

impl Sdf for Torus {
    fn distance(&self, p: Vec3) -> f32 {
        let local = p - self.center;
        let q_x = (local.x * local.x + local.z * local.z).sqrt() - self.major_radius;
        let q = (q_x * q_x + local.y * local.y).sqrt();
        q - self.minor_radius
    }
}

/// Capsule SDF (line segment with radius)
#[derive(Debug, Clone)]
pub struct Capsule {
    pub a: Vec3,
    pub b: Vec3,
    pub radius: f32,
}

impl Capsule {
    pub fn new(a: Vec3, b: Vec3, radius: f32) -> Self {
        Self { a, b, radius }
    }
}

impl Sdf for Capsule {
    fn distance(&self, p: Vec3) -> f32 {
        let pa = p - self.a;
        let ba = self.b - self.a;
        let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
        (pa - ba * h).length() - self.radius
    }
}

/// Cone SDF
#[derive(Debug, Clone)]
pub struct Cone {
    pub tip: Vec3,
    pub height: f32,
    pub angle: f32, // in radians
}

impl Cone {
    pub fn new(tip: Vec3, height: f32, angle: f32) -> Self {
        Self { tip, height, angle }
    }
}

impl Sdf for Cone {
    fn distance(&self, p: Vec3) -> f32 {
        let local = p - self.tip;
        let q = (local.x * local.x + local.z * local.z).sqrt();
        let c = self.angle.cos();
        let s = self.angle.sin();
        (c * q + s * local.y).max(-local.y - self.height)
    }
}

/// Plane SDF
#[derive(Debug, Clone)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal: normal.normalize(), distance }
    }
    
    pub fn ground(height: f32) -> Self {
        Self::new(Vec3::Y, -height)
    }
}

impl Sdf for Plane {
    fn distance(&self, p: Vec3) -> f32 {
        p.dot(self.normal) + self.distance
    }
}

/// Rounded Box SDF
#[derive(Debug, Clone)]
pub struct RoundedBox {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub radius: f32,
}

impl RoundedBox {
    pub fn new(center: Vec3, half_extents: Vec3, radius: f32) -> Self {
        Self { center, half_extents, radius }
    }
}

impl Sdf for RoundedBox {
    fn distance(&self, p: Vec3) -> f32 {
        let q = (p - self.center).abs() - self.half_extents + Vec3::splat(self.radius);
        q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0) - self.radius
    }
}
