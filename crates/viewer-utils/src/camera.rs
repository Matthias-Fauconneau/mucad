// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A camera like `QCamera` of Qt 3D, which a viewer applies to its own camera.

use glam::{Mat4, Quat, Vec3};

/// A camera looking from its position at its view center, like `QCamera` of Qt 3D.
///
/// The projection maps the coordinates of the camera to the normalized device coordinates of
/// OpenGL, from -1 to 1 in x, y and z.
#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    /// The position of the camera.
    pub position: Vec3,
    /// The point the camera looks at.
    pub view_center: Vec3,
    /// The direction, which is upwards in the view.
    pub up_vector: Vec3,
    /// The projection from the coordinates of the camera to normalized device coordinates.
    pub projection_matrix: Mat4,
}

impl Default for Camera {
    /// The camera of Qt 3D: at the origin, looking to -z with y up, with a perspective
    /// projection.
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            view_center: Vec3::new(0.0, 0.0, -100.0),
            up_vector: Vec3::Y,
            projection_matrix: Mat4::perspective_rh_gl(25f32.to_radians(), 1.0, 0.1, 1024.0),
        }
    }
}

impl Camera {
    /// The vector from the position to the view center.
    pub fn view_vector(&self) -> Vec3 {
        self.view_center - self.position
    }

    /// The transformation from world coordinates to the coordinates of the camera.
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.view_center, self.up_vector)
    }

    /// The transformation from the coordinates of the camera to world coordinates, i.e. the
    /// placement of the camera in the world, like `QCamera::transform()->matrix()`.
    pub fn transform_matrix(&self) -> Mat4 {
        self.view_matrix().inverse()
    }

    /// The rotation by `angle` in degrees around the up vector.
    pub fn pan_rotation(&self, angle: f32) -> Quat {
        from_axis_and_angle(self.up_vector, angle)
    }

    /// The rotation by `angle` in degrees around the horizontal axis of the view.
    pub fn tilt_rotation(&self, angle: f32) -> Quat {
        let x_basis = self
            .up_vector
            .cross(self.view_vector().normalize_or_zero())
            .normalize_or_zero();
        from_axis_and_angle(x_basis, -angle)
    }
}

/// The rotation by `angle` in degrees around `axis`, like `QQuaternion::fromAxisAndAngle`.
fn from_axis_and_angle(axis: Vec3, angle: f32) -> Quat {
    Quat::from_axis_angle(axis.normalize_or_zero(), angle.to_radians())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> Camera {
        Camera {
            position: Vec3::new(0.0, 0.0, 10.0),
            view_center: Vec3::ZERO,
            ..Camera::default()
        }
    }

    #[test]
    fn transform() {
        let camera = camera();
        let transform = camera.transform_matrix();
        assert!(
            transform
                .transform_point3(Vec3::ZERO)
                .abs_diff_eq(camera.position, 1e-6)
        );
        // The camera looks along its -z axis.
        let forward = transform.transform_vector3(-Vec3::Z);
        assert!(forward.abs_diff_eq(camera.view_vector().normalize(), 1e-6));
    }

    #[test]
    fn rotations() {
        let camera = camera();
        // Panning to the left turns the view vector to the left.
        let v = camera.pan_rotation(90.0) * camera.view_vector();
        assert!(v.abs_diff_eq(Vec3::new(-10.0, 0.0, 0.0), 1e-5));
        // Tilting upwards turns the view vector upwards.
        let v = camera.tilt_rotation(90.0) * camera.view_vector();
        assert!(v.abs_diff_eq(Vec3::new(0.0, 10.0, 0.0), 1e-5));
    }
}
