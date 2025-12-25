//! Transform type for EPICX

use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// A 3D transform with position, rotation, and scale
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    /// Create a new transform with default values
    pub const fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Create a transform with only position
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create a transform with position and rotation
    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            ..Default::default()
        }
    }

    /// Create a transform with all components
    pub fn from_components(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Convert to a 4x4 transformation matrix
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Create a transform from a 4x4 matrix
    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, position) = matrix.to_scale_rotation_translation();
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Translate the transform
    pub fn translate(&mut self, offset: Vec3) -> &mut Self {
        self.position += offset;
        self
    }

    /// Rotate the transform
    pub fn rotate(&mut self, rotation: Quat) -> &mut Self {
        self.rotation = rotation * self.rotation;
        self
    }

    /// Rotate around an axis
    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) -> &mut Self {
        self.rotation = Quat::from_axis_angle(axis, angle) * self.rotation;
        self
    }

    /// Scale the transform
    pub fn scale_by(&mut self, scale: Vec3) -> &mut Self {
        self.scale *= scale;
        self
    }

    /// Uniform scale
    pub fn scale_uniform(&mut self, scale: f32) -> &mut Self {
        self.scale *= scale;
        self
    }

    /// Get the forward direction (negative Z)
    pub fn forward(&self) -> Vec3 {
        self.rotation * -Vec3::Z
    }

    /// Get the right direction (positive X)
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction (positive Y)
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Look at a target position
    pub fn look_at(&mut self, target: Vec3, up: Vec3) -> &mut Self {
        let forward = (target - self.position).normalize();
        self.rotation = Quat::from_rotation_arc(-Vec3::Z, forward);
        self
    }

    /// Interpolate between two transforms
    pub fn lerp(self, other: Transform, t: f32) -> Transform {
        Transform {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }

    /// Combine two transforms (self * other)
    pub fn combine(&self, other: &Transform) -> Transform {
        Transform {
            position: self.position + self.rotation * (self.scale * other.position),
            rotation: self.rotation * other.rotation,
            scale: self.scale * other.scale,
        }
    }

    /// Get the inverse transform
    pub fn inverse(&self) -> Transform {
        let inv_rotation = self.rotation.inverse();
        let inv_scale = Vec3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        Transform {
            position: inv_rotation * (-self.position * inv_scale),
            rotation: inv_rotation,
            scale: inv_scale,
        }
    }

    /// Transform a point
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.position + self.rotation * (self.scale * point)
    }

    /// Transform a direction (ignores position)
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.rotation * direction
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Transform> for Mat4 {
    fn from(transform: Transform) -> Self {
        transform.to_matrix()
    }
}

impl From<Mat4> for Transform {
    fn from(matrix: Mat4) -> Self {
        Transform::from_matrix(matrix)
    }
}
