// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! WASM camera navigation API: the camera controller and the configurable controls of the
//! µcad viewers, e.g. for Three.js.
//!
//! The methods take the values of DOM events, e.g. `MouseEvent.button`, `WheelEvent.deltaY`
//! and `KeyboardEvent.key`, and the modifiers as bits, see [`CameraNavigation`].

use std::time::Duration;

use glam::{DVec2, Mat4, Vec2, Vec3};
use mucad_viewer_utils::{
    BoundingSphere, CameraControl, CameraEvent, Command, Controls, DEFAULT_CONTROLS_TOML,
    GridConfig, Modifiers, MouseAction, MouseButton, Navigation, Scalar, StandardView, Theme,
    TouchId,
};
use wasm_bindgen::prelude::*;

/// The bit of Ctrl in the modifiers.
pub const CTRL: u8 = 1;
/// The bit of Shift in the modifiers.
pub const SHIFT: u8 = 2;
/// The bit of Alt in the modifiers.
pub const ALT: u8 = 4;
/// The bit of Meta in the modifiers, e.g. Command on macOS.
pub const META: u8 = 8;

/// The pixels of a line of `WheelEvent.deltaMode` 1, which browsers do not tell.
const LINE_PIXELS: f32 = 16.0;

/// A click, which selects the object under the pointer in the viewer.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selection {
    x: f64,
    y: f64,
    multi: bool,
}

#[wasm_bindgen]
impl Selection {
    /// The x coordinate of the click in pixels from the left of the element.
    #[wasm_bindgen(getter)]
    pub fn x(&self) -> f64 {
        self.x
    }

    /// The y coordinate of the click in pixels from the top of the element.
    #[wasm_bindgen(getter)]
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Whether the object is added to the selection, instead of replacing it.
    #[wasm_bindgen(getter)]
    pub fn multi(&self) -> bool {
        self.multi
    }
}

/// The navigation of the camera of a viewer with the mouse and the keys.
///
/// The viewer passes the DOM events of its element and applies the matrices of the camera
/// after them. Coordinates are pixels from the top left of the element, e.g. `offsetX` and
/// `offsetY`. Modifiers are the bits [`CTRL`] (1), [`SHIFT`] (2), [`ALT`] (4) and [`META`] (8),
/// e.g. `ctrlKey | shiftKey << 1 | altKey << 2 | metaKey << 3`.
///
/// The projection maps to the normalized device coordinates of WebGL, from -1 to 1 in z, like
/// the projections of Three.js.
#[wasm_bindgen]
pub struct CameraNavigation {
    navigation: Navigation,
}

#[wasm_bindgen]
impl CameraNavigation {
    /// The navigation for an element of `width` and `height` in pixels with the controls in
    /// TOML, see [`CameraNavigation::default_controls`], or the default controls.
    ///
    /// Throws an error with the reason for invalid controls.
    #[wasm_bindgen(constructor)]
    pub fn new(
        width: f64,
        height: f64,
        controls_toml: Option<String>,
    ) -> Result<CameraNavigation, JsError> {
        console_error_panic_hook::set_once();
        let controls = match controls_toml {
            Some(toml) => parse_controls(&toml).map_err(|error| JsError::new(&error))?,
            None => Controls::default(),
        };
        Ok(Self {
            navigation: Navigation::new(DVec2::new(width, height), controls),
        })
    }

    /// The default controls as TOML with comments, e.g. to start a configuration.
    #[wasm_bindgen(js_name = defaultControls)]
    pub fn default_controls() -> String {
        DEFAULT_CONTROLS_TOML.to_string()
    }

    /// Set the controls in TOML, missing entries are the defaults.
    ///
    /// Throws an error with the reason for invalid controls, which keeps the previous ones.
    #[wasm_bindgen(js_name = setControls)]
    pub fn set_controls(&mut self, controls_toml: &str) -> Result<(), JsError> {
        let controls = parse_controls(controls_toml).map_err(|error| JsError::new(&error))?;
        self.navigation.set_controls(controls);
        Ok(())
    }

