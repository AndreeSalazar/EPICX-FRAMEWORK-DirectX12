//! ADead-AA: SDF Anti-Aliasing
//!
//! Pure mathematical anti-aliasing using SDF gradients.
//! Resolution-independent, zero extra memory, perfect edges.

use crate::math::{Color, Vec2};

/// SDF Anti-Aliasing configuration
#[derive(Debug, Clone)]
pub struct SdfAAConfig {
    /// Edge softness (in screen pixels)
    pub edge_softness: f32,
    /// Enable temporal anti-aliasing blend
    pub temporal_blend: f32,
    /// Subpixel jitter for temporal AA
    pub jitter: Vec2,
}

impl Default for SdfAAConfig {
    fn default() -> Self {
        Self {
            edge_softness: 1.0,
            temporal_blend: 0.9,
            jitter: Vec2::ZERO,
        }
    }
}

/// Apply SDF anti-aliasing to a distance value
/// 
/// Uses the screen-space derivative (fwidth equivalent) to determine
/// the appropriate smoothstep range for anti-aliasing.
/// 
/// # Arguments
/// * `distance` - The signed distance value
/// * `fwidth` - The screen-space derivative magnitude (|dFdx| + |dFdy|)
/// 
/// # Returns
/// Alpha value from 0.0 (outside) to 1.0 (inside)
pub fn sdf_aa(distance: f32, fwidth: f32) -> f32 {
    let edge = fwidth * 0.5;
    smoothstep(edge, -edge, distance)
}

/// Apply SDF anti-aliasing with configurable softness
pub fn sdf_aa_soft(distance: f32, fwidth: f32, softness: f32) -> f32 {
    let edge = fwidth * softness * 0.5;
    smoothstep(edge, -edge, distance)
}

/// Smoothstep function (GLSL/HLSL compatible)
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smoother step (quintic interpolation)
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Calculate approximate fwidth from screen-space derivatives
/// 
/// In a real shader, this would use dFdx/dFdy. Here we approximate
/// based on the distance field gradient.
pub fn calculate_fwidth(distance: f32, distance_dx: f32, distance_dy: f32) -> f32 {
    (distance_dx - distance).abs() + (distance_dy - distance).abs()
}

/// Apply SDF anti-aliasing to a color
pub fn sdf_aa_color(inside_color: Color, outside_color: Color, distance: f32, fwidth: f32) -> Color {
    let alpha = sdf_aa(distance, fwidth);
    outside_color.lerp(inside_color, alpha)
}

/// Multi-sample anti-aliasing for SDFs
/// 
/// Samples the SDF at multiple points and averages the result.
pub fn sdf_msaa<F>(sample_fn: F, center: Vec2, pixel_size: f32, samples: u32) -> f32
where
    F: Fn(Vec2) -> f32,
{
    if samples <= 1 {
        return sample_fn(center);
    }
    
    let mut sum = 0.0;
    let offset = pixel_size * 0.25;
    
    // 2x2 rotated grid pattern
    let offsets = [
        Vec2::new(-offset, -offset * 0.5),
        Vec2::new(offset, -offset * 0.5),
        Vec2::new(-offset * 0.5, offset),
        Vec2::new(offset * 0.5, offset),
    ];
    
    let sample_count = samples.min(4) as usize;
    for i in 0..sample_count {
        sum += sample_fn(center + offsets[i]);
    }
    
    sum / sample_count as f32
}

/// Supersampling anti-aliasing for SDFs
pub fn sdf_ssaa<F>(sample_fn: F, center: Vec2, pixel_size: f32, grid_size: u32) -> f32
where
    F: Fn(Vec2) -> f32,
{
    let mut sum = 0.0;
    let step = pixel_size / grid_size as f32;
    let start = -pixel_size * 0.5 + step * 0.5;
    
    for y in 0..grid_size {
        for x in 0..grid_size {
            let offset = Vec2::new(
                start + x as f32 * step,
                start + y as f32 * step,
            );
            let d = sample_fn(center + offset);
            sum += if d < 0.0 { 1.0 } else { 0.0 };
        }
    }
    
    sum / (grid_size * grid_size) as f32
}

/// Edge detection for SDFs
pub fn sdf_edge_detect(distance: f32, fwidth: f32, edge_width: f32) -> f32 {
    let d = distance.abs();
    let edge = fwidth * edge_width;
    1.0 - smoothstep(0.0, edge, d)
}

/// Outline effect for SDFs
pub fn sdf_outline(distance: f32, fwidth: f32, outline_width: f32) -> (f32, f32) {
    let fill = sdf_aa(distance, fwidth);
    let outline_dist = distance.abs() - outline_width;
    let outline = sdf_aa(outline_dist, fwidth);
    (fill, outline)
}

/// Drop shadow for SDFs
pub fn sdf_shadow(distance: f32, _shadow_offset: Vec2, shadow_softness: f32, fwidth: f32) -> f32 {
    // The shadow distance would be calculated with the offset applied
    // Here we just show the softness calculation
    let shadow_edge = fwidth * shadow_softness;
    smoothstep(shadow_edge, -shadow_edge, distance)
}
