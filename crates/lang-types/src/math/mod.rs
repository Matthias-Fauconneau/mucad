// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Mathematical operations.

mod affine_transform;
pub mod float;
mod ops;
mod trig;

pub use affine_transform::AffineTransform;
pub use float::*;
pub use ops::MathOps;

use crate::{Angle, Mat3, Mat4, Scalar, Scalar3, Vec4};

use glam::{DMat3, DVec3};

/// Helper function to return rotation X,Y,Z rotation matrices.
fn rotation_matrices_xyz(x: Angle, y: Angle, z: Angle) -> (DMat3, DMat3, DMat3) {
    (
        trig::rotation_x(ScalarF::from_fixed(x)),
        trig::rotation_y(ScalarF::from_fixed(y)),
        trig::rotation_z(ScalarF::from_fixed(z)),
    )
}

pub fn rotate_around_axis(angle: Angle, x: Scalar, y: Scalar, z: Scalar) -> Mat3 {
    let axis = Scalar3::new(x, y, z);
    let angle: ScalarF = angle.into_float();
    trig::rotation_axis(axis.into_float(), angle).into_fixed()
}

/// Rotation about the X, then the Y, then the Z axis, which stay fixed (`Rz·Ry·Rx`).
///
/// This is the yaw-pitch-roll rotation of FreeCAD with roll `x`, pitch `y` and yaw `z`.
/// Positive angles rotate counter-clockwise, seen from the positive end of the axis.
pub fn rotate_xyz(x: Angle, y: Angle, z: Angle) -> Mat3 {
    let (x, y, z) = rotation_matrices_xyz(x, y, z);
    (z * y * x).into_fixed()
}

/// Rotation about the Z, then the Y, then the X axis, which stay fixed (`Rx·Ry·Rz`).
///
/// Positive angles rotate counter-clockwise, seen from the positive end of the axis.
pub fn rotate_zyx(x: Angle, y: Angle, z: Angle) -> Mat3 {
    let (x, y, z) = rotation_matrices_xyz(x, y, z);
    (x * y * z).into_fixed()
}

/// Rotation matrix to orient a vector
pub fn orient_to_z(target: Scalar3) -> Mat3 {
    let z_axis = DVec3::Z;
    let target = DVec3::from_fixed(target).normalize();

    // Handle edge case where target is already Z
    if (target - z_axis).length_squared() < 1e-6 {
        return Mat3::from_float(DMat3::IDENTITY);
    }

    // Handle 180-degree rotation (target is -Z)
    if (target + z_axis).length_squared() < 1e-6 {
        // Rotate 180° around any axis perpendicular to Z.
        // For stability, pick X if possible, otherwise Y.
        let perp_axis = if z_axis.cross(DVec3::X).length_squared() > 1e-6 {
            DVec3::X
        } else {
            DVec3::Y
        };
        return Mat3::from_float(trig::rotation_axis(perp_axis, std::f64::consts::PI));
    }

    // Normal case
    let rotation_axis = z_axis.cross(target).normalize();
    let dot = z_axis.dot(target).clamp(-1.0, 1.0); // avoid NaNs
    let angle = libm::acos(dot);

    trig::rotation_axis(rotation_axis, angle).into_fixed()
}

pub fn align_vectors_rodrigues(from: Scalar3, to: Scalar3) -> Mat3 {
    let from = DVec3::from_fixed(from).normalize();
    let to = DVec3::from_fixed(to).normalize();

    let c = from.dot(to);

    if c > 1.0 - 1e-6 {
        return DMat3::IDENTITY.into_fixed();
    }
    if c < -1.0 + 1e-6 {
        let perp_axis = if from.x.abs() > 0.9 {
            DVec3::Y
        } else {
            DVec3::X
        };
        let axis = from.cross(perp_axis).normalize();
        return trig::rotation_axis(axis, std::f64::consts::PI).into_fixed();
    }

    let v = from.cross(to);

    // Skew-symmetric cross-product matrix [v]x
    let vx = DMat3::from_cols_array(&[0.0, v.z, -v.y, -v.z, 0.0, v.x, v.y, -v.x, 0.0]);

    // R = I + vx + vx^2 * (1 / (1 + c))
    let k = 1.0 / (1.0 + c);
    (DMat3::IDENTITY + vx + (vx * vx) * k).into_fixed()
}

pub fn mat3_to_mat4(mat3: Mat3) -> Mat4 {
    let column = |c: crate::Scalar3| Vec4::new(c.x, c.y, c.z, Scalar::ZERO);
    Mat4::from_cols(
        column(mat3.x),
        column(mat3.y),
        column(mat3.z),
        Vec4::new(Scalar::ZERO, Scalar::ZERO, Scalar::ZERO, Scalar::ONE),
    )
}
