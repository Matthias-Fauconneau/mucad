// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A camera controller to orbit around a pivot, translate and zoom an orthographic view.

use std::time::Duration;

use glam::{DVec2, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::{Camera, ProjectionType, Rect, ZoomEvent, ZoomRectModel, ZoomState};

/// A ray from a point on the near plane of a camera into the view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    /// The point on the near plane.
    pub origin: Vec3,
    /// The direction, the view vector of the camera, which is not normalized.
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

/// A change of a [`CameraController`], which a viewer reacts to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraEvent {
    /// The zoom level is changed.
    ZoomLevelChanged,
    /// The pivot is changed.
    PointOfInterestChanged,
}

/// The state of a [`CameraController`], e.g. to restore a view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraState {
    /// The view center of the camera.
    pub view_center: Vec3,
    /// The position of the camera.
    pub position: Vec3,
    /// The up vector of the camera.
    pub up: Vec3,
    /// The zoom.
    pub zoom: ZoomState,
}

/// A camera controller for an orthographic view.
///
/// The camera orbits around a pivot. Translating and zooming change the detail of the
/// [`ZoomRectModel`], which is the projection of the camera.
///
/// It is independent of a viewer: the viewer passes the size of its window, the mouse
/// movement and the time elapsed since the last frame, and applies the [`Camera`] to its own
/// camera. While orbiting or zooming, the viewer keeps the mouse pointer in place, see
/// [`CameraController::pointer_pinned`].
///
/// Its changes are collected as [`CameraEvent`]s, which are taken with
/// [`CameraController::take_events`].
#[derive(Debug, Clone)]
pub struct CameraController {
    camera: Camera,
    window_size: DVec2,

    orbit_speed: f64,
    /// Negative zoom speed means zoom is inverted.
    zoom_speed: f64,
    translate_speed: f64,
    mode: Mode,

    scene_center: Vec3,
    scene_radius: f64,

    pivot: Vec3,

    zoom_rect_model: ZoomRectModel,
    zoom_point: DVec2,

    events: Vec<CameraEvent>,
}

impl CameraController {
    /// A controller for a window of `window_size` in pixels, which shows a scene of radius 100
    /// around the origin from the front.
    pub fn new(window_size: DVec2) -> Self {
        let mut controller = Self {
            camera: Camera::default(),
            window_size,
            orbit_speed: 25.0,
            zoom_speed: 25.0,
            translate_speed: 15.0,
            mode: Mode::Inactive,
            scene_center: Vec3::ZERO,
            scene_radius: 100.0,
            pivot: Vec3::ZERO,
            zoom_rect_model: ZoomRectModel::new(),
            zoom_point: DVec2::ZERO,
            events: Vec::new(),
        };

        let pivot = controller.pivot();
        let radius = controller.scene_radius as f32;
        let camera = &mut controller.camera;
        camera.set_up_vector(Vec3::Y);
        camera.set_view_center(pivot);
        camera.set_position(pivot + Vec3::Z.normalize() * 2.0 * radius);

        controller
            .zoom_rect_model
            .initialize(controller.scene_radius, window_size);
        controller.handle_zoom_events();

        controller.orbit(0.0, 0.0);
        controller
    }

    /// The changes since the last call.
    pub fn take_events(&mut self) -> Vec<CameraEvent> {
        std::mem::take(&mut self.events)
    }

    /// The camera, which the viewer applies to its own camera.
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// The size of the window in pixels.
    pub fn window_size(&self) -> DVec2 {
        self.window_size
    }

    /// Set the size of the window in pixels, e.g. after resizing it.
    pub fn set_window_size(&mut self, window_size: DVec2) {
        self.window_size = window_size;
    }

    /// The radius of the scene around its center.
    pub fn scene_radius(&self) -> f64 {
        self.scene_radius
    }

    /// Set the radius of the scene, the overview is scaled with it.
    pub fn set_scene_radius(&mut self, radius: f32) {
        let scale = radius as f64 / self.scene_radius;
        self.scene_radius = radius as f64;
        self.zoom_rect_model.scale_overview(scale);
        self.handle_zoom_events();
    }

    /// The center of the scene.
    pub fn scene_center(&self) -> Vec3 {
        self.scene_center
    }

    /// Set the center of the scene, the camera looks at it and is moved with it.
    pub fn set_scene_center(&mut self, center: Vec3) {
        self.scene_center = center;

        let d = self.camera.view_center() - self.scene_center();

        self.camera.set_view_center(self.scene_center());
        self.camera.set_position(self.camera.position() - d);
    }