    /// Set the size of the element in pixels, e.g. after resizing it.
    pub fn resize(&mut self, width: f64, height: f64) {
        self.navigation.resize(DVec2::new(width, height));
    }

    /// Set the bounds of what the camera shows: their center and their radius, e.g. of the
    /// bounding sphere of the model.
    #[wasm_bindgen(js_name = setBounds)]
    pub fn set_bounds(&mut self, x: f32, y: f32, z: f32, radius: f32) {
        self.navigation
            .controller_mut()
            .set_bounds(BoundingSphere::new(Vec3::new(x, y, z), radius as Scalar));
    }

    /// Press the button of `MouseEvent.button` at `(x, y)`, other buttons than the left (0),
    /// middle (1) and right (2) one are ignored.
    #[wasm_bindgen(js_name = mouseDown)]
    pub fn mouse_down(&mut self, button: i16, x: f64, y: f64, modifiers: u8) {
        if let Some(button) = mouse_button(button) {
            self.navigation
                .mouse_pressed(button, to_modifiers(modifiers), DVec2::new(x, y));
        }
    }

    /// Release the button of `MouseEvent.button` at `(x, y)`, returns a selection after a
    /// click.
    #[wasm_bindgen(js_name = mouseUp)]
    pub fn mouse_up(&mut self, button: i16, x: f64, y: f64, modifiers: u8) -> Option<Selection> {
        let button = mouse_button(button)?;
        let command =
            self.navigation
                .mouse_released(button, to_modifiers(modifiers), DVec2::new(x, y))?;
        Some(selection(command))
    }

    /// Start the touch with `Touch.identifier` at `(x, y)`.
    #[wasm_bindgen(js_name = touchStart)]
    pub fn touch_start(&mut self, id: f64, x: f64, y: f64) {
        self.navigation
            .touch_started(id as TouchId, DVec2::new(x, y));
    }

    /// Move the touches of one `touchmove` event within `elapsed_ms` milliseconds, since the
    /// last frame: `touches` are `Touch.identifier`, x and y of each touch one after another,
    /// e.g. `[id1, x1, y1, id2, x2, y2]`.
    ///
    /// One finger orbits, two fingers pinch to zoom and pan.
    #[wasm_bindgen(js_name = touchMove)]
    pub fn touch_move(&mut self, touches: &[f64], elapsed_ms: f64) {
        self.navigation
            .touches_moved(&to_touches(touches), duration(elapsed_ms));
    }

    /// End the touch with `Touch.identifier` at `(x, y)`, returns a selection after a tap.
    #[wasm_bindgen(js_name = touchEnd)]
    pub fn touch_end(&mut self, id: f64, x: f64, y: f64) -> Option<Selection> {
        let command = self
            .navigation
            .touch_ended(id as TouchId, DVec2::new(x, y))?;
        Some(selection(command))
    }

    /// Cancel the touch with `Touch.identifier`, e.g. after `touchcancel`.
    #[wasm_bindgen(js_name = touchCancel)]
    pub fn touch_cancel(&mut self, id: f64) {
        self.navigation.touch_cancelled(id as TouchId);
    }

    /// Move the mouse by `dx` and `dy` in pixels within `elapsed_ms` milliseconds, since the
    /// last frame: the movement of the pointer, e.g. the difference of `clientX` and `clientY`,
    /// so that panning follows it exactly.
    #[wasm_bindgen(js_name = mouseMove)]
    pub fn mouse_move(&mut self, dx: f32, dy: f32, elapsed_ms: f64) {
        self.navigation
            .mouse_moved(Vec2::new(dx, dy), duration(elapsed_ms));
    }

