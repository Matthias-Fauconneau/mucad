// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! 2D line geometry.
use geo::AffineOps;

use crate::*;

/// A 2D line type.
#[derive(Debug, Clone)]
pub struct Line(pub geo2d::Point, pub geo2d::Point);

impl Line {
    /// Write a new line
    pub fn new<T: Into<geo2d::Point>>(p0: T, p1: T) -> Self {
        Self(p0.into(), p1.into())
    }

    /// Shorten edge on both ends by a certain amount.
    pub fn shorter(&self, amount: Scalar) -> Self {
        let d = self.vec();
        let d = 0.5 * d * (1.0 - amount / d.length());
        let c = self.center();
        Self(c - (d.x, d.y).into(), c + (d.x, d.y).into())
    }

    /// Calculate the radius edge from a circle.
    ///
    /// An edge from the circle's center to a point on its periphery.
    pub fn radius_edge(circle: &Circle, angle: &Angle) -> Self {
        let Circle {
            radius,
            offset: DVec2 { x, y },
        } = &circle;
        let x1 = x + radius * angle.cos();
        let y1 = y + radius * angle.sin();
        Self(geo::Point::new(*x, *y), geo::Point::new(x1, y1))
    }

    /// Return vector of this edge.
    pub fn vec(&self) -> DVec2 {
        DVec2::from(self.1.x_y()) - DVec2::from(self.0.x_y())
    }

    /// Return center of this edge.
    pub fn center(&self) -> geo2d::Point {
        (self.0 + self.1) * 0.5
    }

    /// Calculate angle in radians.
    pub fn angle(&self) -> Angle {
        let (p1, p2) = (self.0, self.1);
        let dx = p2.x() - p1.x();
        let dy = p2.y() - p1.y();
        Angle(dy.atan2(dx))
    }

    /// 2D Transformation matrix for this edge (rotation and offset).
    pub fn matrix(&self) -> DMat3 {
        let rot = DMat2::from_angle(self.angle().0);
        let mut m = DMat3::from_translation(self.0.x_y().into());
        m.x_axis = DVec3::new(rot.x_axis.x, rot.x_axis.y, 0.0);
        m.y_axis = DVec3::new(rot.y_axis.x, rot.y_axis.y, 0.0);
        m
    }
}

impl CalcBounds2D for Line {
    fn calc_bounds_2d(&self) -> geo2d::Bounds2D {
        geo2d::Bounds2D::new(self.0.x_y(), self.1.x_y())
    }
}

impl Transformed2D for Line {
    fn transformed_2d(&self, mat: &DMat3) -> Self {
        let transform = &geo2d::mat3_to_affine_transform(mat);
        Self(
            self.0.affine_transform(transform),
            self.1.affine_transform(transform),
        )
    }
}
