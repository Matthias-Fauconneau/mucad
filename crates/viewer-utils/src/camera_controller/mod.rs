// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Camera controllers to orbit around a pivot, translate and zoom a view: the
//! [`CameraController`] of a viewer with its projection modes, which share [`CameraControl`].

mod ortho;
mod perspective;

pub use ortho::OrthoCameraController;
pub use perspective::PerspectiveCameraController;

use std::time::Duration;

use glam::{DVec2, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use serde::{Deserialize, Serialize};

use crate::{BoundingSphere, Camera, ProjectionType};

/// The projection mode of a [`CameraController`], written in lower case, e.g. `perspective`.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::IntoStaticStr,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ProjectionMode {
    /// An orthographic projection, e.g. to measure.
    #[default]
    Orthographic,
    /// A perspective projection, e.g. to look at a model.
    Perspective,
}

/// The projection and the speeds of a camera controller, e.g. from TOML.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CameraControllerConfig {
    /// The projection mode at the start.
    pub projection: ProjectionMode,
    /// The vertical field of view of the perspective projection in degrees.
    pub fov_y_degrees: f64,
    /// The speed of orbiting with the mouse.
    pub orbit_speed: f64,
    /// The speed of zooming by dragging the mouse, a negative speed inverts zooming.
    pub zoom_speed: f64,
    /// The speed of zooming with the wheel, a negative speed inverts zooming.
    pub wheel_speed: f64,
    /// The speed of translating with the mouse, which is not used yet: translating follows
    /// the mouse.
    pub translate_speed: f64,
}

impl Default for CameraControllerConfig {
    fn default() -> Self {
        Self {
            projection: ProjectionMode::Orthographic,
            fov_y_degrees: 30.0,
            orbit_speed: 25.0,
            zoom_speed: 25.0,
            wheel_speed: 8.0,
            translate_speed: 15.0,
        }
    }
}

/// A ray from a point on the near plane of a camera into the view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    /// The point on the near plane.
    pub origin: Vec3,
    /// The direction, e.g. the view vector of an orthographic camera, which is not normalized.
    pub direction: Vec3,
}

/// What the mouse moves at the moment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Mode {
    /// Nothing, the mouse is free.
    #[default]
    Inactive,
    /// The camera around the pivot.
    Orbit,
    /// The view.
    Translation,
    /// The zoom level.
    Zoom,
}

/// A change of a camera controller, which a viewer reacts to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraEvent {
    /// The zoom level is changed.
    ZoomLevelChanged,
    /// The pivot is changed.
    PointOfInterestChanged,
}

/// The state, which the projection modes of a camera controller share.
#[derive(Debug, Clone)]
pub struct CameraControllerState {
    /// The camera, which the viewer applies to its own camera.
    pub(crate) camera: Camera,
    /// The bounds of what the camera shows: the overview and the distance of the camera.
    pub(crate) bounds: BoundingSphere,
    /// The point, which the camera orbits around.
    pub(crate) pivot: Vec3,
    /// The size of the window in pixels.
    pub(crate) window_size: DVec2,
    /// The speeds with the mouse.
    pub(crate) config: CameraControllerConfig,
    /// What the mouse moves at the moment.
    pub(crate) mode: Mode,
    /// The point in the coordinates of the camera, which stays in place while zooming.
    pub(crate) zoom_point: DVec2,
    /// The changes since they were taken.
    pub(crate) events: Vec<CameraEvent>,
}

impl CameraControllerState {
    /// The state for a window of `window_size` in pixels with bounds of radius 100 around the
    /// origin, the camera looks at it from the front.
    pub(crate) fn new(window_size: DVec2) -> Self {
        let bounds = BoundingSphere::new(Vec3::ZERO, 100.0);
        let mut camera = Camera::default();
        camera.up_vector = Vec3::Y;
        camera.view_center = bounds.pos;
        camera.position = bounds.pos + Vec3::Z * 2.0 * bounds.radius as f32;
        Self {
            camera,
            bounds,
            pivot: Vec3::ZERO,
            window_size,
            config: CameraControllerConfig::default(),
            mode: Mode::Inactive,
            zoom_point: DVec2::ZERO,
            events: Vec::new(),
        }
    }

