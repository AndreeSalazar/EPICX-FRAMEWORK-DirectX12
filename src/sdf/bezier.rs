//! Bézier Curves and Surfaces as SDFs

use super::Sdf;
use crate::math::Vec3;

/// Quadratic Bézier curve SDF
#[derive(Debug, Clone)]
pub struct BezierQuadratic {
    pub p0: Vec3,
    pub p1: Vec3, // control point
    pub p2: Vec3,
    pub radius: f32,
}

impl BezierQuadratic {
    pub fn new(p0: Vec3, p1: Vec3, p2: Vec3, radius: f32) -> Self {
        Self { p0, p1, p2, radius }
    }
    
    /// Evaluate the curve at parameter t
    pub fn evaluate(&self, t: f32) -> Vec3 {
        let t1 = 1.0 - t;
        self.p0 * (t1 * t1) + self.p1 * (2.0 * t1 * t) + self.p2 * (t * t)
    }
    
    /// Evaluate the derivative at parameter t
    pub fn derivative(&self, t: f32) -> Vec3 {
        let t1 = 1.0 - t;
        (self.p1 - self.p0) * (2.0 * t1) + (self.p2 - self.p1) * (2.0 * t)
    }
}

impl Sdf for BezierQuadratic {
    fn distance(&self, p: Vec3) -> f32 {
        // Find closest point on curve using Newton's method
        let mut t = 0.5;
        for _ in 0..5 {
            let curve_p = self.evaluate(t);
            let deriv = self.derivative(t);
            let diff = curve_p - p;
            let dot = diff.dot(deriv);
            let deriv_len_sq = deriv.length_squared();
            if deriv_len_sq > 0.0001 {
                t -= dot / deriv_len_sq;
                t = t.clamp(0.0, 1.0);
            }
        }
        
        (p - self.evaluate(t)).length() - self.radius
    }
}

/// Cubic Bézier curve SDF
#[derive(Debug, Clone)]
pub struct BezierCubic {
    pub p0: Vec3,
    pub p1: Vec3, // control point 1
    pub p2: Vec3, // control point 2
    pub p3: Vec3,
    pub radius: f32,
}

impl BezierCubic {
    pub fn new(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, radius: f32) -> Self {
        Self { p0, p1, p2, p3, radius }
    }
    
    /// Evaluate the curve at parameter t
    pub fn evaluate(&self, t: f32) -> Vec3 {
        let t1 = 1.0 - t;
        let t1_2 = t1 * t1;
        let t1_3 = t1_2 * t1;
        let t_2 = t * t;
        let t_3 = t_2 * t;
        
        self.p0 * t1_3 + 
        self.p1 * (3.0 * t1_2 * t) + 
        self.p2 * (3.0 * t1 * t_2) + 
        self.p3 * t_3
    }
    
    /// Evaluate the derivative at parameter t
    pub fn derivative(&self, t: f32) -> Vec3 {
        let t1 = 1.0 - t;
        let t1_2 = t1 * t1;
        let t_2 = t * t;
        
        (self.p1 - self.p0) * (3.0 * t1_2) +
        (self.p2 - self.p1) * (6.0 * t1 * t) +
        (self.p3 - self.p2) * (3.0 * t_2)
    }
}

impl Sdf for BezierCubic {
    fn distance(&self, p: Vec3) -> f32 {
        // Find closest point using Newton's method
        let mut t = 0.5;
        for _ in 0..8 {
            let curve_p = self.evaluate(t);
            let deriv = self.derivative(t);
            let diff = curve_p - p;
            let dot = diff.dot(deriv);
            let deriv_len_sq = deriv.length_squared();
            if deriv_len_sq > 0.0001 {
                t -= dot / deriv_len_sq;
                t = t.clamp(0.0, 1.0);
            }
        }
        
        (p - self.evaluate(t)).length() - self.radius
    }
}

/// Bicubic Bézier patch (surface)
#[derive(Debug, Clone)]
pub struct BezierPatch {
    /// 4x4 control points
    pub control_points: [[Vec3; 4]; 4],
}

impl BezierPatch {
    pub fn new(control_points: [[Vec3; 4]; 4]) -> Self {
        Self { control_points }
    }
    
    /// Evaluate the surface at parameters (u, v)
    pub fn evaluate(&self, u: f32, v: f32) -> Vec3 {
        let mut result = Vec3::ZERO;
        
        for i in 0..4 {
            for j in 0..4 {
                let bi = bernstein_basis(i, u);
                let bj = bernstein_basis(j, v);
                result += self.control_points[i][j] * (bi * bj);
            }
        }
        
        result
    }
    
    /// Evaluate partial derivative with respect to u
    pub fn derivative_u(&self, u: f32, v: f32) -> Vec3 {
        let mut result = Vec3::ZERO;
        
        for i in 0..3 {
            for j in 0..4 {
                let diff = self.control_points[i + 1][j] - self.control_points[i][j];
                let bi = bernstein_basis_derivative(i, u);
                let bj = bernstein_basis(j, v);
                result += diff * (3.0 * bi * bj);
            }
        }
        
        result
    }
    
    /// Evaluate partial derivative with respect to v
    pub fn derivative_v(&self, u: f32, v: f32) -> Vec3 {
        let mut result = Vec3::ZERO;
        
        for i in 0..4 {
            for j in 0..3 {
                let diff = self.control_points[i][j + 1] - self.control_points[i][j];
                let bi = bernstein_basis(i, u);
                let bj = bernstein_basis_derivative(j, v);
                result += diff * (3.0 * bi * bj);
            }
        }
        
        result
    }
    
    /// Get surface normal at (u, v)
    pub fn normal(&self, u: f32, v: f32) -> Vec3 {
        let du = self.derivative_u(u, v);
        let dv = self.derivative_v(u, v);
        du.cross(dv).normalize()
    }
}

impl Sdf for BezierPatch {
    fn distance(&self, p: Vec3) -> f32 {
        // Sample the surface to find approximate closest point
        let samples = 8;
        let mut min_dist = f32::MAX;
        
        for i in 0..=samples {
            for j in 0..=samples {
                let u = i as f32 / samples as f32;
                let v = j as f32 / samples as f32;
                let surface_p = self.evaluate(u, v);
                let dist = (p - surface_p).length();
                min_dist = min_dist.min(dist);
            }
        }
        
        min_dist
    }
}

/// Bernstein basis polynomial
fn bernstein_basis(i: usize, t: f32) -> f32 {
    let t1 = 1.0 - t;
    match i {
        0 => t1 * t1 * t1,
        1 => 3.0 * t1 * t1 * t,
        2 => 3.0 * t1 * t * t,
        3 => t * t * t,
        _ => 0.0,
    }
}

/// Bernstein basis derivative
fn bernstein_basis_derivative(i: usize, t: f32) -> f32 {
    let t1 = 1.0 - t;
    match i {
        0 => t1 * t1,
        1 => 2.0 * t1 * t,
        2 => t * t,
        _ => 0.0,
    }
}