    /// Turn the wheel by `WheelEvent.deltaY` in its `deltaMode` at `(x, y)` within
    /// `elapsed_ms` milliseconds, since the last frame: turning it away zooms in.
    ///
    /// Returns whether the wheel is bound, then the event is prevented, e.g. not scrolling.
    pub fn wheel(
        &mut self,
        delta_y: f64,
        delta_mode: u32,
        x: f64,
        y: f64,
        modifiers: u8,
        elapsed_ms: f64,
    ) -> bool {
        let height = self.navigation.controller().window_size().y as f32;
        self.navigation.wheel(
            wheel_delta(delta_y as f32, delta_mode, height),
            to_modifiers(modifiers),
            DVec2::new(x, y),
            duration(elapsed_ms),
        )
    }

    /// Press the key of `KeyboardEvent.key`.
    ///
    /// Returns whether the key is bound, then the event is prevented, e.g. zooming the page
    /// with Ctrl and `+`.
    #[wasm_bindgen(js_name = keyDown)]
    pub fn key_down(&mut self, key: &str, modifiers: u8) -> bool {
        self.navigation.key_pressed(key, to_modifiers(modifiers))
    }

    /// The action of the pressed buttons, e.g. for the cursor: `select`, `orbit`, `pan`,
    /// `zoom`, or none.
    #[wasm_bindgen(js_name = mouseAction)]
    pub fn mouse_action(&self) -> Option<String> {
        let action = match self.navigation.mouse_action()? {
            MouseAction::Select { .. } => "select",
            MouseAction::Orbit => "orbit",
            MouseAction::Pan => "pan",
            MouseAction::Zoom => "zoom",
        };
        Some(action.to_string())
    }

    /// Whether the pointer is locked while orbiting or zooming, e.g. with
    /// `requestPointerLock`, then the movement is passed with [`CameraNavigation::mouse_move`].
    #[wasm_bindgen(js_name = pointerPinned)]
    pub fn pointer_pinned(&self) -> bool {
        self.navigation.controller().pointer_pinned()
    }

    /// Whether the mouse moves the camera, e.g. to capture the pointer.
    #[wasm_bindgen(js_name = pointerWrapped)]
    pub fn pointer_wrapped(&self) -> bool {
        self.navigation.controller().pointer_wrapped()
    }

    /// Show a standard view: `isometric`, `front`, `top`, `right`, `rear`, `bottom` or
    /// `left`.
    ///
    /// Throws an error for another name.
    #[wasm_bindgen(js_name = setView)]
    pub fn set_view(&mut self, name: &str) -> Result<(), JsError> {
        let view = standard_view(name)
            .ok_or_else(|| JsError::new(&format!("unknown standard view `{name}`")))?;
        self.navigation
            .controller_mut()
            .set_view(view.view_vector(), view.up());
        Ok(())
    }

    /// Show the whole scene in the middle of the view, e.g. to reset the camera.
    #[wasm_bindgen(js_name = showAll)]
    pub fn show_all(&mut self) {
        self.navigation.controller_mut().show_all();
    }

    /// Show the whole scene with the zoom level 1 around the center of the view.
    #[wasm_bindgen(js_name = zoomFit)]
    pub fn zoom_fit(&mut self) {
        self.navigation.controller_mut().zoom_fit();
    }

    /// The zoom level, 1 shows the whole scene.
    #[wasm_bindgen(js_name = zoomLevel)]
    pub fn zoom_level(&self) -> f64 {
        self.navigation.controller().zoom_level()
    }

    /// The point, which the camera orbits around, as `[x, y, z]`.
    pub fn pivot(&self) -> Vec<f32> {
        self.navigation.controller().pivot().to_array().to_vec()
    }

    /// Set the pivot to the center of the bounds.
    #[wasm_bindgen(js_name = resetPivot)]
    pub fn reset_pivot(&mut self) {
        self.navigation.controller_mut().reset_pivot();
    }

