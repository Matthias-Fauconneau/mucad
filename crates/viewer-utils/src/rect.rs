// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A rectangle like `QRectF` of Qt: a top left corner and a size, which can be negative.

use glam::DVec2;

/// A rectangle with its top left corner at the smallest coordinates, like `QRectF` of Qt.
///
/// The "top" is the smaller y coordinate, also when y points upwards, e.g. in a view.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    /// The x coordinate of the left edge.
    pub x: f64,
    /// The y coordinate of the top edge.
    pub y: f64,
    /// The width, which is negative, if the rectangle is not normalized.
    pub width: f64,
    /// The height, which is negative, if the rectangle is not normalized.
    pub height: f64,
}

impl Rect {
    /// A rectangle at `(x, y)` with `width` and `height`.
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// The rectangle from `top_left` to `bottom_right`.
    pub fn from_corners(top_left: DVec2, bottom_right: DVec2) -> Self {
        let size = bottom_right - top_left;
        Self::new(top_left.x, top_left.y, size.x, size.y)
    }

    /// The rectangle at `top_left` with `size`.
    pub fn from_top_left_size(top_left: DVec2, size: DVec2) -> Self {
        Self::new(top_left.x, top_left.y, size.x, size.y)
    }

    /// The x coordinate of the left edge.
    pub fn left(&self) -> f64 {
        self.x
    }

    /// The x coordinate of the right edge.
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    /// The y coordinate of the top edge.
    pub fn top(&self) -> f64 {
        self.y
    }

    /// The y coordinate of the bottom edge.
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// The top left corner.
    pub fn top_left(&self) -> DVec2 {
        DVec2::new(self.x, self.y)
    }

    /// The bottom right corner.
    pub fn bottom_right(&self) -> DVec2 {
        DVec2::new(self.right(), self.bottom())
    }

    /// The width and the height.
    pub fn size(&self) -> DVec2 {
        DVec2::new(self.width, self.height)
    }

    /// The center point.
    pub fn center(&self) -> DVec2 {
        DVec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// The rectangle moved by `offset`.
    pub fn translated(&self, offset: DVec2) -> Self {
        Self::new(
            self.x + offset.x,
            self.y + offset.y,
            self.width,
            self.height,
        )
    }

    /// The rectangle with `dx1` and `dy1` added to the top left corner, `dx2` and `dy2` to the
    /// bottom right corner.
    pub fn adjusted(&self, dx1: f64, dy1: f64, dx2: f64, dy2: f64) -> Self {
        Self::new(
            self.x + dx1,
            self.y + dy1,
            self.width + dx2 - dx1,
            self.height + dy2 - dy1,
        )
    }

    /// The rectangle with a non-negative width and height, which covers the same area.
    pub fn normalized(&self) -> Self {
        let mut rect = *self;
        if rect.width < 0.0 {
            rect.x += rect.width;
            rect.width = -rect.width;
        }
        if rect.height < 0.0 {
            rect.y += rect.height;
            rect.height = -rect.height;
        }
        rect
    }

    /// Whether the rectangles are equal up to rounding errors, like `==` of `QRectF`.
    pub fn fuzzy_eq(&self, other: &Self) -> bool {
        fuzzy_eq(self.x, other.x)
            && fuzzy_eq(self.y, other.y)
            && fuzzy_eq(self.width, other.width)
            && fuzzy_eq(self.height, other.height)
    }
}

/// Whether `a` and `b` are equal up to rounding errors, like `==` of `QPointF` and `QSizeF`:
/// relative to their size, or absolute if one is 0.
fn fuzzy_eq(a: f64, b: f64) -> bool {
    if a == 0.0 || b == 0.0 {
        (a - b).abs() <= 1e-12
    } else {
        (a - b).abs() * 1e12 <= a.abs().min(b.abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges() {
        let rect = Rect::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(rect.right(), 4.0);
        assert_eq!(rect.bottom(), 6.0);
        assert_eq!(rect.center(), DVec2::new(2.5, 4.0));
        assert_eq!(
            Rect::from_corners(rect.top_left(), rect.bottom_right()),
            rect
        );
    }

    #[test]
    fn normalized() {
        let rect = Rect::new(4.0, 6.0, -3.0, -4.0).normalized();
        assert_eq!(rect, Rect::new(1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn adjusted() {
        let rect = Rect::new(0.0, 0.0, 2.0, 2.0).adjusted(-1.0, -1.0, 1.0, 1.0);
        assert_eq!(rect, Rect::new(-1.0, -1.0, 4.0, 4.0));
    }

    #[test]
    fn fuzzy() {
        let rect = Rect::new(0.0, 1.0, 3.0, 0.1 + 0.2);
        assert!(rect.fuzzy_eq(&Rect::new(1e-13, 1.0, 3.0, 0.3)));
        assert!(!rect.fuzzy_eq(&Rect::new(0.0, 1.0, 3.0, 0.31)));
    }
}
