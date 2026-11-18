// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The camera controller of an orthographic view.

use std::time::Duration;

use glam::{DVec2, Vec2, Vec3, Vec4, Vec4Swizzles};

use super::{CameraControl, CameraControllerState, CameraEvent, Mode, Ray, scaled_by_time};
use crate::{BoundingSphere, Rect, ZoomEvent, ZoomRectModel};

/// A camera controller for an orthographic view.
///
/// The camera orbits around a pivot. Translating and zooming change the detail of the
/// [`ZoomRectModel`], which is the projection of the camera.
///
/// It is independent of a viewer: the viewer passes the size of its window, the mouse
/// movement and the time elapsed since the last frame, and applies the camera to its own
/// camera. While orbiting or zooming, the viewer keeps the mouse pointer in place, see
/// [`CameraControl::pointer_pinned`].
///
/// Its changes are collected as [`CameraEvent`]s, which are taken with
/// [`CameraControl::take_events`].
#[derive(Debug, Clone)]
pub struct OrthoCameraController {
    state: CameraControllerState,
    /// The zoom of the view: the detail of the overview, which is the projection.
    zoom_rect_model: ZoomRectModel,
}

impl OrthoCameraController {
    /// A controller for a window of `window_size` in pixels, which shows bounds of radius 100
    /// around the origin from the front.
    pub fn new(window_size: DVec2) -> Self {
        let mut controller = Self {
            state: CameraControllerState::new(window_size),
            zoom_rect_model: ZoomRectModel::new(),
        };
        controller
            .zoom_rect_model
            .initialize(controller.state.bounds.radius, window_size);
        controller.handle_zoom_events();

        controller.orbit(0.0, 0.0);
        controller
    }

    /// A controller with `state`, e.g. of another projection mode, which shows the area of
    /// `height` around `center` in the plane of the pivot, see [`CameraControl::visible_area`].
    ///
    /// The view direction stays, the view center is the center of the bounds.
    pub fn from_visible_area(state: CameraControllerState, center: Vec3, height: f64) -> Self {
        let window_size = state.window_size;
        let mut controller = Self {
            state,
            zoom_rect_model: ZoomRectModel::new(),
        };
        controller
            .zoom_rect_model
            .initialize(controller.state.bounds.radius, window_size);
        // The view direction stays, the camera looks at the center of the bounds.
        let bounds = controller.state.bounds;
        let camera = &mut controller.state.camera;
        let direction = camera.view_vector().normalize();
        camera.view_center = bounds.pos;
        camera.position = bounds.pos - direction * bounds.radius as f32 * 10.0;

        // The detail around the center in the coordinates of the camera.
        let p = controller.state.camera.transform_matrix().inverse() * center.extend(1.0);
        let size = DVec2::new(height * window_size.x / window_size.y, height);
        controller
            .zoom_rect_model
            .set_detail(Rect::from_top_left_size(
                (p / p.w).xy().as_dvec2() - size / 2.0,
                size,
            ));
        controller.handle_zoom_events();
        controller
    }

    /// The zoom of the view, which is the projection of the camera.
    pub fn zoom_model(&self) -> &ZoomRectModel {
        &self.zoom_rect_model
    }

    /// Move the view center back to the center of the bounds, while the view stays the same,
    /// i.e. the detail is moved by the offset of the view center instead.
    pub fn readjust_camera_position_and_detail(&mut self) {
        let camera = &self.state.camera;
        let to_camera = camera.transform_matrix().inverse();
        let get_point_2d = |v: Vec3| -> DVec2 {
            let p = to_camera * v.extend(1.0);
            (p / p.w).xy().as_dvec2()
        };

        let bounds_center = self.state.bounds.pos;
        let d = camera.view_center - bounds_center;
        let view_center_xy = get_point_2d(camera.view_center);
        let bounds_center_xy = get_point_2d(bounds_center);

        let camera = &mut self.state.camera;
        camera.view_center = bounds_center;
        camera.position -= d;
        self.zoom_rect_model
            .translate((view_center_xy - bounds_center_xy).as_vec2());
        self.handle_zoom_events();
    }

