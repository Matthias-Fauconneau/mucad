// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The camera controller of a perspective view.

use std::time::Duration;

use glam::{DVec2, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};

use super::{CameraControl, CameraControllerState, CameraEvent, Mode, Ray, scaled_by_time};
use crate::BoundingSphere;

/// The smallest zoom level, like the one of the orthographic view.
const MIN_ZOOM_LEVEL: f64 = 0.25;

/// The largest zoom level, like the one of the orthographic view.
const MAX_ZOOM_LEVEL: f64 = 160.0;

/// The depth of the bounds in radii in front and behind their center, between the near and the
/// far plane, which also encloses the grid, which fades out at twice the radius.
const DEPTH_RADII: f32 = 3.0;

/// A camera controller for a perspective view, like in CAD applications.
///
/// The camera orbits around a pivot like in the orthographic view. Zooming moves the camera
/// along the ray to a target, e.g. the point under the pointer, which stays in place, the field
/// of view stays. At the zoom level 1, the bounding sphere fits into the view: the camera is at
/// the distance `radius / sin(fov / 2)` from the view center. Panning moves the camera parallel
/// to the view, so that the plane of the pivot follows the pointer.
#[derive(Debug, Clone)]
pub struct PerspectiveCameraController {
    state: CameraControllerState,
    /// The point in world coordinates, which stays in place while zooming.
    zoom_target: Vec3,
}

impl PerspectiveCameraController {
    /// A controller for a window of `window_size` in pixels, which shows bounds of radius 100
    /// around the origin from the front.
    pub fn new(window_size: DVec2) -> Self {
        let mut controller = Self::from_state(CameraControllerState::new(window_size));
        controller.show_all();
        controller
    }

    /// A controller with `state`, e.g. of another projection mode, which keeps its camera.
    pub fn from_state(state: CameraControllerState) -> Self {
        let zoom_target = state.camera.view_center();
        let mut controller = Self { state, zoom_target };
        controller.update_projection();
        controller
    }

    /// A controller with `state`, e.g. of another projection mode, which shows the area of
    /// `height` around `center` in the plane of the pivot, see [`CameraControl::visible_area`].
    pub fn from_visible_area(state: CameraControllerState, center: Vec3, height: f64) -> Self {
        let mut controller = Self::from_state(state);
        let distance = height as f32 / 2.0 / controller.half_fov_tan();
        controller.look_at(center, distance);
        controller
    }

    /// The vertical field of view in radians.
    fn fov(&self) -> f32 {
        (self.state.config.fov_y_degrees as f32).to_radians()
    }

    /// `tan(fov / 2)`, the half height of the view per distance.
    fn half_fov_tan(&self) -> f32 {
        (self.fov() / 2.0).tan()
    }

    /// The distance of the camera to the view center at the zoom level 1, where the bounding
    /// sphere fits into the view.
    fn fit_distance(&self) -> f32 {
        self.state.bounds.radius as f32 / (self.fov() / 2.0).sin()
    }

    /// The distance of the camera to the view center.
    fn distance(&self) -> f32 {
        self.state.camera.view_vector().length()
    }

    /// The unit vector of the view direction.
    fn direction(&self) -> Vec3 {
        self.state.camera.view_vector().normalize()
    }

    /// Look at `center` from `distance` along the view direction.
    fn look_at(&mut self, center: Vec3, distance: f32) {
        let direction = self.direction();
        let camera = &mut self.state.camera;
        camera.set_view_center(center);
        camera.set_position(center - direction * distance);
        self.update_projection();
    }

    /// Move the camera and the view center towards `target` by `zoom_factor`, a factor below
    /// 1 zooms in: their distances to the target are scaled, so that the target stays in place.
    ///
    /// The zoom level stays between its minimum and maximum.
    fn dolly(&mut self, target: Vec3, zoom_factor: f64) {
        let level = self.zoom_level();
        let new_level = (level / zoom_factor).clamp(MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL);
        let scale = (level / new_level) as f32;
        let camera = &mut self.state.camera;
        camera.set_position(target + (camera.position() - target) * scale);
        camera.set_view_center(target + (camera.view_center() - target) * scale);
        self.update_projection();
        if new_level != level {
            self.state.events.push(CameraEvent::ZoomLevelChanged);
        }
    }

    /// Zoom by `delta` within `elapsed` with `speed`: a positive `y` zooms in.
    fn zoom_by(&mut self, delta: Vec2, elapsed: Duration, speed: f64) {
        let d = scaled_by_time(delta, elapsed);
        let delta = -d.y as f64 * speed;
        // Like the orthographic view, zooming in and out by the same delta cancel out.
        let zoom_factor = (delta / 100.0).exp();
        self.dolly(self.zoom_target, zoom_factor);
    }

