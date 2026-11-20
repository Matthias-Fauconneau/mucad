// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad core

mod angle;
pub mod boolean_op;
pub mod bounds;
pub mod color;
pub mod core_error;
pub mod geo2d;
pub mod geo3d;
pub mod geometry;
pub mod length;
pub mod svg;
pub mod traits;
pub mod triangle;

/// Primitive integer type.
pub type Integer = i64;
/// Primitive floating point type.
pub type Scalar = f64;
/// Angle type in radians.
pub use angle::Angle;
/// The vectors and matrices of glam: of scalars, e.g. [`DVec3`], and of `f32` for meshes, i.e.
/// [`Vec3`].
pub use glam::{DMat2, DMat3, DMat4, DVec2, DVec3, DVec4, Vec3};
/// Length type.
pub use length::Length;

/// Constants.
pub mod consts {
    pub use std::f64::consts::PI;
    pub use std::f64::consts::TAU;
}

pub use boolean_op::{BooleanOp, UnaryBooleanOp};

pub use bounds::*;
pub use color::*;
pub use core_error::*;
pub use geo2d::*;
pub use geo3d::*;
pub use geometry::{Geometry, GeometryType};
pub use triangle::Triangle;

use crate::traits::TransformAffine;

/// Convert a Matrix4 to Matrix3.
pub fn mat4_to_mat3(m: &DMat4) -> DMat3 {
    let [x, y, _, w] = [m.x_axis, m.y_axis, m.z_axis, m.w_axis].map(|c| DVec3::new(c.x, c.y, c.w));
    DMat3::from_cols(x, y, w)
}

/// Convert a Matrix3 to Matrix4.
pub fn mat3_to_mat4(m: &DMat3) -> DMat4 {
    let (x, y) = (m.x_axis, m.y_axis);
    DMat4::from_cols_array(&[
        x.x, x.y, 0.0, x.z, // First column: X basis + X translation
        y.x, y.y, 0.0, y.z, // Second column: Y basis + Y translation
        0.0, 0.0, 1.0, 0.0, // Z axis: identity (no change)
        0.0, 0.0, 0.0, 1.0, // Homogeneous row
    ])
}