    /// Show the rectangle between two screen coordinates in pixels.
    pub fn zoom_to_rect(&mut self, screen_rect: Rect, factor: f64) {
        let rect = Rect::from_corners(
            self.zoom_point_from_screen_coordinates(screen_rect.top_left()),
            self.zoom_point_from_screen_coordinates(screen_rect.bottom_right()),
        );
        self.zoom_rect_model.zoom_to_rect(rect.normalized(), factor);
        self.handle_zoom_events();
    }

    /// Zoom by `delta` within `elapsed` with `speed`: a positive `y` zooms in.
    fn zoom_by(&mut self, delta: Vec2, elapsed: Duration, speed: f64) {
        let d = scaled_by_time(delta, elapsed);
        let delta = -d.y as f64 * speed;
        // Like `1 + delta / 100` for small deltas, but never 0 or negative, e.g. for a turn of
        // the wheel in a long frame, and zooming in and out by the same delta cancel out.
        let zoom_factor = (delta / 100.0).exp();

        self.zoom_rect_model
            .zoom_to_point(self.zoom_point(), zoom_factor);
        self.handle_zoom_events();
    }

    /// Apply the orthographic projection of the detail to the camera.
    fn update_camera_lens(&mut self) {
        self.state.camera.projection_matrix = self.zoom_rect_model.detail_ortho_projection_matrix();
    }

    /// React to the changes of the zoom, like the connected signals of Qt.
    fn handle_zoom_events(&mut self) {
        for event in self.zoom_rect_model.take_events() {
            match event {
                ZoomEvent::DetailChanged | ZoomEvent::OverviewChanged => self.update_camera_lens(),
                ZoomEvent::ZoomLevelChanged => {
                    self.state.events.push(CameraEvent::ZoomLevelChanged)
                }
            }
        }
    }

    /// The point in the coordinates of the camera under the screen coordinate in pixels.
    fn zoom_point_from_screen_coordinates(&self, screen_coords: DVec2) -> DVec2 {
        let p = self.state.camera.transform_matrix().inverse()
            * self.world_point(screen_coords).extend(1.0);
        (p / p.w).xy().as_dvec2()
    }

    /// Place the camera on the periphery of the bounding sphere along its view vector, so
    /// that the near and far plane of the projection enclose the bounds.
    fn place_camera_outside_of_bounds(&mut self) {
        let radius = self.state.bounds.radius as f32;
        let camera = &mut self.state.camera;
        camera.position = camera.view_center - camera.view_vector().normalize() * radius * 10.0;
    }
}

impl CameraControl for OrthoCameraController {
    fn state(&self) -> &CameraControllerState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut CameraControllerState {
        &mut self.state
    }

    /// The view keeps its height and becomes wider or narrower.
    fn set_window_size(&mut self, window_size: DVec2) {
        if !(window_size.x > 0.0 && window_size.y > 0.0) {
            return;
        }
        self.state.window_size = window_size;
        self.zoom_rect_model
            .fit_window_aspect_ratio(window_size.x / window_size.y);
        self.handle_zoom_events();
    }

    /// The overview is scaled with the radius of the bounds.
    fn set_bounds(&mut self, bounds: BoundingSphere) {
        let scale = bounds.radius / self.state.bounds.radius;
        self.state.bounds = bounds;

        let camera = &mut self.state.camera;
        let d = camera.view_center - bounds.pos;
        camera.view_center = bounds.pos;
        camera.position -= d;
        // The camera stays outside of the bounds, like after orbiting, e.g. of a smaller or
        // bigger model, so that the near and far plane of the projection enclose them.
        self.place_camera_outside_of_bounds();

        self.zoom_rect_model.scale_overview(scale);
        self.handle_zoom_events();
    }

    /// After orbiting, the view center is moved back to the center of the bounds.
    fn set_mode(&mut self, mode: Mode) {
        if self.state.mode == Mode::Orbit && mode == Mode::Inactive {
            self.readjust_camera_position_and_detail();
        }
        self.state.mode = mode;
        if mode == Mode::Zoom {
            self.update_camera_lens();
        }
    }

    fn orbit(&mut self, pan: f64, tilt: f64) {
        // Place camera on bounding sphere's periphery
        self.place_camera_outside_of_bounds();
        self.state.rotate_around_pivot(pan, tilt);
    }

