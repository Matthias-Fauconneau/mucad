// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The zoom of an orthographic view: the shown detail of the overview of the scene.

use glam::{DVec2, Mat4, Vec2};

use crate::Rect;

/// `rect` scaled by `zoom_factor` around `pivot`, which stays in place.
fn zoomed_rect(rect: &Rect, zoom_factor: f64, pivot: DVec2) -> Rect {
    Rect::from_corners(
        (rect.top_left() - pivot) * zoom_factor + pivot,
        (rect.bottom_right() - pivot) * zoom_factor + pivot,
    )
}

/// A change of a [`ZoomRectModel`], which a viewer reacts to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomEvent {
    /// The detail changed.
    DetailChanged,
    /// The overview changed.
    OverviewChanged,
    /// The zoom level changed.
    ZoomLevelChanged,
}

/// The zoom of an orthographic view in the coordinates of the camera.
///
/// The overview covers the whole scene, the detail is the shown part of it, which is the
/// projection of the camera. The zoom level is how often the detail fits into the overview.
///
/// Its changes are collected as [`ZoomEvent`]s, which are taken with
/// [`ZoomRectModel::take_events`].
#[derive(Debug, Clone)]
pub struct ZoomRectModel {
    /// The overview of the scene.
    overview: Rect,
    /// The shown detail.
    detail: Rect,
    min_zoom_level: f64,
    max_zoom_level: f64,
    events: Vec<ZoomEvent>,
}

impl Default for ZoomRectModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ZoomRectModel {
    /// A model without overview and detail, see [`ZoomRectModel::initialize`].
    pub fn new() -> Self {
        Self {
            overview: Rect::default(),
            detail: Rect::default(),
            min_zoom_level: 0.25,
            max_zoom_level: 160.0,
            events: Vec::new(),
        }
    }

    /// The changes since the last call.
    pub fn take_events(&mut self) -> Vec<ZoomEvent> {
        std::mem::take(&mut self.events)
    }

    /// Show the whole bounds of `radius` in a window with `window_size`.
    pub fn initialize(&mut self, radius: f64, window_size: DVec2) {
        let r = radius;
        let aspect = window_size.x / window_size.y;
        self.overview = Rect::new(-r * aspect, -r, 2.0 * r * aspect, 2.0 * r);
        self.set_detail(self.overview);
        self.events.push(ZoomEvent::OverviewChanged);
    }

    /// The overview of the scene.
    pub fn overview(&self) -> Rect {
        self.overview
    }

    /// Scale the overview around its center by `scale_factor`.
    pub fn scale_overview(&mut self, scale_factor: f64) {
        let overview = zoomed_rect(&self.overview, scale_factor, self.overview.center());
        if !self.overview.fuzzy_eq(&overview) {
            self.overview = overview;
            self.events.push(ZoomEvent::OverviewChanged);
        }
        self.events.push(ZoomEvent::ZoomLevelChanged);
    }

    /// The shown detail.
    pub fn detail(&self) -> Rect {
        self.detail
    }

    /// Set the shown detail.
    pub fn set_detail(&mut self, detail: Rect) {
        if self.detail.fuzzy_eq(&detail) {
            return;
        }
        let zoom_level_changed = self.zoom_level() != self.overview.width / detail.width;
        self.detail = detail;
        if zoom_level_changed {
            // Keeps the zoom level between the minimum and the maximum.
            self.set_zoom_level(self.zoom_level());
        }
        self.events.push(ZoomEvent::DetailChanged);
    }

    /// Move the detail by `d`, also out of the overview, so that panning follows the mouse
    /// exactly at any zoom level.
    pub fn translate(&mut self, d: Vec2) {
        self.set_detail(self.detail.translated(d.as_dvec2()));
    }

