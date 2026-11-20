// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A bounding sphere, e.g. of a model.

use glam::Vec3;

use crate::Scalar;

/// A sphere around a model, e.g. the bounds of what a camera shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingSphere {
    /// The radius around the position.
    pub radius: Scalar,
    /// The position of the center.
    pub pos: Vec3,
}

impl BoundingSphere {
    /// A sphere with `radius` around `pos`.
    pub const fn new(pos: Vec3, radius: Scalar) -> Self {
        Self { radius, pos }
    }
}