    /// The near and the far plane, which enclose the bounds, also when the camera is in them.
    fn near_far(&self) -> (f32, f32) {
        let camera = &self.state.camera;
        let radius = self.state.bounds.radius as f32;
        let depth = (self.state.bounds.pos - camera.position()).dot(self.direction());
        let near = (depth - DEPTH_RADII * radius).max(radius * 1e-3);
        let far = (depth + DEPTH_RADII * radius).max(near * 2.0);
        (near, far)
    }

    /// Apply the perspective projection to the camera.
    fn update_projection(&mut self) {
        let aspect = (self.state.window_size.x / self.state.window_size.y) as f32;
        let (near, far) = self.near_far();
        let projection = Mat4::perspective_rh_gl(self.fov(), aspect, near, far);
        self.state.camera.set_projection_matrix(projection);
    }

    /// The distance of the plane of the pivot from the camera along the view direction.
    fn pivot_depth(&self) -> f32 {
        (self.state.pivot - self.state.camera.position()).dot(self.direction())
    }
}

impl CameraControl for PerspectiveCameraController {
    fn state(&self) -> &CameraControllerState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut CameraControllerState {
        &mut self.state
    }

    /// The field of view stays vertical, the view becomes wider or narrower.
    fn set_window_size(&mut self, window_size: DVec2) {
        if !(window_size.x > 0.0 && window_size.y > 0.0) {
            return;
        }
        self.state.window_size = window_size;
        self.update_projection();
    }

    /// The camera keeps its distance, i.e. the zoom level changes with the radius.
    fn set_bounds(&mut self, bounds: BoundingSphere) {
        let offset = bounds.pos - self.state.bounds.pos;
        self.state.bounds = bounds;
        let camera = &mut self.state.camera;
        camera.set_view_center(camera.view_center() + offset);
        camera.set_position(camera.position() + offset);
        self.update_projection();
    }

    fn set_mode(&mut self, mode: Mode) {
        self.state.mode = mode;
    }

    /// The camera keeps its distance to the pivot.
    fn orbit(&mut self, pan: f64, tilt: f64) {
        self.state.rotate_around_pivot(pan, tilt);
        self.update_projection();
    }

    /// The camera keeps its distance.
    fn set_view(&mut self, view_vector: Vec3, up: Vec3) {
        let distance = self.distance();
        let center = self.state.bounds.pos;
        let camera = &mut self.state.camera;
        camera.set_view_center(center);
        camera.set_up_vector(up);
        camera.set_position(center - view_vector.normalize() * distance);
        self.update_projection();
    }

    /// The distance at the zoom level 1 per the distance of the camera to the view center.
    fn zoom_level(&self) -> f64 {
        (self.fit_distance() / self.distance()) as f64
    }

    fn set_zoom_level(&mut self, zoom_level: f64) {
        self.dolly(self.zoom_target, self.zoom_level() / zoom_level);
    }

    fn zoom_by_wheel(&mut self, wheel_delta: Vec2, elapsed: Duration) {
        self.zoom_by(wheel_delta, elapsed, self.state.config.wheel_speed);
    }

    fn zoom_by_mouse(&mut self, delta: Vec2, elapsed: Duration) {
        self.zoom_by(-delta, elapsed, self.state.config.zoom_speed);
    }

    fn zoom_at_center(&mut self, zoom_factor: f64) {
        self.dolly(self.state.camera.view_center(), zoom_factor);
    }

    fn zoom_at_screen_point(&mut self, screen_coords: DVec2, zoom_factor: f64) {
        self.set_zoom_point_from_screen_coordinates(screen_coords);
        self.dolly(self.zoom_target, zoom_factor);
    }

    fn show_all(&mut self) {
        let level = self.zoom_level();
        self.look_at(self.state.bounds.pos, self.fit_distance());
        if level != 1.0 {
            self.state.events.push(CameraEvent::ZoomLevelChanged);
        }
    }

    fn zoom_fit(&mut self) {
        let level = self.zoom_level();
        self.look_at(self.state.camera.view_center(), self.fit_distance());
        if level != 1.0 {
            self.state.events.push(CameraEvent::ZoomLevelChanged);
        }
    }

    /// The zoom target is the point under the screen coordinates in the plane of the pivot.
    fn set_zoom_point_from_screen_coordinates(&mut self, screen_coords: DVec2) {
        self.zoom_target = self.world_point(screen_coords);
        let p = self.state.camera.transform_matrix().inverse() * self.zoom_target.extend(1.0);
        self.state.zoom_point = (p / p.w).xy().as_dvec2();
    }

    /// The camera and the view center move by `delta` in pixels of the plane of the pivot.
    fn translate(&mut self, delta: Vec2) {
        let pixel =
            2.0 * self.pivot_depth() * self.half_fov_tan() / self.state.window_size.y as f32;
        let transform = self.state.camera.transform_matrix();
        let offset = transform.transform_vector3(Vec3::new(delta.x, delta.y, 0.0) * pixel);
        let camera = &mut self.state.camera;
        camera.set_position(camera.position() + offset);
        camera.set_view_center(camera.view_center() + offset);
        self.update_projection();
    }