    /// Scale the detail by `zoom_factor` around `p`, a factor below 1 zooms in.
    ///
    /// Nothing is zoomed in at the maximum zoom level, or out at the minimum one.
    pub fn zoom_to_point(&mut self, p: DVec2, zoom_factor: f64) {
        let zoom_level = self.zoom_level();
        if (zoom_factor < 1.0 && zoom_level >= self.max_zoom_level)
            || (zoom_factor > 1.0 && zoom_level <= self.min_zoom_level)
        {
            return;
        }
        self.set_detail(zoomed_rect(&self.detail, zoom_factor, p));
    }

    /// Show the whole overview with the zoom level 1.
    pub fn reset(&mut self) {
        self.set_zoom_level(1.0);
    }

    /// How often the width of the detail fits into the width of the overview.
    pub fn zoom_level(&self) -> f64 {
        self.overview.width / self.detail.width
    }

    /// Set the zoom level around the center of the detail, within the minimum and maximum.
    pub fn set_zoom_level(&mut self, zoom_level: f64) {
        let zoom_level = zoom_level.clamp(self.min_zoom_level, self.max_zoom_level);
        let width = self.overview.width / zoom_level;
        let size = DVec2::new(width, width / self.aspect_ratio());
        self.set_detail(Rect::from_top_left_size(
            self.detail.center() - 0.5 * size,
            size,
        ));
        self.events.push(ZoomEvent::ZoomLevelChanged);
    }

    /// Set the zoom level around `pivot`, which stays in place.
    pub fn set_zoom_level_at(&mut self, new_zoom_level: f64, pivot: DVec2) {
        let zoom_factor = self.zoom_level() / new_zoom_level;
        self.set_detail(zoomed_rect(&self.detail, zoom_factor, pivot));
    }

    /// Show the rectangle `r` in the middle of the detail, which keeps its aspect ratio.
    ///
    /// The detail is scaled to fit `r` and then by `factor`, e.g. 1 fits `r` exactly and 1.1
    /// leaves a margin. The zoom level stays between the minimum and the maximum.
    pub fn zoom_to_rect(&mut self, r: Rect, factor: f64) {
        let r = r.normalized();
        let zoom_factor = (r.width / self.detail.width).max(r.height / self.detail.height) * factor;
        let size = self.detail.size() * zoom_factor;
        self.set_detail(Rect::from_top_left_size(r.center() - 0.5 * size, size));
    }

    /// The ratio of the width to the height of the detail.
    pub fn aspect_ratio(&self) -> f64 {
        self.detail.width / self.detail.height
    }

    /// Adapt the overview and the detail to a window with `aspect_ratio`, e.g. after resizing
    /// it: their widths change around their centers, their heights and the zoom level stay.
    pub fn fit_window_aspect_ratio(&mut self, aspect_ratio: f64) {
        let fit = |rect: Rect| {
            let width = rect.height * aspect_ratio;
            Rect::new(rect.center().x - width / 2.0, rect.y, width, rect.height)
        };
        let overview = fit(self.overview);
        if !self.overview.fuzzy_eq(&overview) {
            self.overview = overview;
            self.events.push(ZoomEvent::OverviewChanged);
        }
        self.set_detail(fit(self.detail));
    }

    /// The orthographic projection of the detail, like `QMatrix4x4::ortho`.
    pub fn detail_ortho_projection_matrix(&self) -> Mat4 {
        ortho(&self.detail)
    }
}

