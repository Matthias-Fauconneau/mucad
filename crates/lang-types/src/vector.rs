// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The vectors of fixed point scalars.
//!
//! They only hold values, the math converts them to the floating point vectors of glam, see
//! [`crate::math::float`].

use serde::{Deserialize, Serialize};

use crate::Scalar;

/// 2D vector type.
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: Scalar,
    pub y: Scalar,
}

/// 3D vector type.
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

/// 4D vector type.
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vec4 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
    pub w: Scalar,
}

impl Vec2 {
    /// A vector of its components.
    pub const fn new(x: Scalar, y: Scalar) -> Self {
        Self { x, y }
    }

    /// The components.
    pub const fn to_array(self) -> [Scalar; 2] {
        [self.x, self.y]
    }
}

impl Vec3 {
    /// A vector of its components.
    pub const fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self { x, y, z }
    }

    /// The components.
    pub const fn to_array(self) -> [Scalar; 3] {
        [self.x, self.y, self.z]
    }
}

impl Vec4 {
    /// A vector of its components.
    pub const fn new(x: Scalar, y: Scalar, z: Scalar, w: Scalar) -> Self {
        Self { x, y, z, w }
    }

    /// The components.
    pub const fn to_array(self) -> [Scalar; 4] {
        [self.x, self.y, self.z, self.w]
    }
}

/// The conversions of a vector from and to the array of its components, and its `Debug`
/// format, e.g. `Vector3 [1, 2, 3]`.
macro_rules! impl_vector {
    ($vector:ident, $n:literal, $name:literal, [$($component:ident),+]) => {
        impl From<[Scalar; $n]> for $vector {
            fn from([$($component),+]: [Scalar; $n]) -> Self {
                Self { $($component),+ }
            }
        }

        impl From<$vector> for [Scalar; $n] {
            fn from(vector: $vector) -> Self {
                vector.to_array()
            }
        }

        impl std::fmt::Debug for $vector {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, concat!($name, " "))?;
                std::fmt::Debug::fmt(&self.to_array(), f)
            }
        }
    };
}

impl_vector!(Vec2, 2, "Vector2", [x, y]);
impl_vector!(Vec3, 3, "Vector3", [x, y, z]);
impl_vector!(Vec4, 4, "Vector4", [x, y, z, w]);