    fn set_view(&mut self, view_vector: Vec3, up: Vec3) {
        let v = view_vector.normalize();
        let bounds = self.state.bounds;
        let camera = &mut self.state.camera;
        camera.view_center = bounds.pos;
        camera.up_vector = up;
        camera.position = bounds.pos - v * bounds.radius as f32 * 10.0;

        self.update_camera_lens();
    }

    fn zoom_level(&self) -> f64 {
        self.zoom_rect_model.zoom_level()
    }

    fn set_zoom_level(&mut self, zoom_level: f64) {
        self.zoom_rect_model
            .set_zoom_level_at(zoom_level, self.zoom_point());
        self.handle_zoom_events();
    }

    fn zoom_by_wheel(&mut self, wheel_delta: Vec2, elapsed: Duration) {
        self.zoom_by(wheel_delta, elapsed, self.state.config.wheel_speed);
    }

    fn zoom_by_mouse(&mut self, delta: Vec2, elapsed: Duration) {
        self.zoom_by(-delta, elapsed, self.state.config.zoom_speed);
    }

    fn zoom_at_center(&mut self, zoom_factor: f64) {
        let center = self.zoom_rect_model.detail().center();
        self.zoom_rect_model.zoom_to_point(center, zoom_factor);
        self.handle_zoom_events();
    }

    fn zoom_at_screen_point(&mut self, screen_coords: DVec2, zoom_factor: f64) {
        self.set_zoom_point_from_screen_coordinates(screen_coords);
        self.zoom_rect_model
            .zoom_to_point(self.zoom_point(), zoom_factor);
        self.handle_zoom_events();
    }

    fn show_all(&mut self) {
        let overview = self.zoom_rect_model.overview();
        self.zoom_rect_model.set_detail(overview);
        self.handle_zoom_events();
    }

    fn zoom_fit(&mut self) {
        self.zoom_rect_model.reset();
        self.handle_zoom_events();
    }

    fn set_zoom_point_from_screen_coordinates(&mut self, screen_coords: DVec2) {
        self.state.zoom_point = self.zoom_point_from_screen_coordinates(screen_coords);
    }

    /// The detail is moved by `delta` in pixels.
    fn translate(&mut self, delta: Vec2) {
        let detail = self.zoom_rect_model.detail();
        let window_size = self.state.window_size;
        let pixel = DVec2::new(detail.width / window_size.x, detail.height / window_size.y);
        self.zoom_rect_model
            .translate((delta.as_dvec2() * pixel).as_vec2());
        self.handle_zoom_events();
    }

    /// The ray is parallel to the view vector.
    fn camera_ray(&self, pos: DVec2) -> Ray {
        let camera = &self.state.camera;
        let window_size = self.state.window_size;
        // Normalized device coordinates
        let x = (2.0 * pos.x as f32) / window_size.x as f32 - 1.0;
        let y = 1.0 - (2.0 * pos.y as f32) / window_size.y as f32;
        let ndc = Vec4::new(x, y, -1.0, 1.0);

        // Eye coordinates
        let mut eye = camera.projection_matrix.inverse() * ndc;
        eye /= eye.w;

        // World coordinates
        let world = (camera.transform_matrix() * eye).xyz();

        Ray {
            origin: world,
            direction: camera.view_vector(),
        }
    }