    /// The ray starts on the near plane and goes away from the camera.
    fn camera_ray(&self, pos: DVec2) -> Ray {
        let camera = &self.state.camera;
        let window_size = self.state.window_size;
        let x = (2.0 * pos.x as f32) / window_size.x as f32 - 1.0;
        let y = 1.0 - (2.0 * pos.y as f32) / window_size.y as f32;
        let world_from_clip = camera.transform_matrix() * camera.projection_matrix().inverse();
        let unproject = |z: f32| {
            let p = world_from_clip * Vec4::new(x, y, z, 1.0);
            (p / p.w).xyz()
        };
        let near = unproject(-1.0);
        Ray {
            origin: near,
            direction: unproject(1.0) - near,
        }
    }

    fn visible_area(&self) -> (Vec3, f64) {
        let depth = self.pivot_depth();
        let center = self.state.camera.position() + self.direction() * depth;
        (center, (2.0 * depth * self.half_fov_tan()) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: DVec2 = DVec2::new(400.0, 200.0);

    fn controller() -> PerspectiveCameraController {
        PerspectiveCameraController::new(WINDOW)
    }

    #[test]
    fn fits_at_zoom_level_one() {
        let controller = controller();
        assert!((controller.zoom_level() - 1.0).abs() < 1e-6);
        // The bounding sphere touches the top and the bottom of the view.
        let distance = 100.0 / (15f32.to_radians()).sin();
        assert!((controller.camera().position().length() - distance).abs() < 1e-2);
        let top = controller.screen_coord(Vec3::new(0.0, 100.0 / 15f32.to_radians().cos(), 0.0));
        assert!(top.y.abs() < 1.0, "{top}");
    }

    #[test]
    fn zoom_keeps_the_point_under_the_pointer() {
        let mut controller = controller();
        let pointer = DVec2::new(300.0, 60.0);
        let world = controller.world_point(pointer);
        controller.zoom_at_screen_point(pointer, 0.5);
        assert!((controller.zoom_level() - 2.0).abs() < 1e-4);
        assert!(controller.screen_coord(world).abs_diff_eq(pointer, 1e-2));
        // The view direction stays.
        assert!(
            controller
                .camera()
                .view_vector()
                .normalize()
                .abs_diff_eq(Vec3::NEG_Z, 1e-5)
        );
    }

    #[test]
    fn zoom_is_limited() {
        let mut controller = controller();
        controller.zoom_at_center(1e-6);
        assert!((controller.zoom_level() - MAX_ZOOM_LEVEL).abs() < 1e-2);
        controller.zoom_at_center(1e6);
        assert!((controller.zoom_level() - MIN_ZOOM_LEVEL).abs() < 1e-4);
    }

    #[test]
    fn wheel_zooms_in_and_out() {
        let mut controller = controller();
        controller.zoom_by_wheel(Vec2::new(0.0, 2.0), Duration::from_secs(1));
        assert!(controller.zoom_level() > 1.0);
        controller.zoom_by_wheel(Vec2::new(0.0, -2.0), Duration::from_secs(1));
        assert!((controller.zoom_level() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn pan_follows_the_pointer() {
        let mut controller = controller();
        let point = DVec2::new(100.0, 50.0);
        let world = controller.world_point(point);
        controller.translate_by_mouse(Vec2::new(30.0, -20.0));
        let moved = controller.screen_coord(world);
        assert!(
            moved.abs_diff_eq(point + DVec2::new(30.0, -20.0), 1e-2),
            "{moved}"
        );
    }

    #[test]
    fn orbit_keeps_the_distance_to_the_pivot() {
        let mut controller = controller();
        controller.set_pivot(Vec3::new(10.0, 20.0, 0.0));
        let distance = (controller.camera().position() - controller.pivot()).length();
        controller.orbit(40.0, 25.0);
        let orbited = (controller.camera().position() - controller.pivot()).length();
        assert!((orbited - distance).abs() < 1e-2);
    }

    #[test]
    fn world_and_screen() {
        let controller = controller();
        let point = DVec2::new(123.0, 45.0);
        let back = controller.screen_coord(controller.world_point(point));
        assert!(back.abs_diff_eq(point, 1e-2), "{back}");
    }

    #[test]
    fn visible_area() {
        let controller = controller();
        let (center, height) = controller.visible_area();
        assert!(center.abs_diff_eq(Vec3::ZERO, 1e-3));
        // At the zoom level 1 the sphere fits, the view is higher than its diameter.
        assert!(height > 200.0);
        let rebuilt = PerspectiveCameraController::from_visible_area(
            controller.state().clone(),
            center,
            height,
        );
        assert!((rebuilt.zoom_level() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn near_and_far_enclose_the_bounds() {
        let mut controller = controller();
        controller.zoom_at_center(1.0 / 100.0);
        let (near, far) = controller.near_far();
        let depth = controller.camera().position().length();
        assert!(
            near > 0.0 && near < depth && far > depth + 100.0,
            "{near} {far}"
        );
    }
}
