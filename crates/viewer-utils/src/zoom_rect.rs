// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The zoom of an orthographic view: the shown detail of the overview of the scene.

use glam::{DVec2, Mat4, Vec2};

use crate::Rect;

/// `rect` scaled by `zoom_factor` around `pivot`, which stays in place.
pub fn zoomed_rect(rect: &Rect, zoom_factor: f64, pivot: DVec2) -> Rect {
    // Translate the rectangle so that the pivot point is at the origin
    let zoomed_rect = Rect::from_corners(
        (rect.top_left() - pivot) * zoom_factor,
        (rect.bottom_right() - pivot) * zoom_factor,
    );
    zoomed_rect.translated(pivot)
}

/// A change of a [`ZoomRectModel`], which a viewer reacts to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoomEvent {
    /// The detail is changed to the rectangle.
    DetailChanged(Rect),
    /// The overview is changed to the rectangle.
    OverviewChanged(Rect),
    /// The zoom level is changed to the value.
    ZoomLevelChanged(f64),
}

/// The state of a [`ZoomRectModel`], e.g. to restore a view.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ZoomState {
    /// The shown detail.
    pub detail: Rect,
    /// The overview of the scene.
    pub overview: Rect,
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
    /// The overview and the detail.
    state: ZoomState,
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
            state: ZoomState::default(),
            min_zoom_level: 0.25,
            max_zoom_level: 160.0,
            events: Vec::new(),
        }
    }

    /// The changes since the last call.
    pub fn take_events(&mut self) -> Vec<ZoomEvent> {
        std::mem::take(&mut self.events)
    }

    /// Show the whole scene with `scene_radius` in a window with `window_size`.
    pub fn initialize(&mut self, scene_radius: f64, window_size: DVec2) {
        let r = scene_radius;
        let aspect = window_size.x / window_size.y;

        self.state.overview = Rect::new(-r * aspect, -r, 2.0 * r * aspect, 2.0 * r);
        self.set_detail(self.state.overview);
        self.events
            .push(ZoomEvent::OverviewChanged(self.state.overview));
    }

    /// The overview of the scene.
    pub fn overview(&self) -> Rect {
        self.state.overview
    }

    /// Set the overview of the scene.
    pub fn set_overview(&mut self, overview: Rect) {
        if self.state.overview.fuzzy_eq(&overview) {
            return;
        }
        self.state.overview = overview;
        self.set_detail(self.state.detail); // Update detail rect to fit into new overview rect
        self.events
            .push(ZoomEvent::OverviewChanged(self.state.overview));
    }

    /// Scale the overview around its center by `scale_factor`.
    pub fn scale_overview(&mut self, scale_factor: f64) {
        self.set_overview(zoomed_rect(
            &self.state.overview,
            scale_factor,
            self.state.overview.center(),
        ));
        self.events
            .push(ZoomEvent::ZoomLevelChanged(self.zoom_level()));
    }

    /// The shown detail.
    pub fn detail(&self) -> Rect {
        self.state.detail
    }

    /// Set the shown detail.
    pub fn set_detail(&mut self, detail: Rect) {
        let detail = keep_in_rect(detail, &self.boundary_rect());
        let zoom_level_has_changed = self.zoom_level() != self.state.overview.width / detail.width;
        if self.state.detail.fuzzy_eq(&detail) {
            return;
        }
        self.state.detail = detail;

        if zoom_level_has_changed {
            self.set_zoom_level(self.zoom_level());
        }
        self.events
            .push(ZoomEvent::DetailChanged(self.state.detail));
    }

    /// The detail relative to the overview, from 0 to 1 within the overview.
    pub fn normalized_detail(&self) -> Rect {
        let o = self.overview();
        let d = self.detail().translated(-o.top_left());

        Rect::new(
            d.x / o.width,
            d.y / o.height,
            d.width / o.width,
            d.height / o.height,
        )
        .normalized()
    }

    /// Move the detail by `d`, but not further out of the overview.
    pub fn translate(&mut self, mut d: Vec2) {
        if d.x < 0.0 && self.state.detail.left() < self.state.overview.left() {
            d.x = 0.0;
        }
        if d.x > 0.0 && self.state.detail.right() > self.state.overview.right() {
            d.x = 0.0;
        }
        if d.y < 0.0 && self.state.detail.top() < self.state.overview.top() {
            d.y = 0.0;
        }
        if d.y > 0.0 && self.state.detail.bottom() > self.state.overview.bottom() {
            d.y = 0.0;
        }
        self.set_detail(self.state.detail.translated(d.as_dvec2()));
    }

    /// Move the detail by the offset from `p` to its center.
    pub fn move_to(&mut self, p: DVec2) {
        self.set_detail(self.state.detail.translated(self.state.detail.center() - p));
    }

    /// Scale the detail by `zoom_factor` around `p`, a factor below 1 zooms in.
    ///
    /// Nothing is zoomed in at the maximum zoom level, or out at the minimum one.
    pub fn zoom_to_point(&mut self, p: DVec2, zoom_factor: f64) {
        if zoom_factor < 1.0 && self.zoom_level() >= self.max_zoom_level() {
            return;
        }
        if zoom_factor > 1.0 && self.zoom_level() <= self.min_zoom_level() {
            return;
        }

        self.set_detail(zoomed_rect(&self.state.detail, zoom_factor, p));
    }

    /// Show the whole overview with the zoom level 1.
    pub fn reset(&mut self) {
        self.set_zoom_level(1.0);
    }

    /// How often the width of the detail fits into the width of the overview.
    pub fn zoom_level(&self) -> f64 {
        self.state.overview.width / self.state.detail.width
    }

    /// Set the zoom level around the center of the detail, within the minimum and maximum.
    pub fn set_zoom_level(&mut self, mut zoom_level: f64) {
        if zoom_level < self.min_zoom_level {
            zoom_level = self.min_zoom_level;
        }
        if zoom_level > self.max_zoom_level {
            zoom_level = self.max_zoom_level;
        }

        let new_size = DVec2::new(
            self.state.overview.width / zoom_level,
            self.state.overview.width / self.aspect_ratio() / zoom_level,
        );

        self.set_detail(Rect::from_top_left_size(
            self.state.detail.center() - 0.5 * new_size,
            new_size,
        ));
        self.events.push(ZoomEvent::ZoomLevelChanged(zoom_level));
    }

    /// Set the zoom level around `pivot`, which stays in place.
    pub fn set_zoom_level_at(&mut self, new_zoom_level: f64, pivot: DVec2) {
        let zoom_factor = self.zoom_level() / new_zoom_level;
        self.set_detail(zoomed_rect(&self.state.detail, zoom_factor, pivot));
    }

    /// The maximum zoom level, i.e. how far is zoomed in.
    pub fn max_zoom_level(&self) -> f64 {
        self.max_zoom_level
    }

    /// Set the maximum zoom level, a larger zoom level is reduced to it.
    pub fn set_max_zoom_level(&mut self, max_zoom_level: f64) {
        self.max_zoom_level = max_zoom_level;
        if self.zoom_level() > self.max_zoom_level {
            self.set_zoom_level(self.max_zoom_level);
        }
    }

    /// The minimum zoom level, i.e. how far is zoomed out.
    pub fn min_zoom_level(&self) -> f64 {
        self.min_zoom_level
    }

    /// Set the minimum zoom level, a smaller zoom level is increased to it.
    pub fn set_min_zoom_level(&mut self, min_zoom_level: f64) {
        self.min_zoom_level = min_zoom_level;
        if self.zoom_level() < self.min_zoom_level {
            self.set_zoom_level(self.min_zoom_level);
        }
    }

    /// Show the rectangle `r` in the middle of the detail, which keeps its aspect ratio.
    ///
    /// The detail is scaled to fit `r` and then by `factor`, e.g. 1 fits `r` exactly and 1.1
    /// leaves a margin. The zoom level stays between the minimum and the maximum.
    pub fn zoom_to_rect(&mut self, r: Rect, factor: f64) {
        let r = r.normalized();
        let zoom_factor =
            (r.width / self.state.detail.width).max(r.height / self.state.detail.height) * factor;
        let size = self.state.detail.size() * zoom_factor;
        self.set_detail(Rect::from_top_left_size(r.center() - 0.5 * size, size));
    }

    /// The ratio of the width to the height of the detail.
    pub fn aspect_ratio(&self) -> f64 {
        self.state.detail.width / self.state.detail.height
    }

    /// Set the aspect ratio of the detail around its center, e.g. after resizing the window.
    pub fn set_aspect_ratio(&mut self, new_aspect_ratio: f64) {
        let new_size = DVec2::new(
            self.state.detail.width,
            self.state.detail.width / new_aspect_ratio,
        );
        self.set_detail(Rect::from_top_left_size(
            self.state.detail.center() - 0.5 * new_size,
            new_size,
        ));
    }

    /// Adapt the overview and the detail to a window with `aspect_ratio`, e.g. after resizing
    /// it: their widths change around their centers, their heights and the zoom level stay.
    pub fn fit_window_aspect_ratio(&mut self, aspect_ratio: f64) {
        let fit = |rect: Rect| {
            let width = rect.height * aspect_ratio;
            Rect::new(rect.center().x - width / 2.0, rect.y, width, rect.height)
        };
        let overview = fit(self.state.overview);
        if !self.state.overview.fuzzy_eq(&overview) {
            self.state.overview = overview;
            self.events
                .push(ZoomEvent::OverviewChanged(self.state.overview));
        }
        self.set_detail(fit(self.state.detail));
    }

    /// The detail and the overview, e.g. to restore them later.
    pub fn state(&self) -> ZoomState {
        self.state
    }

    /// Restore the detail and the overview.
    pub fn set_state(&mut self, state: ZoomState) {
        self.set_overview(state.overview);
        self.set_detail(state.detail);
    }

    /// The orthographic projection of the detail, like `QMatrix4x4::ortho`.
    pub fn detail_ortho_projection_matrix(&self) -> Mat4 {
        ortho(&self.state.detail)
    }

    /// The orthographic projection of the overview, like `QMatrix4x4::ortho`.
    pub fn overview_ortho_projection_matrix(&self) -> Mat4 {
        ortho(&self.state.overview)
    }

    /// The left and right edge of [`ZoomRectModel::normalized_detail`].
    pub fn normalized_horizontal_range(&self) -> (f64, f64) {
        let rect = self.normalized_detail();
        (rect.left(), rect.right())
    }

    /// The bottom and top edge of [`ZoomRectModel::normalized_detail`].
    pub fn normalized_vertical_range(&self) -> (f64, f64) {
        let rect = self.normalized_detail();
        (rect.bottom(), rect.top())
    }

    fn boundary_rect(&self) -> Rect {
        let ds = self.state.detail.size();
        self.state.overview.adjusted(-ds.x, -ds.y, ds.x, ds.y)
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

fn keep_in_rect(inner: Rect, rect: &Rect) -> Rect {
    let _ = rect;
    // Check if this function is required
    // TODO
    inner
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
                ZoomEvent::ZoomLevelChanged(1.0),
                ZoomEvent::DetailChanged(model.overview()),
                ZoomEvent::OverviewChanged(model.overview()),
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
        model.set_max_zoom_level(4.0);
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
        assert_eq!(model.zoom_level(), model.max_zoom_level());
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
        assert_eq!(model.zoom_level(), model.max_zoom_level());
    }

    #[test]
    fn normalized_detail() {
        let mut model = model();
        model.set_zoom_level(2.0);
        assert_eq!(model.normalized_detail(), Rect::new(0.25, 0.25, 0.5, 0.5));
        assert_eq!(model.normalized_horizontal_range(), (0.25, 0.75));
        assert_eq!(model.normalized_vertical_range(), (0.75, 0.25));
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
        assert!(events.contains(&ZoomEvent::DetailChanged(model.detail())));
        assert!(events.contains(&ZoomEvent::OverviewChanged(model.overview())));
    }

    #[test]
    fn state() {
        let mut model = model();
        let state = model.state();
        model.set_zoom_level(3.0);
        model.set_state(state);
        assert_eq!(model.state(), state);
    }

    #[test]
    fn projection() {
        let m = model().detail_ortho_projection_matrix();
        let corner = m.project_point3(glam::Vec3::new(-200.0, -100.0, -0.1));
        assert!(corner.abs_diff_eq(glam::Vec3::new(-1.0, -1.0, -1.0), 1e-6));
    }
}