    /// Rotate the camera around the pivot by `pan` and `tilt` in degrees, its distance to the
    /// pivot stays.
    pub(crate) fn rotate_around_pivot(&mut self, pan: f64, tilt: f64) {
        let camera = &mut self.camera;
        let p = self.pivot;
        let v = p - camera.position;
        let w = p - camera.view_center;

        // Do pan rotation first
        let pq = camera.pan_rotation(-pan as f32);

        // Apply tilt rotation and set camera position and view angle
        let tq = camera.tilt_rotation(tilt as f32);
        camera.position = p - pq * tq * v;
        camera.view_center = p - pq * tq * w;

        let up = pq * tq * camera.up_vector;
        camera.up_vector = up.normalize();
    }
}

/// The control of a camera with the mouse and the keys, which the projection modes and the
/// [`CameraController`] share.
///
/// A projection mode implements the required methods, the provided ones work with them and
/// with its [`CameraControllerState`]. The provided methods are not overridden, so that the
/// [`CameraController`] only delegates the required ones.
pub trait CameraControl {
    /// The state, which the projection modes share.
    fn state(&self) -> &CameraControllerState;

    /// The state, which the projection modes share.
    fn state_mut(&mut self) -> &mut CameraControllerState;

    /// Set the size of the window in pixels, e.g. after resizing it.
    ///
    /// An empty size is ignored, e.g. of a hidden window.
    fn set_window_size(&mut self, window_size: DVec2);

    /// Set the bounds of what the camera shows, e.g. of the model: the camera looks at its
    /// position and is moved with it.
    fn set_bounds(&mut self, bounds: BoundingSphere);

    /// Set what the mouse moves at the moment, e.g. to start orbiting.
    fn set_mode(&mut self, mode: Mode);

    /// Orbit the camera around the pivot by `pan` and `tilt` in degrees.
    fn orbit(&mut self, pan: f64, tilt: f64);

    /// Look at the center of the bounds along `view_vector` with `up`.
    fn set_view(&mut self, view_vector: Vec3, up: Vec3);

    /// The zoom level, 1 shows the whole scene.
    fn zoom_level(&self) -> f64;

    /// Set the zoom level around the zoom point.
    fn set_zoom_level(&mut self, zoom_level: f64);

    /// Zoom at the zoom point by the vertical delta of the mouse wheel in pixels within the
    /// time `elapsed`, since the last frame: a positive `y` zooms in, a negative one out, with
    /// a positive wheel speed.
    fn zoom_by_wheel(&mut self, wheel_delta: Vec2, elapsed: Duration);

    /// Zoom at the zoom point by dragging the mouse by `delta` in pixels within the time
    /// `elapsed`, since the last frame or the start of zooming: dragging upwards, i.e. a
    /// negative `y`, zooms in, with a positive zoom speed.
    fn zoom_by_mouse(&mut self, delta: Vec2, elapsed: Duration);

    /// Zoom by `zoom_factor` around the center of the view, e.g. by keys: a factor below 1
    /// zooms in.
    fn zoom_at_center(&mut self, zoom_factor: f64);

    /// Zoom by `zoom_factor` around the point under the screen coordinates in pixels, which
    /// stays in place, e.g. for pinching: a factor below 1 zooms in.
    fn zoom_at_screen_point(&mut self, screen_coords: DVec2, zoom_factor: f64);

    /// Show the whole scene in the middle of the view, unlike [`CameraControl::zoom_fit`],
    /// which keeps the center of the view, e.g. after panning.
    fn show_all(&mut self);

    /// Show the whole scene with the zoom level 1 around the center of the view.
    fn zoom_fit(&mut self);

    /// Set the zoom point to the point under the screen coordinates in pixels.
    fn set_zoom_point_from_screen_coordinates(&mut self, screen_coords: DVec2);

    /// Translate the view by `delta` in pixels, positive values move the view to the right
    /// and upwards, i.e. the scene to the left and downwards.
    fn translate(&mut self, delta: Vec2);

