// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rotation matrices, which are the same on all platforms.

use glam::{DMat3, DVec3};

use super::ScalarF;

/// A rotation about the X axis by `angle` in radians.
pub fn rotation_x(angle: ScalarF) -> DMat3 {
    let (s, c) = libm::sincos(angle);
    DMat3::from_cols_array(&[1.0, 0.0, 0.0, 0.0, c, s, 0.0, -s, c])
}

/// A rotation about the Y axis by `angle` in radians.
pub fn rotation_y(angle: ScalarF) -> DMat3 {
    let (s, c) = libm::sincos(angle);
    DMat3::from_cols_array(&[c, 0.0, -s, 0.0, 1.0, 0.0, s, 0.0, c])
}

/// A rotation about the Z axis by `angle` in radians.
pub fn rotation_z(angle: ScalarF) -> DMat3 {
    let (s, c) = libm::sincos(angle);
    DMat3::from_cols_array(&[c, s, 0.0, -s, c, 0.0, 0.0, 0.0, 1.0])
}

/// A rotation about `axis`, which is a unit vector, by `angle` in radians.
pub fn rotation_axis(axis: DVec3, angle: ScalarF) -> DMat3 {
    let (s, c) = libm::sincos(angle);
    let one_minus_c = 1.0 - c;
    DMat3::from_cols_array(&[
        one_minus_c * axis.x * axis.x + c,
        one_minus_c * axis.x * axis.y + s * axis.z,
        one_minus_c * axis.x * axis.z - s * axis.y,
        one_minus_c * axis.x * axis.y - s * axis.z,
        one_minus_c * axis.y * axis.y + c,
        one_minus_c * axis.y * axis.z + s * axis.x,
        one_minus_c * axis.x * axis.z + s * axis.y,
        one_minus_c * axis.y * axis.z - s * axis.x,
        one_minus_c * axis.z * axis.z + c,
    ])
}