    /// The point, which the camera orbits around.
    pub fn pivot(&self) -> Vec3 {
        self.pivot
    }

    /// Set the point, which the camera orbits around.
    pub fn set_pivot(&mut self, v: Vec3) {
        if v != self.pivot {
            self.pivot = v;
            self.events.push(CameraEvent::PointOfInterestChanged);
        }
    }

    /// Set the pivot to the point under the screen coordinates in pixels, in the plane of the
    /// pivot parallel to the view.
    pub fn set_pivot_from_screen_coordinates(&mut self, screen_coords: DVec2) {
        self.set_pivot(self.world_point(screen_coords));
    }

    /// Sets the pivot to the scene center.
    pub fn reset_pivot(&mut self) {
        self.set_pivot(self.scene_center());
    }

    /// Start orbiting with the mouse.
    pub fn start_orbit(&mut self) {
        self.set_mode(Mode::Orbit);
    }

    /// Orbit the camera around the pivot by `pan` and `tilt` in degrees.
    pub fn orbit(&mut self, pan: f64, tilt: f64) {
        let camera = &mut self.camera;
        // Place camera on scene sphere's periphery
        camera.set_position(
            camera.view_center()
                - camera.view_vector().normalize() * self.scene_radius as f32 * 10.0,
        );

        let p = self.pivot;
        let v = p - camera.position();
        let w = p - camera.view_center();

        // Do pan rotation first
        let pq = camera.pan_rotation(-pan as f32);

        // Apply tilt rotation and set camera position and view angle
        let tq = camera.tilt_rotation(tilt as f32);
        camera.set_position(p - pq * tq * v);
        camera.set_view_center(p - pq * tq * w);

        let up = pq * tq * camera.up_vector();
        camera.set_up_vector(up.normalize());
    }

    /// Orbit by the movement of the mouse in pixels within the time `elapsed`, since the last
    /// frame or the start of orbiting.
    ///
    /// The angles are bounded to 15° per call.
    pub fn orbit_by_mouse(&mut self, mouse_delta: Vec2, elapsed: Duration) {
        let wr = self.window_size;
        let d = scaled_by_time(mouse_delta, elapsed);
        let delta = d.as_dvec2() * self.orbit_speed;
        let max_angle = 15.0;

        let pan = (delta.x / wr.x * 360.0).clamp(-max_angle, max_angle);
        let tilt = (-delta.y / wr.y * 360.0).clamp(-max_angle, max_angle);

        self.orbit(pan, tilt);
    }

    /// The speed of orbiting with the mouse.
    pub fn orbit_speed(&self) -> f64 {
        self.orbit_speed
    }

    /// Set the speed of orbiting with the mouse.
    pub fn set_orbit_speed(&mut self, orbit_speed: f64) {
        self.orbit_speed = orbit_speed;
    }

    /// Move the view center back to the scene center, while the view stays the same, i.e. the
    /// detail is moved by the offset of the view center instead.
    pub fn readjust_camera_position_and_detail(&mut self) {
        let to_camera = self.camera.transform_matrix().inverse();
        let get_point_2d = |v: Vec3| -> DVec2 {
            let p = to_camera * v.extend(1.0);
            (p / p.w).xy().as_dvec2()
        };

        let d = self.camera.view_center() - self.scene_center();
        let view_center_xy = get_point_2d(self.camera.view_center());
        let scene_center_xy = get_point_2d(self.scene_center());

        self.camera.set_view_center(self.scene_center());
        self.camera.set_position(self.camera.position() - d);
        self.zoom_rect_model
            .translate((view_center_xy - scene_center_xy).as_vec2());
        self.handle_zoom_events();
    }

    /// Start zooming with the mouse.
    pub fn start_zoom(&mut self) {
        self.set_mode(Mode::Zoom);
        self.update_camera_lens();
    }

    /// Zoom at the zoom point by the vertical delta of the mouse wheel within the time
    /// `elapsed`, since the last frame or the start of zooming: a positive `y` zooms in, a
    /// negative one out, with a positive zoom speed.
    pub fn zoom_by_wheel(&mut self, wheel_delta: Vec2, elapsed: Duration) {
        let d = scaled_by_time(wheel_delta, elapsed);
        let delta = -d.y as f64 * self.zoom_speed();
        let zoom_factor = 1.0 + delta / 100.0;

        self.zoom_rect_model
            .zoom_to_point(self.zoom_point(), zoom_factor);
        self.handle_zoom_events();
    }

