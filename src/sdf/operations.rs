//! SDF Operations - CSG and transformations

use super::Sdf;
use crate::math::Vec3;

/// Union of two SDFs (min)
pub struct Union<A: Sdf, B: Sdf> {
    pub a: A,
    pub b: B,
}

impl<A: Sdf, B: Sdf> Union<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Sdf, B: Sdf> Sdf for Union<A, B> {
    fn distance(&self, p: Vec3) -> f32 {
        self.a.distance(p).min(self.b.distance(p))
    }
}

/// Intersection of two SDFs (max)
pub struct Intersection<A: Sdf, B: Sdf> {
    pub a: A,
    pub b: B,
}

impl<A: Sdf, B: Sdf> Intersection<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Sdf, B: Sdf> Sdf for Intersection<A, B> {
    fn distance(&self, p: Vec3) -> f32 {
        self.a.distance(p).max(self.b.distance(p))
    }
}

/// Subtraction of two SDFs (A - B)
pub struct Subtraction<A: Sdf, B: Sdf> {
    pub a: A,
    pub b: B,
}

impl<A: Sdf, B: Sdf> Subtraction<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Sdf, B: Sdf> Sdf for Subtraction<A, B> {
    fn distance(&self, p: Vec3) -> f32 {
        self.a.distance(p).max(-self.b.distance(p))
    }
}

/// Smooth union (blend)
pub struct SmoothUnion<A: Sdf, B: Sdf> {
    pub a: A,
    pub b: B,
    pub k: f32, // smoothness factor
}

impl<A: Sdf, B: Sdf> SmoothUnion<A, B> {
    pub fn new(a: A, b: B, k: f32) -> Self {
        Self { a, b, k }
    }
}

impl<A: Sdf, B: Sdf> Sdf for SmoothUnion<A, B> {
    fn distance(&self, p: Vec3) -> f32 {
        let d1 = self.a.distance(p);
        let d2 = self.b.distance(p);
        let h = (0.5 + 0.5 * (d2 - d1) / self.k).clamp(0.0, 1.0);
        d2 * (1.0 - h) + d1 * h - self.k * h * (1.0 - h)
    }
}

/// Smooth subtraction
pub struct SmoothSubtraction<A: Sdf, B: Sdf> {
    pub a: A,
    pub b: B,
    pub k: f32,
}

impl<A: Sdf, B: Sdf> SmoothSubtraction<A, B> {
    pub fn new(a: A, b: B, k: f32) -> Self {
        Self { a, b, k }
    }
}

impl<A: Sdf, B: Sdf> Sdf for SmoothSubtraction<A, B> {
    fn distance(&self, p: Vec3) -> f32 {
        let d1 = self.a.distance(p);
        let d2 = self.b.distance(p);
        let h = (0.5 - 0.5 * (d2 + d1) / self.k).clamp(0.0, 1.0);
        d1 * (1.0 - h) + (-d2) * h + self.k * h * (1.0 - h)
    }
}

/// Smooth intersection
pub struct SmoothIntersection<A: Sdf, B: Sdf> {
    pub a: A,
    pub b: B,
    pub k: f32,
}

impl<A: Sdf, B: Sdf> SmoothIntersection<A, B> {
    pub fn new(a: A, b: B, k: f32) -> Self {
        Self { a, b, k }
    }
}

impl<A: Sdf, B: Sdf> Sdf for SmoothIntersection<A, B> {
    fn distance(&self, p: Vec3) -> f32 {
        let d1 = self.a.distance(p);
        let d2 = self.b.distance(p);
        let h = (0.5 - 0.5 * (d2 - d1) / self.k).clamp(0.0, 1.0);
        d2 * (1.0 - h) + d1 * h + self.k * h * (1.0 - h)
    }
}

/// Translation transform
pub struct Translate<S: Sdf> {
    pub sdf: S,
    pub offset: Vec3,
}

impl<S: Sdf> Translate<S> {
    pub fn new(sdf: S, offset: Vec3) -> Self {
        Self { sdf, offset }
    }
}

impl<S: Sdf> Sdf for Translate<S> {
    fn distance(&self, p: Vec3) -> f32 {
        self.sdf.distance(p - self.offset)
    }
}

/// Scale transform
pub struct Scale<S: Sdf> {
    pub sdf: S,
    pub scale: f32,
}

impl<S: Sdf> Scale<S> {
    pub fn new(sdf: S, scale: f32) -> Self {
        Self { sdf, scale }
    }
}

