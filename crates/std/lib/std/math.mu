// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

pub use __mu::math::PI;
pub use __mu::math::X;
pub use __mu::math::Y;
pub use __mu::math::Z;
pub use __mu::math::sqrt;
pub use __mu::math::pow;
pub use __mu::math::abs;
pub use __mu::math::sin;
pub use __mu::math::cos;
pub use __mu::math::int;
pub use __mu::math::floor;
pub use __mu::math::round;
pub use __mu::math::fract;
pub use __mu::math::rotate_around_axis;
pub use __mu::math::rotate_xyz;
pub use __mu::math::rotate_zyx;
pub use __mu::math::min;
pub use __mu::math::max;
pub use __mu::math::clamp;

/// Perform smooth Hermite interpolation between 0 and 1.
///
/// Returns 0 when `x <= edge0`, 1 when `x >= edge1`, and smoothly
/// interpolates between 0 and 1 for values of `x` between `edge0` and `edge1`.
/// The result is a cubic Hermite curve with zero first-order derivatives at both edges.
///
/// ## Parameters
/// - `edge0`: The lower edge of the interpolation range.
/// - `edge1`: The upper edge of the interpolation range.
/// - `x`: The value to interpolate.
///
/// ## Examples
///
/// ```mu,smoothstep_below?mode=compile
/// std::assert_eq(math::smoothstep(0.0, 1.0, 0.0), 0.0);
/// std::assert_eq(math::smoothstep(0.0, 1.0, -1.0), 0.0);
/// ```
///
/// ```mu,smoothstep_above?mode=compile
/// std::assert_eq(math::smoothstep(0.0, 1.0, 1.0), 1.0);
/// std::assert_eq(math::smoothstep(0.0, 1.0, 2.0), 1.0);
/// ```
///
/// ```mu,smoothstep_mid?mode=compile
/// std::assert_eq(math::smoothstep(0.0, 1.0, 0.5), 0.5);
/// ```
///
/// ```mu,smoothstep_quarter?mode=compile
/// std::assert_eq(math::smoothstep(0.0, 1.0, 0.25), 0.15625);
/// std::assert_eq(math::smoothstep(0.0, 1.0, 0.75), 0.84375);
/// ```
pub fn smoothstep(edge0: Scalar, edge1: Scalar, x: Scalar) -> Scalar {
    t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