/// The orthographic projection of `rect`, its top is the bottom of the view, like
/// `QMatrix4x4::ortho(left, right, top, bottom, …)`.
fn ortho(rect: &Rect) -> Mat4 {
    Mat4::orthographic_rh_gl(
        rect.left() as f32,
        rect.right() as f32,
        rect.top() as f32,
        rect.bottom() as f32,
        0.1,
        100000.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> ZoomRectModel {
        let mut model = ZoomRectModel::new();
        model.initialize(100.0, DVec2::new(400.0, 200.0));
        model
    }

    #[test]
    fn zoomed() {
        let rect = zoomed_rect(&Rect::new(0.0, 0.0, 2.0, 2.0), 2.0, DVec2::new(1.0, 1.0));
        assert_eq!(rect, Rect::new(-1.0, -1.0, 4.0, 4.0));
    }

    #[test]
    fn initialize() {
        let mut model = model();
        assert_eq!(model.overview(), Rect::new(-200.0, -100.0, 400.0, 200.0));
        assert_eq!(model.detail(), model.overview());
        assert_eq!(model.zoom_level(), 1.0);
        assert_eq!(
            model.take_events(),
            [
                ZoomEvent::ZoomLevelChanged,
                ZoomEvent::DetailChanged,
                ZoomEvent::OverviewChanged,
            ]
        );
        assert!(model.take_events().is_empty());
    }

    #[test]
    fn zoom_to_point() {
        let mut model = model();
        model.zoom_to_point(DVec2::new(100.0, 0.0), 0.5);
        assert_eq!(model.zoom_level(), 2.0);
        assert_eq!(model.detail(), Rect::new(-50.0, -50.0, 200.0, 100.0));

        // Not further than the maximum zoom level.
        model.max_zoom_level = 4.0;
        model.zoom_to_point(DVec2::ZERO, 0.5);
        model.zoom_to_point(DVec2::ZERO, 0.5);
        assert_eq!(model.zoom_level(), 4.0);
    }

    #[test]
    fn zoom_level() {
        let mut model = model();
        model.set_zoom_level(2.0);
        assert_eq!(model.detail(), Rect::new(-100.0, -50.0, 200.0, 100.0));
        model.set_zoom_level(1000.0);
        assert_eq!(model.zoom_level(), model.max_zoom_level);
        model.reset();
        assert_eq!(model.detail(), model.overview());
    }

    #[test]
    fn translate() {
        let mut model = model();
        model.set_zoom_level(2.0);
        model.translate(Vec2::new(10.0, -10.0));
        assert_eq!(model.detail(), Rect::new(-90.0, -60.0, 200.0, 100.0));
    }

    #[test]
    fn zoom_to_rect() {
        let mut model = model();
        // A tall rectangle is fitted by its height into the 2:1 detail.
        model.zoom_to_rect(Rect::new(10.0, 10.0, 20.0, 50.0), 1.0);
        assert_eq!(model.detail(), Rect::new(-30.0, 10.0, 100.0, 50.0));
        assert_eq!(model.aspect_ratio(), 2.0);
        assert_eq!(model.zoom_level(), 4.0);
        // A factor leaves a margin around the rectangle.
        model.zoom_to_rect(Rect::new(10.0, 10.0, 20.0, 50.0), 2.0);
        assert_eq!(model.detail(), Rect::new(-80.0, -15.0, 200.0, 100.0));
        // Not further than the maximum zoom level.
        model.zoom_to_rect(Rect::new(0.0, 0.0, 0.01, 0.01), 1.0);
        assert_eq!(model.zoom_level(), model.max_zoom_level);
    }

    #[test]
    fn fit_window_aspect_ratio() {
        let mut model = model();
        model.set_zoom_level(2.0);
        model.take_events();
        // From 2:1 to 1:1, the heights stay.
        model.fit_window_aspect_ratio(1.0);
        assert_eq!(model.overview(), Rect::new(-100.0, -100.0, 200.0, 200.0));
        assert_eq!(model.detail(), Rect::new(-50.0, -50.0, 100.0, 100.0));
        assert_eq!(model.zoom_level(), 2.0);
        let events = model.take_events();
        assert!(events.contains(&ZoomEvent::DetailChanged));
        assert!(events.contains(&ZoomEvent::OverviewChanged));
    }

    #[test]
    fn projection() {
        let m = model().detail_ortho_projection_matrix();
        let corner = m.project_point3(glam::Vec3::new(-200.0, -100.0, -0.1));
        assert!(corner.abs_diff_eq(glam::Vec3::new(-1.0, -1.0, -1.0), 1e-6));
    }
}