impl<S: Sdf> Sdf for Scale<S> {
    fn distance(&self, p: Vec3) -> f32 {
        self.sdf.distance(p / self.scale) * self.scale
    }
}

/// Onion (shell) operation
pub struct Onion<S: Sdf> {
    pub sdf: S,
    pub thickness: f32,
}

impl<S: Sdf> Onion<S> {
    pub fn new(sdf: S, thickness: f32) -> Self {
        Self { sdf, thickness }
    }
}

impl<S: Sdf> Sdf for Onion<S> {
    fn distance(&self, p: Vec3) -> f32 {
        self.sdf.distance(p).abs() - self.thickness
    }
}

/// Round operation (add radius)
pub struct Round<S: Sdf> {
    pub sdf: S,
    pub radius: f32,
}

impl<S: Sdf> Round<S> {
    pub fn new(sdf: S, radius: f32) -> Self {
        Self { sdf, radius }
    }
}

impl<S: Sdf> Sdf for Round<S> {
    fn distance(&self, p: Vec3) -> f32 {
        self.sdf.distance(p) - self.radius
    }
}

/// Elongate operation
pub struct Elongate<S: Sdf> {
    pub sdf: S,
    pub h: Vec3,
}

impl<S: Sdf> Elongate<S> {
    pub fn new(sdf: S, h: Vec3) -> Self {
        Self { sdf, h }
    }
}

impl<S: Sdf> Sdf for Elongate<S> {
    fn distance(&self, p: Vec3) -> f32 {
        let q = p.abs() - self.h;
        let clamped = Vec3::new(
            q.x.max(0.0),
            q.y.max(0.0),
            q.z.max(0.0),
        );
        self.sdf.distance(clamped) + q.x.max(q.y.max(q.z)).min(0.0)
    }
}

/// Twist operation (around Y axis)
pub struct Twist<S: Sdf> {
    pub sdf: S,
    pub k: f32, // twist amount per unit
}

impl<S: Sdf> Twist<S> {
    pub fn new(sdf: S, k: f32) -> Self {
        Self { sdf, k }
    }
}

impl<S: Sdf> Sdf for Twist<S> {
    fn distance(&self, p: Vec3) -> f32 {
        let angle = self.k * p.y;
        let c = angle.cos();
        let s = angle.sin();
        let q = Vec3::new(c * p.x - s * p.z, p.y, s * p.x + c * p.z);
        self.sdf.distance(q)
    }
}

/// Bend operation (around Z axis)
pub struct Bend<S: Sdf> {
    pub sdf: S,
    pub k: f32,
}

impl<S: Sdf> Bend<S> {
    pub fn new(sdf: S, k: f32) -> Self {
        Self { sdf, k }
    }
}

impl<S: Sdf> Sdf for Bend<S> {
    fn distance(&self, p: Vec3) -> f32 {
        let angle = self.k * p.x;
        let c = angle.cos();
        let s = angle.sin();
        let q = Vec3::new(c * p.x - s * p.y, s * p.x + c * p.y, p.z);
        self.sdf.distance(q)
    }
}

/// Repetition (infinite)
pub struct Repeat<S: Sdf> {
    pub sdf: S,
    pub period: Vec3,
}

impl<S: Sdf> Repeat<S> {
    pub fn new(sdf: S, period: Vec3) -> Self {
        Self { sdf, period }
    }
}

impl<S: Sdf> Sdf for Repeat<S> {
    fn distance(&self, p: Vec3) -> f32 {
        let q = Vec3::new(
            ((p.x / self.period.x).fract() - 0.5) * self.period.x,
            ((p.y / self.period.y).fract() - 0.5) * self.period.y,
            ((p.z / self.period.z).fract() - 0.5) * self.period.z,
        );
        self.sdf.distance(q)
    }
}

/// Limited repetition
pub struct RepeatLimited<S: Sdf> {
    pub sdf: S,
    pub period: Vec3,
    pub limit: Vec3, // number of repetitions in each direction
}

impl<S: Sdf> RepeatLimited<S> {
    pub fn new(sdf: S, period: Vec3, limit: Vec3) -> Self {
        Self { sdf, period, limit }
    }
}

impl<S: Sdf> Sdf for RepeatLimited<S> {
    fn distance(&self, p: Vec3) -> f32 {
        let q = p - self.period * Vec3::new(
            (p.x / self.period.x).round().clamp(-self.limit.x, self.limit.x),
            (p.y / self.period.y).round().clamp(-self.limit.y, self.limit.y),
            (p.z / self.period.z).round().clamp(-self.limit.z, self.limit.z),
        );
        self.sdf.distance(q)
    }
}