    /// The parameters of the grid shader for the camera with the colors of the default theme,
    /// like in the Bevy viewer: the members of `GridConfig` as floats one after another, e.g.
    /// `[radius, zoom_level, view_angle.x, view_angle.y, …]`, see `grid.uniformBlocks` of the
    /// generated `grid.ts` of the web components.
    #[wasm_bindgen(js_name = gridConfig)]
    pub fn grid_config(&self) -> Vec<f32> {
        grid_floats(&GridConfig::new(
            self.navigation.controller(),
            &Theme::default(),
        ))
    }

    /// The changes since the last call: `zoomLevelChanged` and `pivotChanged`.
    #[wasm_bindgen(js_name = takeEvents)]
    pub fn take_events(&mut self) -> Vec<String> {
        self.navigation
            .controller_mut()
            .take_events()
            .into_iter()
            .map(|event| {
                match event {
                    CameraEvent::ZoomLevelChanged => "zoomLevelChanged",
                    CameraEvent::PointOfInterestChanged => "pivotChanged",
                }
                .to_string()
            })
            .collect()
    }

    /// The position of the camera as `[x, y, z]`.
    pub fn position(&self) -> Vec<f32> {
        self.camera().position().to_array().to_vec()
    }

    /// The point, which the camera looks at, as `[x, y, z]`.
    #[wasm_bindgen(js_name = viewCenter)]
    pub fn view_center(&self) -> Vec<f32> {
        self.camera().view_center().to_array().to_vec()
    }

    /// The direction, which is upwards in the view, as `[x, y, z]`.
    pub fn up(&self) -> Vec<f32> {
        self.camera().up_vector().to_array().to_vec()
    }

    /// The transformation from the camera to the world, e.g. `Object3D.matrixWorld` of the
    /// camera in Three.js, as 16 numbers by columns.
    #[wasm_bindgen(js_name = cameraMatrix)]
    pub fn camera_matrix(&self) -> Vec<f32> {
        columns(self.camera().transform_matrix())
    }

    /// The transformation from the world to the camera, e.g. `matrixWorldInverse` of a camera
    /// in Three.js, as 16 numbers by columns.
    #[wasm_bindgen(js_name = viewMatrix)]
    pub fn view_matrix(&self) -> Vec<f32> {
        columns(self.camera().view_matrix())
    }

    /// The projection, e.g. `projectionMatrix` of a camera in Three.js, as 16 numbers by
    /// columns.
    #[wasm_bindgen(js_name = projectionMatrix)]
    pub fn projection_matrix(&self) -> Vec<f32> {
        columns(self.camera().projection_matrix())
    }
}

impl CameraNavigation {
    fn camera(&self) -> &mucad_viewer_utils::Camera {
        self.navigation.controller().camera()
    }
}

/// The fields of `config` as floats one after another in their order, which is the one of the
/// WGSL struct.
fn grid_floats(config: &GridConfig) -> Vec<f32> {
    let mut floats = vec![config.radius, config.zoom_level];
    for vector in [
        config.view_angle,
        config.grid_color,
        config.x_axis_color,
        config.y_axis_color,
    ] {
        floats.extend(vector.to_array());
    }
    floats
}

/// The selection of a command.
fn selection(command: Command) -> Selection {
    match command {
        Command::Select { position, multi } => Selection {
            x: position.x,
            y: position.y,
            multi,
        },
    }
}

/// The touches of identifiers, x and y one after another, an incomplete last one is ignored.
fn to_touches(values: &[f64]) -> Vec<(TouchId, DVec2)> {
    let (touches, _) = values.as_chunks::<3>();
    touches
        .iter()
        .map(|[id, x, y]| (*id as TouchId, DVec2::new(*x, *y)))
        .collect()
}

/// The controls in TOML or the reason, why they are invalid.
fn parse_controls(toml: &str) -> Result<Controls, String> {
    Controls::from_toml(toml).map_err(|error| format!("invalid controls: {error}"))
}