    /// The ray through the screen coordinate in pixels.
    fn camera_ray(&self, pos: DVec2) -> Ray;

    /// The center and the height of the visible area in the plane of the pivot parallel to the
    /// view, which stays when the projection mode is changed.
    fn visible_area(&self) -> (Vec3, f64);

    /// The changes since the last call.
    fn take_events(&mut self) -> Vec<CameraEvent> {
        std::mem::take(&mut self.state_mut().events)
    }

    /// The camera, which the viewer applies to its own camera.
    fn camera(&self) -> &Camera {
        &self.state().camera
    }

    /// The size of the window in pixels.
    fn window_size(&self) -> DVec2 {
        self.state().window_size
    }

    /// The ratio of the width to the height of the window.
    fn aspect_ratio(&self) -> f64 {
        let size = self.window_size();
        size.x / size.y
    }

    /// The bounds of what the camera shows.
    fn bounds(&self) -> BoundingSphere {
        self.state().bounds
    }

    /// The speeds with the mouse.
    fn config(&self) -> &CameraControllerConfig {
        &self.state().config
    }

    /// Set the speeds with the mouse.
    fn set_config(&mut self, config: CameraControllerConfig) {
        self.state_mut().config = config;
    }

    /// The point, which the camera orbits around.
    fn pivot(&self) -> Vec3 {
        self.state().pivot
    }

    /// Set the point, which the camera orbits around.
    fn set_pivot(&mut self, v: Vec3) {
        let state = self.state_mut();
        if v != state.pivot {
            state.pivot = v;
            state.events.push(CameraEvent::PointOfInterestChanged);
        }
    }

    /// Set the pivot to the point under the screen coordinates in pixels, in the plane of the
    /// pivot parallel to the view.
    fn set_pivot_from_screen_coordinates(&mut self, screen_coords: DVec2) {
        let pivot = self.world_point(screen_coords);
        self.set_pivot(pivot);
    }

    /// Sets the pivot to the center of the bounds.
    fn reset_pivot(&mut self) {
        let center = self.bounds().pos;
        self.set_pivot(center);
    }

    /// The kind of the projection of the camera.
    fn projection_type(&self) -> ProjectionType {
        self.camera().projection_type()
    }

    /// What the mouse moves at the moment.
    fn mode(&self) -> Mode {
        self.state().mode
    }

    /// Start orbiting with the mouse.
    fn start_orbit(&mut self) {
        self.set_mode(Mode::Orbit);
    }

    /// Start translating with the mouse.
    fn start_translate(&mut self) {
        self.set_mode(Mode::Translation);
    }

    /// Start zooming with the mouse.
    fn start_zoom(&mut self) {
        self.set_mode(Mode::Zoom);
    }

    /// Stop orbiting, translating or zooming with the mouse.
    fn set_inactive(&mut self) {
        self.set_mode(Mode::Inactive);
    }

    /// Whether the mouse moves something, then the viewer wraps the pointer around the
    /// edges of the window, so that the mouse moves on.
    fn pointer_wrapped(&self) -> bool {
        self.mode() != Mode::Inactive
    }

    /// Whether the viewer keeps the pointer in place while orbiting or zooming, then the
    /// mouse movement is passed as delta.
    fn pointer_pinned(&self) -> bool {
        !matches!(self.mode(), Mode::Inactive | Mode::Translation)
    }

    /// Orbit by the movement of the mouse in pixels within the time `elapsed`, since the last
    /// frame or the start of orbiting.
    ///
    /// The angles are bounded to 15° per call.
    fn orbit_by_mouse(&mut self, mouse_delta: Vec2, elapsed: Duration) {
        let wr = self.window_size();
        let d = scaled_by_time(mouse_delta, elapsed);
        let delta = d.as_dvec2() * self.config().orbit_speed;
        let max_angle = 15.0;

        let pan = (delta.x / wr.x * 360.0).clamp(-max_angle, max_angle);
        let tilt = (-delta.y / wr.y * 360.0).clamp(-max_angle, max_angle);

        self.orbit(pan, tilt);
    }

