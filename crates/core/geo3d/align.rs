// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Builtin align for 3D geometries.

use crate::*;

/// Trait to align a 2D geometry collection with spacing along an axis.
pub trait Align3D<T = Self> {
    /// Align geometry.
    fn align_3d(&self, direction: DVec3, spacing: Length) -> T;
}

impl Align3D for Geometries3D {
    fn align_3d(&self, direction: DVec3, spacing: Length) -> Self {
        Geometries3D::from_iter(self.iter().scan(0.0_f64, |pos, geo3d| {
            let bounds = geo3d.calc_bounds_3d();
            let dir = if direction.length() > f64::EPSILON {
                direction.normalize()
            } else {
                DVec3::ZERO
            };
            let dist = *bounds.distance_center_to_boundary(dir);
            let c = bounds.center();

            let d = (*pos + dist) * dir - dir * c;
            *pos += 2.0 * dist + *spacing;

            Some(geo3d.transformed_3d(&DMat4::from_translation(d)))
        }))
    }
}
