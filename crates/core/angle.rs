// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Angles in radians.

use crate::Scalar;

/// An angle in radians, e.g. of a rotation.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Angle(pub Scalar);

impl Angle {
    /// The full turn of 360°.
    pub const FULL_TURN: Self = Self(std::f64::consts::TAU);

    /// The angle of `degrees`.
    pub fn from_degrees(degrees: Scalar) -> Self {
        Self(degrees.to_radians())
    }

    /// The angle in degrees.
    pub fn to_degrees(self) -> Scalar {
        self.0.to_degrees()
    }

    /// The sine of the angle.
    pub fn sin(self) -> Scalar {
        self.0.sin()
    }

    /// The cosine of the angle.
    pub fn cos(self) -> Scalar {
        self.0.cos()
    }
}

impl std::ops::Add for Angle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Angle {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl std::ops::Mul<Scalar> for Angle {
    type Output = Self;

    fn mul(self, rhs: Scalar) -> Self {
        Self(self.0 * rhs)
    }
}

impl std::ops::Div<Scalar> for Angle {
    type Output = Self;

    fn div(self, rhs: Scalar) -> Self {
        Self(self.0 / rhs)
    }
}
