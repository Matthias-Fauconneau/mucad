// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The vectors of fixed point scalars.
//!
//! They only hold values, the math converts them to the floating point vectors of glam, see
//! [`crate::math::float`].

use glam::DVec3;
use serde::{Deserialize, Serialize};

use crate::{Length, Scalar, math::{FromFixed, FromFloat, ScalarF}};

/// 2D vector type.
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: Scalar,
    pub y: Scalar,
}

/// 3D vector type.
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scalar3 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

/// 3D vector type.
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coord3 {
    pub x: Length,
    pub y: Length,
    pub z: Length,
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

impl Scalar3 {
    /// A vector of its components.
    pub const fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self { x, y, z }
    }

    /// The components.
    pub const fn to_array(self) -> [Scalar; 3] {
        [self.x, self.y, self.z]
    }
}

impl Coord3 {
    /// A vector of its components.
    pub const fn new(x: Length, y: Length, z: Length) -> Self {
        Self { x, y, z }
    }

    /// The components.
    pub const fn to_array(self) -> [Length; 3] {
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
impl_vector!(Scalar3, 3, "Scalar3", [x, y, z]);
impl_vector!(Vec4, 4, "Vector4", [x, y, z, w]);

impl From<[Length; 3]> for Coord3 {
    fn from([x,y,z]: [Length; 3]) -> Self {
        Self { x,y,z }
    }
}

impl From<Coord3> for [Length; 3] {
    fn from(vector: Coord3) -> Self {
        vector.to_array()
    }
}

impl std::fmt::Debug for Coord3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Coord3 ")?;
        std::fmt::Debug::fmt(&self.to_array(), f)
    }
}

impl FromFixed<Coord3> for DVec3 {
    #[inline]
    fn from_fixed(val: Coord3) -> Self {
        let array: [Length; _] = val.into();
        array.map(ScalarF::from_fixed).into()
    }
}

impl FromFloat<DVec3> for Coord3 {
    #[inline]
    fn from_float(val: DVec3) -> Self {
        val.to_array().map(Length::from_float).into()
    }
}