    /// Translate by the movement of the mouse in pixels, the scene follows the mouse.
    fn translate_by_mouse(&mut self, mouse_delta: Vec2) {
        let d = mouse_delta;
        self.translate(Vec2::new(-d.x, d.y));
    }

    /// The point in the coordinates of the camera, which stays in place while zooming.
    fn zoom_point(&self) -> DVec2 {
        self.state().zoom_point
    }

    /// The point under the screen coordinate in pixels, in the plane of the pivot parallel
    /// to the view.
    fn world_point(&self, screen_coord: DVec2) -> Vec3 {
        let ray = self.camera_ray(screen_coord);

        // The virtual scene plane is defined by the camera viewVector and the
        // pivot
        let view_vector = self.camera().view_vector();
        let dot = ray.direction.dot(view_vector);
        if dot.abs() <= 0.00001 {
            // The ray is parallel to the virtual plane
            return self.bounds().pos;
        }

        let t = (self.pivot() - ray.origin).dot(view_vector) / dot;

        // The intersection point is behind the origin of the ray
        // debug_assert!(t >= 0.0);
        ray.origin + ray.direction * t
    }

    /// The screen coordinate in pixels of a point in world coordinates.
    fn screen_coord(&self, world_point: Vec3) -> DVec2 {
        let camera = self.camera();
        let point_in_screen_space = camera.projection_matrix()
            * camera.transform_matrix().inverse()
            * world_point.extend(1.0);
        let ndc = point_in_screen_space.xy().as_dvec2() / point_in_screen_space.w as f64;
        let size = self.window_size();

        DVec2::new((ndc.x + 1.0) / 2.0 * size.x, (1.0 - ndc.y) / 2.0 * size.y)
    }

    /// The point in the coordinates of the plane with `plane_transform`, which is under the
    /// screen coordinate in pixels.
    fn plane_coord_from_screen_coord(&self, screen_coord: DVec2, plane_transform: &Mat4) -> DVec2 {
        // Intersection of a ray and a plane
        // http://lousodrome.net/blog/light/2020/07/03/intersection-of-a-ray-and-a-plane/

        // ray in world coordinates
        let ray = self.camera_ray(screen_coord);

        // normal of plane in world coordinates
        let mut rotation = *plane_transform;
        rotation.w_axis = Vec4::new(0.0, 0.0, 0.0, 1.0);
        let plane_normal_wcs = rotation.project_point3(Vec3::Z).normalize();

        // point on plane in world coordinates
        let plane_point_wcs = plane_transform.project_point3(Vec3::ZERO);

        // calculate intersection in world coordinates
        let diff_wcs = plane_point_wcs - ray.origin;
        let dot1 = diff_wcs.dot(plane_normal_wcs);
        let dot2 = ray.direction.dot(plane_normal_wcs);
        let t = dot1 / dot2;
        let intersection_wcs = ray.origin + ray.direction * t;

        // transform intersection to plane coordinates
        let intersection_pcs = plane_transform.inverse().project_point3(intersection_wcs);

        DVec2::new(intersection_pcs.x as f64, intersection_pcs.y as f64)
    }
}

/// The camera controller of a viewer in one of its projection modes, orthographic by default.
#[derive(Debug, Clone)]
pub enum CameraController {
    /// An orthographic projection.
    Ortho(OrthoCameraController),
    /// A perspective projection.
    Perspective(PerspectiveCameraController),
}

impl CameraController {
    /// An orthographic controller for a window of `window_size` in pixels, which shows bounds
    /// of radius 100 around the origin from the front.
    pub fn new(window_size: DVec2) -> Self {
        Self::Ortho(OrthoCameraController::new(window_size))
    }

    /// The projection mode.
    pub fn projection_mode(&self) -> ProjectionMode {
        match self {
            Self::Ortho(_) => ProjectionMode::Orthographic,
            Self::Perspective(_) => ProjectionMode::Perspective,
        }
    }