    /// The zoom of the view, which is the projection of the camera.
    pub fn zoom_model(&self) -> &ZoomRectModel {
        &self.zoom_rect_model
    }

    /// The zoom level, 1 shows the whole scene.
    pub fn zoom_level(&self) -> f64 {
        self.zoom_rect_model.zoom_level()
    }

    /// Set the zoom level around the zoom point.
    pub fn set_zoom_level(&mut self, zoom_level: f64) {
        self.zoom_rect_model
            .set_zoom_level_at(zoom_level, self.zoom_point());
        self.handle_zoom_events();
    }

    /// Scale the detail by `zoom_factor` around the center of the view, e.g. by keys: a factor
    /// below 1 zooms in.
    pub fn zoom_at_center(&mut self, zoom_factor: f64) {
        let center = self.zoom_rect_model.detail().center();
        self.zoom_rect_model.zoom_to_point(center, zoom_factor);
        self.handle_zoom_events();
    }

    /// Show the whole scene.
    pub fn zoom_fit(&mut self) {
        self.zoom_rect_model.reset();
        self.handle_zoom_events();
    }

    /// The speed of zooming with the mouse, a negative speed inverts zooming.
    pub fn zoom_speed(&self) -> f64 {
        self.zoom_speed
    }

    /// Set the speed of zooming with the mouse, a negative speed inverts zooming.
    pub fn set_zoom_speed(&mut self, zoom_speed: f64) {
        self.zoom_speed = zoom_speed;
    }

    /// The point in the coordinates of the camera, which stays in place while zooming.
    pub fn zoom_point(&self) -> DVec2 {
        self.zoom_point
    }

    /// Set the zoom point to the point under the screen coordinates in pixels.
    pub fn set_zoom_point_from_screen_coordinates(&mut self, screen_coords: DVec2) {
        self.zoom_point = self.zoom_point_from_screen_coordinates(screen_coords);
    }

    /// The camera and the zoom, e.g. to restore them later.
    pub fn state(&self) -> CameraState {
        CameraState {
            view_center: self.camera.view_center(),
            position: self.camera.position(),
            up: self.camera.up_vector(),
            zoom: self.zoom_rect_model.state(),
        }
    }