/// The button of `MouseEvent.button`.
fn mouse_button(button: i16) -> Option<MouseButton> {
    match button {
        0 => Some(MouseButton::Left),
        1 => Some(MouseButton::Middle),
        2 => Some(MouseButton::Right),
        _ => None,
    }
}

/// The modifiers of the bits [`CTRL`], [`SHIFT`], [`ALT`] and [`META`].
fn to_modifiers(bits: u8) -> Modifiers {
    Modifiers {
        ctrl: bits & CTRL != 0,
        shift: bits & SHIFT != 0,
        alt: bits & ALT != 0,
        meta: bits & META != 0,
    }
}

/// The delta of the wheel in pixels, positive away from the user, of `WheelEvent.deltaY`,
/// which is positive towards the user, in pixels (0), lines (1) or pages (2) of `height`.
fn wheel_delta(delta_y: f32, delta_mode: u32, height: f32) -> f32 {
    let pixels = match delta_mode {
        1 => delta_y * LINE_PIXELS,
        2 => delta_y * height,
        _ => delta_y,
    };
    -pixels
}

/// The standard view of `name`, e.g. `front`.
fn standard_view(name: &str) -> Option<StandardView> {
    Some(match name {
        "isometric" => StandardView::Isometric,
        "front" => StandardView::Front,
        "top" => StandardView::Top,
        "right" => StandardView::Right,
        "rear" => StandardView::Rear,
        "bottom" => StandardView::Bottom,
        "left" => StandardView::Left,
        _ => return None,
    })
}

/// The duration of `ms` milliseconds, 0 for negative or invalid times.
fn duration(ms: f64) -> Duration {
    Duration::try_from_secs_f64(ms / 1000.0).unwrap_or_default()
}

/// The 16 numbers of `matrix` by columns, like `Matrix4.fromArray` of Three.js.
fn columns(matrix: Mat4) -> Vec<f32> {
    matrix.to_cols_array().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers() {
        assert_eq!(to_modifiers(0), Modifiers::NONE);
        assert_eq!(to_modifiers(CTRL), Modifiers::CTRL);
        let all = to_modifiers(CTRL | SHIFT | ALT | META);
        assert!(all.ctrl && all.shift && all.alt && all.meta);
    }

    #[test]
    fn buttons() {
        assert_eq!(mouse_button(0), Some(MouseButton::Left));
        assert_eq!(mouse_button(1), Some(MouseButton::Middle));
        assert_eq!(mouse_button(2), Some(MouseButton::Right));
        assert_eq!(mouse_button(3), None);
    }

    #[test]
    fn wheel() {
        // Scrolling towards the user is positive in the DOM, but zooms out.
        assert_eq!(wheel_delta(100.0, 0, 500.0), -100.0);
        assert_eq!(wheel_delta(-3.0, 1, 500.0), 48.0);
        assert_eq!(wheel_delta(1.0, 2, 500.0), -500.0);
    }

    #[test]
    fn durations() {
        assert_eq!(duration(16.0), Duration::from_millis(16));
        assert_eq!(duration(-1.0), Duration::ZERO);
        assert_eq!(duration(f64::NAN), Duration::ZERO);
    }

    #[test]
    fn touches() {
        assert_eq!(
            to_touches(&[1.0, 10.0, 20.0, 2.0, 30.0, 40.0, 3.0]),
            [(1, DVec2::new(10.0, 20.0)), (2, DVec2::new(30.0, 40.0))]
        );
    }

    #[test]
    fn views() {
        assert_eq!(standard_view("front"), Some(StandardView::Front));
        assert_eq!(standard_view("Front"), None);
    }

    #[test]
    fn controls() {
        assert!(parse_controls(DEFAULT_CONTROLS_TOML).is_ok());
        let error = parse_controls("[mouse]\npan = [\"Up\"]\n").expect_err("invalid");
        assert!(error.starts_with("invalid controls:"), "{error}");
    }
}