    /// Change the projection mode, the visible area in the plane of the pivot stays, see
    /// [`CameraControl::visible_area`].
    pub fn set_projection_mode(&mut self, mode: ProjectionMode) {
        if mode == self.projection_mode() {
            return;
        }
        let (center, height) = self.visible_area();
        let state = self.state().clone();
        *self = match mode {
            ProjectionMode::Orthographic => Self::Ortho(OrthoCameraController::from_visible_area(
                state, center, height,
            )),
            ProjectionMode::Perspective => Self::Perspective(
                PerspectiveCameraController::from_visible_area(state, center, height),
            ),
        };
    }

    /// Change between the orthographic and the perspective projection.
    pub fn toggle_projection(&mut self) {
        self.set_projection_mode(match self.projection_mode() {
            ProjectionMode::Orthographic => ProjectionMode::Perspective,
            ProjectionMode::Perspective => ProjectionMode::Orthographic,
        });
    }

    /// The orthographic controller, if it is the projection mode.
    pub fn ortho(&self) -> Option<&OrthoCameraController> {
        match self {
            Self::Ortho(controller) => Some(controller),
            Self::Perspective(_) => None,
        }
    }

    /// The orthographic controller, if it is the projection mode.
    pub fn ortho_mut(&mut self) -> Option<&mut OrthoCameraController> {
        match self {
            Self::Ortho(controller) => Some(controller),
            Self::Perspective(_) => None,
        }
    }

    /// The perspective controller, if it is the projection mode.
    pub fn perspective(&self) -> Option<&PerspectiveCameraController> {
        match self {
            Self::Perspective(controller) => Some(controller),
            Self::Ortho(_) => None,
        }
    }
}

/// The controller of the projection mode, e.g. `controller.orbit(…)` of [`CameraControl`].
impl std::ops::Deref for CameraController {
    type Target = dyn CameraControl;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Ortho(controller) => controller,
            Self::Perspective(controller) => controller,
        }
    }
}

impl std::ops::DerefMut for CameraController {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Ortho(controller) => controller,
            Self::Perspective(controller) => controller,
        }
    }
}

/// The movement `d` of the mouse or its wheel multiplied with the elapsed time in seconds,
/// so that orbiting and zooming have the same speed at any frame rate.
pub(crate) fn scaled_by_time(d: Vec2, elapsed: Duration) -> Vec2 {
    d * elapsed.as_secs_f32()
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: DVec2 = DVec2::new(400.0, 200.0);

    /// Points in the plane of the pivot stay at their screen coordinates.
    fn assert_same_view(before: &CameraController, after: &CameraController) {
        for point in [
            DVec2::new(200.0, 100.0),
            DVec2::new(30.0, 40.0),
            DVec2::new(350.0, 170.0),
        ] {
            let world = before.world_point(point);
            let moved = after.screen_coord(world);
            assert!(moved.abs_diff_eq(point, 0.5), "{point} -> {moved}");
        }
    }

    #[test]
    fn toggle_keeps_the_visible_area() {
        let mut controller = CameraController::new(WINDOW);
        controller.set_view(Vec3::new(-1.0, 1.0, -1.0), Vec3::Z);
        controller.set_pivot(Vec3::new(20.0, -10.0, 5.0));
        controller.zoom_at_screen_point(DVec2::new(120.0, 80.0), 0.4);
        controller.translate(Vec2::new(25.0, -15.0));

        let ortho = controller.clone();
        controller.toggle_projection();
        assert_eq!(controller.projection_mode(), ProjectionMode::Perspective);
        assert!(controller.perspective().is_some());
        assert_same_view(&ortho, &controller);
        assert!((controller.visible_area().1 - ortho.visible_area().1).abs() < 1e-3);

        let perspective = controller.clone();
        controller.toggle_projection();
        assert_eq!(controller.projection_mode(), ProjectionMode::Orthographic);
        assert_same_view(&perspective, &controller);
        // The same zoom level within the precision of `f32` world coordinates.
        assert!((controller.zoom_level() - ortho.zoom_level()).abs() < 1e-4);
    }

    #[test]
    fn same_mode_is_kept() {
        let mut controller = CameraController::new(WINDOW);
        controller.zoom_at_center(0.5);
        controller.set_projection_mode(ProjectionMode::Orthographic);
        assert!((controller.zoom_level() - 2.0).abs() < 1e-9);
    }
}
