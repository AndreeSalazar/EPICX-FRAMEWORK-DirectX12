//! ADead-Vector3D: SDF (Signed Distance Functions) Module
//!
//! Pure mathematics 3D rendering - migrated from ADead-GPU C++
//!
//! Inspired by Adobe Illustrator - pure vector mathematics in 3D!
//! - Infinite scalability
//! - Perfect anti-aliasing
//! - Minimal memory (~1KB vs ~1MB for meshes)

mod primitives;
mod operations;
mod bezier;
mod antialiasing;

pub use primitives::*;
pub use operations::*;
pub use bezier::*;
pub use antialiasing::*;

use crate::math::{Vec2, Vec3};

/// A Signed Distance Function trait
pub trait Sdf: Send + Sync {
    /// Evaluate the SDF at a point
    fn distance(&self, p: Vec3) -> f32;
    
    /// Get the normal at a point (using gradient)
    fn normal(&self, p: Vec3) -> Vec3 {
        let eps = 0.001;
        let dx = self.distance(p + Vec3::new(eps, 0.0, 0.0)) - self.distance(p - Vec3::new(eps, 0.0, 0.0));
        let dy = self.distance(p + Vec3::new(0.0, eps, 0.0)) - self.distance(p - Vec3::new(0.0, eps, 0.0));
        let dz = self.distance(p + Vec3::new(0.0, 0.0, eps)) - self.distance(p - Vec3::new(0.0, 0.0, eps));
        Vec3::new(dx, dy, dz).normalize()
    }
    
    /// Get bounding box (for acceleration)
    fn bounds(&self) -> (Vec3, Vec3) {
        (Vec3::splat(-1000.0), Vec3::splat(1000.0))
    }
}

/// Ray marching configuration
#[derive(Debug, Clone)]
pub struct RayMarchConfig {
    pub max_steps: u32,
    pub max_distance: f32,
    pub epsilon: f32,
    pub over_relaxation: f32,
}

impl Default for RayMarchConfig {
    fn default() -> Self {
        Self {
            max_steps: 128,
            max_distance: 100.0,
            epsilon: 0.001,
            over_relaxation: 1.6, // Over-relaxation for faster convergence
        }
    }
}

/// Ray march result
#[derive(Debug, Clone)]
pub struct RayMarchHit {
    pub hit: bool,
    pub distance: f32,
    pub position: Vec3,
    pub normal: Vec3,
    pub steps: u32,
}

/// Perform ray marching against an SDF
pub fn ray_march<S: Sdf>(sdf: &S, origin: Vec3, direction: Vec3, config: &RayMarchConfig) -> RayMarchHit {
    let mut t = 0.0f32;
    let dir = direction.normalize();
    
    for step in 0..config.max_steps {
        let p = origin + dir * t;
        let d = sdf.distance(p);
        
        if d < config.epsilon {
            let normal = sdf.normal(p);
            return RayMarchHit {
                hit: true,
                distance: t,
                position: p,
                normal,
                steps: step,
            };
        }
        
        if t > config.max_distance {
            break;
        }
        
        // Over-relaxation sphere tracing
        t += d * config.over_relaxation;
    }
    
    RayMarchHit {
        hit: false,
        distance: config.max_distance,
        position: origin + dir * config.max_distance,
        normal: Vec3::ZERO,
        steps: config.max_steps,
    }
}

/// SDF Scene - collection of SDF objects
pub struct SdfScene {
    objects: Vec<Box<dyn Sdf>>,
}

impl SdfScene {
    pub fn new() -> Self {
        Self { objects: Vec::new() }
    }
    
    pub fn add<S: Sdf + 'static>(&mut self, sdf: S) {
        self.objects.push(Box::new(sdf));
    }
    
    pub fn clear(&mut self) {
        self.objects.clear();
    }
}

impl Default for SdfScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Sdf for SdfScene {
    fn distance(&self, p: Vec3) -> f32 {
        self.objects
            .iter()
            .map(|obj| obj.distance(p))
            .fold(f32::MAX, f32::min)
    }
}