    /// The center of the detail, moved along the view into the plane of the pivot.
    fn visible_area(&self) -> (Vec3, f64) {
        let detail = self.zoom_rect_model.detail();
        let camera = &self.state.camera;
        let center = detail.center().as_vec2();
        let point = camera
            .transform_matrix()
            .transform_point3(center.extend(0.0));
        let direction = camera.view_vector().normalize();
        let center = point + direction * (self.state.pivot - point).dot(direction);
        (center, detail.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Mat4;

    const WINDOW: DVec2 = DVec2::new(400.0, 200.0);

    #[test]
    fn new() {
        let mut controller = OrthoCameraController::new(WINDOW);
        let camera = controller.camera();
        // On the bounding sphere's periphery, looking at the pivot from the front.
        assert!(
            camera
                .position
                .abs_diff_eq(Vec3::new(0.0, 0.0, 1000.0), 1e-3)
        );
        assert_eq!(camera.view_center, Vec3::ZERO);
        assert_eq!(
            camera.projection_matrix,
            controller.zoom_model().detail_ortho_projection_matrix()
        );
        assert_eq!(controller.take_events(), [CameraEvent::ZoomLevelChanged]);
    }

    #[test]
    fn screen_and_world() {
        let controller = OrthoCameraController::new(WINDOW);
        let center = WINDOW / 2.0;
        assert!(controller.world_point(center).abs_diff_eq(Vec3::ZERO, 1e-3));
        // The overview is 400 by 200 for a radius of 100 in a 2:1 window.
        let corner = controller.world_point(DVec2::ZERO);
        assert!(corner.abs_diff_eq(Vec3::new(-200.0, 100.0, 0.0), 1e-3));
        let point = DVec2::new(123.0, 45.0);
        let back = controller.screen_coord(controller.world_point(point));
        assert!(back.abs_diff_eq(point, 1e-3));
    }

    #[test]
    fn orbit_keeps_pivot_distance() {
        let mut controller = OrthoCameraController::new(WINDOW);
        controller.set_pivot(Vec3::new(10.0, 0.0, 0.0));
        let distance = |c: &OrthoCameraController| (c.camera().position - c.pivot()).length();
        controller.orbit(0.0, 0.0);
        let before = distance(&controller);
        controller.orbit(30.0, 20.0);
        assert!((distance(&controller) - before).abs() < 1e-2);
        assert!(controller.camera().up_vector.is_normalized());
    }

    #[test]
    fn orbit_by_mouse_is_bounded() {
        let mut controller = OrthoCameraController::new(WINDOW);
        let before = controller.camera().view_vector().normalize();
        controller.orbit_by_mouse(Vec2::new(1e6, 0.0), Duration::from_secs(1));
        let after = controller.camera().view_vector().normalize();
        let angle = before.angle_between(after).to_degrees();
        assert!((angle - 15.0).abs() < 1e-2, "{angle}");
    }

    #[test]
    fn zoom_by_wheel() {
        let mut controller = OrthoCameraController::new(WINDOW);
        controller.set_zoom_point_from_screen_coordinates(WINDOW / 2.0);
        controller.start_zoom();
        // A positive delta zooms in, a negative one out by the same factor, with the wheel
        // speed 8.
        controller.zoom_by_wheel(Vec2::new(0.0, 2.0), Duration::from_secs(1));
        assert!((controller.zoom_level() - 0.16f64.exp()).abs() < 1e-6);
        controller.zoom_by_wheel(Vec2::new(0.0, -2.0), Duration::from_secs(1));
        assert!((controller.zoom_level() - 1.0).abs() < 1e-9);
        // A large delta in a long frame zooms in, but does not flip the view.
        controller.zoom_by_wheel(Vec2::new(0.0, 100.0), Duration::from_millis(170));
        assert!(controller.zoom_model().detail().width > 0.0);
        assert!(controller.zoom_level() > 1.0);
        assert_eq!(
            controller.camera().projection_matrix,
            controller.zoom_model().detail_ortho_projection_matrix()
        );
    }

    #[test]
    fn zoom_by_mouse() {
        let mut controller = OrthoCameraController::new(WINDOW);
        controller.set_zoom_point_from_screen_coordinates(WINDOW / 2.0);
        controller.start_zoom();
        // Dragging upwards zooms in with the zoom speed 25.
        controller.zoom_by_mouse(Vec2::new(0.0, -2.0), Duration::from_secs(1));
        assert!((controller.zoom_level() - 0.5f64.exp()).abs() < 1e-6);
    }

    #[test]
    fn camera_outside_of_bounds() {
        let mut controller = OrthoCameraController::new(WINDOW);
        // A small model, e.g. of 18mm, is between the near and the far plane.
        controller.set_bounds(BoundingSphere::new(Vec3::ZERO, 18.0));
        let distance = controller.camera().position.length();
        assert!((distance - 180.0).abs() < 1e-3, "{distance}");
    }

    #[test]
    fn pan_at_zoom_level_one() {
        let mut controller = OrthoCameraController::new(WINDOW);
        let point = DVec2::new(100.0, 50.0);
        let world = controller.world_point(point);
        // Panning is not limited by the overview, the point stays under the pointer.
        for _ in 0..10 {
            controller.translate_by_mouse(Vec2::new(-20.0, 10.0));
        }
        let moved = controller.screen_coord(world);
        assert!(
            moved.abs_diff_eq(point + DVec2::new(-200.0, 100.0), 1e-2),
            "{moved}"
        );
    }

    #[test]
    fn translate_follows_mouse() {
        let mut controller = OrthoCameraController::new(WINDOW);
        controller.set_zoom_level(2.0);
        let point = DVec2::new(100.0, 50.0);
        let world = controller.world_point(point);
        controller.start_translate();
        controller.translate_by_mouse(Vec2::new(10.0, 20.0));
        let moved = controller.screen_coord(world);
        assert!(
            moved.abs_diff_eq(point + DVec2::new(10.0, 20.0), 1e-3),
            "{moved}"
        );
    }

    #[test]
    fn readjust_keeps_view() {
        let mut controller = OrthoCameraController::new(WINDOW);
        controller.set_pivot(Vec3::new(50.0, 20.0, 0.0));
        controller.start_orbit();
        controller.orbit(20.0, 10.0);
        let point = Vec3::new(5.0, 6.0, 7.0);
        let before = controller.screen_coord(point);
        controller.set_inactive();
        assert_eq!(controller.camera().view_center, controller.bounds().pos);
        let after = controller.screen_coord(point);
        assert!(after.abs_diff_eq(before, 1e-2), "{before} {after}");
    }

    #[test]
    fn zoom_at_screen_point() {
        let mut controller = OrthoCameraController::new(WINDOW);
        let point = DVec2::new(100.0, 50.0);
        let world = controller.world_point(point);
        controller.zoom_at_screen_point(point, 0.5);
        assert!((controller.zoom_level() - 2.0).abs() < 1e-9);
        // The point under the screen coordinates stays in place.
        assert!(controller.screen_coord(world).abs_diff_eq(point, 1e-3));
    }

    #[test]
    fn show_all() {
        let mut controller = OrthoCameraController::new(WINDOW);
        controller.set_zoom_level(2.0);
        controller.translate(Vec2::new(50.0, 20.0));
        controller.zoom_fit();
        assert_eq!(controller.zoom_level(), 1.0);
        assert_ne!(
            controller.zoom_model().detail(),
            controller.zoom_model().overview()
        );
        controller.show_all();
        assert_eq!(
            controller.zoom_model().detail(),
            controller.zoom_model().overview()
        );
    }

    #[test]
    fn resize() {
        let mut controller = OrthoCameraController::new(WINDOW);
        controller.set_window_size(DVec2::new(200.0, 200.0));
        assert_eq!(controller.aspect_ratio(), 1.0);
        // The projection is square, i.e. the scene is not distorted.
        let corner = controller.world_point(DVec2::ZERO);
        assert!(corner.abs_diff_eq(Vec3::new(-100.0, 100.0, 0.0), 1e-3));
        assert_eq!(
            controller.camera().projection_matrix,
            controller.zoom_model().detail_ortho_projection_matrix()
        );
        // A hidden window is ignored.
        controller.set_window_size(DVec2::ZERO);
        assert_eq!(controller.window_size(), DVec2::new(200.0, 200.0));
    }

    #[test]
    fn set_view() {
        let mut controller = OrthoCameraController::new(WINDOW);
        controller.set_view(Vec3::new(0.0, -1.0, 0.0), Vec3::Z);
        let camera = controller.camera();
        assert!(
            camera
                .position
                .abs_diff_eq(Vec3::new(0.0, 1000.0, 0.0), 1e-3)
        );
    }

    #[test]
    fn plane_coord() {
        let controller = OrthoCameraController::new(WINDOW);
        let plane = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let coord = controller.plane_coord_from_screen_coord(WINDOW / 2.0, &plane);
        assert!(coord.abs_diff_eq(DVec2::new(-10.0, 0.0), 1e-3));
    }
}
