// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Floating point/fixed point conversion handling.
//!
//! All values in µcad are fixed point. Use these traits and their impls to convert them to floating point types and back.
//! The default floating point type `ScalarF` is `f64`:

use glam::{DMat2, DMat3, DMat4, DVec2, DVec3, DVec4};

use crate::{Angle, Mat2, Mat3, Mat4, Scalar, Vec2, Vec3, Vec4};

/// Floating point scalar
pub type ScalarF = f64;

/// Quantized conversion back into exact fixed-point representation.
pub trait FromFloat<Source> {
    fn from_float(val: Source) -> Self;
}

/// Convert a float from fixed point number.
pub trait FromFixed<Source> {
    fn from_fixed(val: Source) -> Self;
}

/// Lossy conversion to floating-point representation.
pub trait IntoFloat<Target> {
    fn into_float(self) -> Target;
}

impl<T, Target> IntoFloat<Target> for T
where
    Target: FromFixed<T>,
{
    #[inline]
    fn into_float(self) -> Target {
        Target::from_fixed(self)
    }
}

/// Lossy conversion to floating-point representation.
pub trait IntoFixed<Target> {
    fn into_fixed(self) -> Target;
}

impl<T, Target> IntoFixed<Target> for T
where
    Target: FromFloat<T>,
{
    #[inline]
    fn into_fixed(self) -> Target {
        Target::from_float(self)
    }
}

// --- Scalar Implementations ---

impl FromFixed<Scalar> for ScalarF {
    #[inline]
    fn from_fixed(val: Scalar) -> Self {
        val.to_num()
    }
}

impl FromFloat<ScalarF> for Scalar {
    #[inline]
    fn from_float(val: ScalarF) -> Self {
        Scalar::from_num(val)
    }
}

// --- Angle Implementations ---

/// An angle in radians, like the angles of glam.
impl FromFixed<Angle> for ScalarF {
    #[inline]
    fn from_fixed(val: Angle) -> Self {
        ScalarF::from_fixed(val.0)
    }
}

/// An angle of radians.
impl FromFloat<ScalarF> for Angle {
    #[inline]
    fn from_float(val: ScalarF) -> Self {
        cgmath::Rad(Scalar::from_num(val))
    }
}

// --- Vector and Matrix Implementations ---

/// The conversions of a fixed point vector from and to its floating point one, through their
/// arrays.
macro_rules! impl_vec_convert {
    ($fixed_type:ident, $float_type:ident) => {
        impl FromFixed<$fixed_type> for $float_type {
            #[inline]
            fn from_fixed(val: $fixed_type) -> Self {
                let array: [Scalar; _] = val.into();
                array.map(ScalarF::from_fixed).into()
            }
        }

        impl FromFloat<$float_type> for $fixed_type {
            #[inline]
            fn from_float(val: $float_type) -> Self {
                val.to_array().map(Scalar::from_float).into()
            }
        }
    };
}

impl_vec_convert!(Vec2, DVec2);
impl_vec_convert!(Vec3, DVec3);
impl_vec_convert!(Vec4, DVec4);

/// The conversions of a fixed point matrix from and to its floating point one, through the
/// arrays of their columns.
macro_rules! impl_matrix_convert {
    ($fixed_type:ident, $float_type:ident) => {
        impl FromFixed<$fixed_type> for $float_type {
            #[inline]
            fn from_fixed(val: $fixed_type) -> Self {
                let columns: [[Scalar; _]; _] = val.into();
                $float_type::from_cols_array_2d(&columns.map(|c| c.map(ScalarF::from_fixed)))
            }
        }

        impl FromFloat<$float_type> for $fixed_type {
            #[inline]
            fn from_float(val: $float_type) -> Self {
                val.to_cols_array_2d()
                    .map(|c| c.map(Scalar::from_float))
                    .into()
            }
        }
    };
}

impl_matrix_convert!(Mat2, DMat2);
impl_matrix_convert!(Mat3, DMat3);
impl_matrix_convert!(Mat4, DMat4);