    /// Restore the camera and the zoom.
    pub fn set_state(&mut self, state: &CameraState) {
        self.camera.set_view_center(state.view_center);
        self.camera.set_position(state.position);
        self.camera.set_up_vector(state.up);
        self.zoom_rect_model.set_state(state.zoom);
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

    /// Start translating with the mouse.
    pub fn start_translate(&mut self) {
        self.set_mode(Mode::Translation);
    }

    /// Translate by the movement of the mouse in pixels, the scene follows the mouse.
    pub fn translate_by_mouse(&mut self, mouse_delta: Vec2) {
        let d = mouse_delta;
        self.translate(Vec2::new(-d.x, d.y));
    }

    /// Translate the detail by `delta` in pixels, positive values move the detail to the
    /// right and upwards, i.e. the scene to the left and downwards.
    pub fn translate(&mut self, delta: Vec2) {
        let detail = self.zoom_rect_model.detail();
        let pixel = DVec2::new(
            detail.width / self.window_size.x,
            detail.height / self.window_size.y,
        );
        self.zoom_rect_model
            .translate((delta.as_dvec2() * pixel).as_vec2());
        self.handle_zoom_events();
    }

    /// The speed of translating with the mouse.
    pub fn translate_speed(&self) -> f64 {
        self.translate_speed
    }

    /// Set the speed of translating with the mouse.
    pub fn set_translate_speed(&mut self, translate_speed: f64) {
        self.translate_speed = translate_speed;
    }

    /// Stop orbiting, translating or zooming with the mouse.
    pub fn set_inactive(&mut self) {
        self.set_mode(Mode::Inactive);
    }

    /// What the mouse moves at the moment.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Whether the mouse moves something, then the viewer wraps the pointer around the
    /// edges of the window, so that the mouse moves on.
    pub fn pointer_wrapped(&self) -> bool {
        self.mode != Mode::Inactive
    }

    /// Whether the viewer keeps the pointer in place while orbiting or zooming, then the
    /// mouse movement is passed as delta.
    pub fn pointer_pinned(&self) -> bool {
        !matches!(self.mode, Mode::Inactive | Mode::Translation)
    }

    /// The kind of the projection of the camera.
    pub fn projection_type(&self) -> ProjectionType {
        self.camera.projection_type()
    }

    /// Set the kind of the projection of the camera.
    ///
    /// Only the orthographic projection of the detail is supported, `projection_type` is
    /// ignored.
    pub fn set_projection_type(&mut self, projection_type: ProjectionType) {
        let _ = projection_type;
        self.camera
            .set_projection_matrix(self.zoom_rect_model.detail_ortho_projection_matrix());
    }

    /// The ratio of the width to the height of the window.
    pub fn aspect_ratio(&self) -> f64 {
        self.window_size.x / self.window_size.y
    }

    /// Look at the scene center along `view_vector` with `up`.
    pub fn set_view(&mut self, view_vector: Vec3, up: Vec3) {
        let v = view_vector.normalize();
        self.camera.set_view_center(self.scene_center());
        self.camera.set_up_vector(up);
        self.camera
            .set_position(self.scene_center() - v * self.scene_radius as f32 * 10.0);

        self.update_camera_lens();
    }

    /// The ray through the screen coordinate in pixels.
    pub fn camera_ray(&self, pos: DVec2) -> Ray {
        // Normalized device coordinates
        let x = (2.0 * pos.x as f32) / self.window_size.x as f32 - 1.0;
        let y = 1.0 - (2.0 * pos.y as f32) / self.window_size.y as f32;
        let ndc = Vec4::new(x, y, -1.0, 1.0);

        // Eye coordinates
        let mut eye = self.camera.projection_matrix().inverse() * ndc;
        eye /= eye.w;

        // World coordinates
        let world = (self.camera.transform_matrix() * eye).xyz();

        Ray {
            origin: world,
            direction: self.camera.view_vector(),
        }
    }

    /// The point in the coordinates of the plane with `plane_transform`, which is under the
    /// screen coordinate in pixels.
    pub fn plane_coord_from_screen_coord(
        &self,
        screen_coord: DVec2,
        plane_transform: &Mat4,
    ) -> DVec2 {
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

    /// Apply the projection of the detail to the camera.
    fn update_camera_lens(&mut self) {
        self.set_projection_type(self.camera.projection_type());
    }

    /// React to the changes of the zoom, like the connected signals of Qt.
    fn handle_zoom_events(&mut self) {
        for event in self.zoom_rect_model.take_events() {
            match event {
                ZoomEvent::DetailChanged(_) | ZoomEvent::OverviewChanged(_) => {
                    self.update_camera_lens()
                }
                ZoomEvent::ZoomLevelChanged(_) => self.events.push(CameraEvent::ZoomLevelChanged),
            }
        }
    }

    fn set_mode(&mut self, mode: Mode) {
        if self.mode == Mode::Orbit && mode == Mode::Inactive {
            self.readjust_camera_position_and_detail();
        }

        self.mode = mode;
    }

    /// The point under the screen coordinate in pixels, in the plane of the pivot parallel
    /// to the view.
    fn world_point(&self, screen_coord: DVec2) -> Vec3 {
        let ray = self.camera_ray(screen_coord);

        // The virtual scene plane is defined by the camera viewVector and the
        // sceneCenter
        let view_vector = self.camera.view_vector();
        let dot = ray.direction.dot(view_vector);
        if dot.abs() <= 0.00001 {
            // The ray is parallel to the virtual plane
            return self.scene_center();
        }

        let t = (self.pivot() - ray.origin).dot(view_vector) / dot;

        // The intersection point is behind the origin of the ray
        // debug_assert!(t >= 0.0);
        ray.origin + ray.direction * t
    }

    /// The screen coordinate in pixels of a point in world coordinates.
    #[cfg_attr(not(test), expect(dead_code))]
    fn screen_coord(&self, world_point: Vec3) -> DVec2 {
        let ndc = self.normalized_screen_coord(world_point);

        DVec2::new(
            (ndc.x + 1.0) / 2.0 * self.window_size.x,
            (1.0 - ndc.y) / 2.0 * self.window_size.y,
        )
    }

    /// The normalized device coordinates of a point in world coordinates.
    fn normalized_screen_coord(&self, world_point: Vec3) -> DVec2 {
        let point_in_screen_space = self.camera.projection_matrix()
            * self.camera.transform_matrix().inverse()
            * world_point.extend(1.0);
        point_in_screen_space.xy().as_dvec2() / point_in_screen_space.w as f64
    }

    /// The point in the coordinates of the camera under the screen coordinate in pixels.
    fn zoom_point_from_screen_coordinates(&self, screen_coords: DVec2) -> DVec2 {
        let p =
            self.camera.transform_matrix().inverse() * self.world_point(screen_coords).extend(1.0);
        (p / p.w).xy().as_dvec2()
    }
}

/// The movement `d` of the mouse or its wheel multiplied with the elapsed time in seconds,
/// so that orbiting and zooming have the same speed at any frame rate.
fn scaled_by_time(d: Vec2, elapsed: Duration) -> Vec2 {
    d * elapsed.as_secs_f32()
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: DVec2 = DVec2::new(400.0, 200.0);

    #[test]
    fn new() {
        let mut controller = CameraController::new(WINDOW);
        let camera = controller.camera();
        // On the scene sphere's periphery, looking at the pivot from the front.
        assert!(
            camera
                .position()
                .abs_diff_eq(Vec3::new(0.0, 0.0, 1000.0), 1e-3)
        );
        assert_eq!(camera.view_center(), Vec3::ZERO);
        assert_eq!(camera.projection_type(), ProjectionType::Custom);
        assert_eq!(
            camera.projection_matrix(),
            controller.zoom_model().detail_ortho_projection_matrix()
        );
        assert_eq!(controller.take_events(), [CameraEvent::ZoomLevelChanged]);
    }

    #[test]
    fn screen_and_world() {
        let controller = CameraController::new(WINDOW);
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
        let mut controller = CameraController::new(WINDOW);
        controller.set_pivot(Vec3::new(10.0, 0.0, 0.0));
        let distance = |c: &CameraController| (c.camera().position() - c.pivot()).length();
        controller.orbit(0.0, 0.0);
        let before = distance(&controller);
        controller.orbit(30.0, 20.0);
        assert!((distance(&controller) - before).abs() < 1e-2);
        assert!(controller.camera().up_vector().is_normalized());
    }

    #[test]
    fn orbit_by_mouse_is_bounded() {
        let mut controller = CameraController::new(WINDOW);
        let before = controller.camera().view_vector().normalize();
        controller.orbit_by_mouse(Vec2::new(1e6, 0.0), Duration::from_secs(1));
        let after = controller.camera().view_vector().normalize();
        let angle = before.angle_between(after).to_degrees();
        assert!((angle - 15.0).abs() < 1e-2, "{angle}");
    }

    #[test]
    fn zoom_by_wheel() {
        let mut controller = CameraController::new(WINDOW);
        controller.set_zoom_point_from_screen_coordinates(WINDOW / 2.0);
        controller.start_zoom();
        // A positive delta zooms in, a negative one out.
        controller.zoom_by_wheel(Vec2::new(0.0, 2.0), Duration::from_secs(1));
        assert!((controller.zoom_level() - 2.0).abs() < 1e-9);
        controller.zoom_by_wheel(Vec2::new(0.0, -2.0), Duration::from_secs(1));
        assert!((controller.zoom_level() - 4.0 / 3.0).abs() < 1e-9);
        assert_eq!(
            controller.camera().projection_matrix(),
            controller.zoom_model().detail_ortho_projection_matrix()
        );
    }

    #[test]
    fn translate_follows_mouse() {
        let mut controller = CameraController::new(WINDOW);
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
        let mut controller = CameraController::new(WINDOW);
        controller.set_pivot(Vec3::new(50.0, 20.0, 0.0));
        controller.start_orbit();
        controller.orbit(20.0, 10.0);
        let point = Vec3::new(5.0, 6.0, 7.0);
        let before = controller.screen_coord(point);
        controller.set_inactive();
        assert_eq!(controller.camera().view_center(), controller.scene_center());
        let after = controller.screen_coord(point);
        assert!(after.abs_diff_eq(before, 1e-2), "{before} {after}");
    }

    #[test]
    fn state() {
        let mut controller = CameraController::new(WINDOW);
        let state = controller.state();
        controller.orbit(45.0, 30.0);
        controller.set_zoom_level(3.0);
        controller.set_state(&state);
        assert_eq!(controller.state(), state);
    }

    #[test]
    fn set_view() {
        let mut controller = CameraController::new(WINDOW);
        controller.set_view(Vec3::new(0.0, -1.0, 0.0), Vec3::Z);
        let camera = controller.camera();
        assert!(
            camera
                .position()
                .abs_diff_eq(Vec3::new(0.0, 1000.0, 0.0), 1e-3)
        );
    }

    #[test]
    fn plane_coord() {
        let controller = CameraController::new(WINDOW);
        let plane = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let coord = controller.plane_coord_from_screen_coord(WINDOW / 2.0, &plane);
        assert!(coord.abs_diff_eq(DVec2::new(-10.0, 0.0), 1e-3));
    }
}
