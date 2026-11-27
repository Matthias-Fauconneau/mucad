// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

// FIXME: grouped items are not supported yet : use std::{list::count, math::{norm, pow}, geo2d::PolyBezier};
use std::list::count;
use std::math::norm;
use std::math::pow;
use std::geo2d::PolyBezier;

// Converts a 4-point Catmull-Rom stencil (p_prev, p0, p1, p2) into one cubic Bézier segment (p0, c1, c2, p1).
// tension = 0.0 -> standard centripetal Catmull-Rom; tension = 1.0 -> straight line segments.
fn catmull_rom_segment(
    p_prev: Vec2,
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    alpha: Scalar = 0.5,
    tension: Scalar = 0.0
) -> CubicSegment2D {
    d1 = pow(norm(p0 - p_prev) / 1mm, alpha);
    d2 = pow(norm(p1 - p0) / 1mm, alpha);
    d3 = pow(norm(p2 - p1) / 1mm, alpha);

    scale = (1.0 - tension) / 3.0;

    // Outgoing handle c1 from p0 (guarding d1 == 0 avoids NaN & lets repeated points create sharp corners)
    m1 = (d2 * d2 * (p0 - p_prev) + d1 * d1 * (p1 - p0)) / (d1 * (d1 + d2));
    c1 = if d1 < 1e-9 /*| d2 < 1e-9*/ {
        p0
    } else {
        p0 + m1 * scale
    };

    // Incoming handle c2 into p1
    m2 = (d2 * d2 * (p1 - p2) + d3 * d3 * (p0 - p1)) / (d3 * (d2 + d3));
    c2 = if d3 < 1e-9 /*d2 < 1e-9*/ {
        p1
    } else {
        p1 + m2 * scale
    };

    (p0, c1, c2, p1)
}

fn catmull_rom_segment_i(
    points: [Vec2],
    i: Integer,
    closed: Bool = true,
    tension: Scalar = 0.0
) -> CubicSegment2D {
    n = count(points);
    p0 = points[i];
    p1 = points[(i + 1) rem n];

    // Periodic wrap for closed loops; linear reflection for open endpoints
    p_prev = if i > 0 {
        points[i - 1]
    } else if closed {
        points[n - 1]
    } else {
        p0 + (p0 - p1)
    };

    p2 = if i + 2 < n {
        points[i + 2]
    } else if closed {
        points[(i + 2) rem n]
    } else {
        p1 + (p1 - p0)
    };

    catmull_rom_segment(p_prev, p0, p1, p2, alpha = 0.5, tension)
}

pub sketch Spline(
    points: [Vec2],
    closed: Bool = true,
    tension: Scalar = 0.0
) {
    n = count(points);
    //std::assert(if closed { n >= 3 } else { n >= 2 }, "Spline requires at least 2 points (3 if closed)");
    seg_count = if closed { n } else { n - 1 };
    PolyBezier(catmull_rom_segment_i(points, [0..seg_count - 1], closed, tension), closed);
}