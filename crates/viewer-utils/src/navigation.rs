// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The navigation with the mouse, the keys and touch: input events of a viewer control the
//! camera by the configured [`Controls`].

use std::time::Duration;

use glam::{DVec2, Vec2, Vec3};

use crate::{
    CameraController, Controls, KeyBinding, Modifiers, MouseButton, MouseInput, TouchDrag,
};

/// The distance in pixels, which the mouse can move while clicking.
const CLICK_TOLERANCE: f32 = 4.0;

/// A standard view of the scene with Z upwards, written in lower case, e.g. `front`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString, strum::IntoStaticStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum StandardView {
    /// From the front, the right and the top.
    Isometric,
    /// From the front, along +Y.
    Front,
    /// From the top, along -Z.
    Top,
    /// From the right, along -X.
    Right,
    /// From the rear, along -Y.
    Rear,
    /// From the bottom, along +Z.
    Bottom,
    /// From the left, along +X.
    Left,
}

impl StandardView {
    /// The direction, which the camera looks into.
    pub fn view_vector(self) -> Vec3 {
        match self {
            Self::Isometric => Vec3::new(-1.0, 1.0, -1.0).normalize(),
            Self::Front => Vec3::Y,
            Self::Top => Vec3::NEG_Z,
            Self::Right => Vec3::NEG_X,
            Self::Rear => Vec3::NEG_Y,
            Self::Bottom => Vec3::Z,
            Self::Left => Vec3::X,
        }
    }

    /// The direction, which is upwards in the view, orthogonal to the view vector.
    pub fn up(self) -> Vec3 {
        match self {
            Self::Isometric => Vec3::new(-1.0, 1.0, 2.0).normalize(),
            Self::Front | Self::Right | Self::Rear | Self::Left => Vec3::Z,
            Self::Top => Vec3::Y,
            Self::Bottom => Vec3::NEG_Y,
        }
    }
}

/// What the viewer does after an input, which the camera controller cannot do itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    /// Select the object under the screen coordinate in pixels, `multi` adds it to the
    /// selection.
    Select {
        /// The screen coordinate in pixels.
        position: DVec2,
        /// Whether the object is added to the selection, instead of replacing it.
        multi: bool,
    },
}

/// The action of the pressed mouse buttons, written in lower case, e.g. `orbit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum MouseAction {
    /// Select by clicking, `multi` adds to the selection.
    Select {
        /// Whether the object is added to the selection, instead of replacing it.
        multi: bool,
    },
    /// Orbit by dragging.
    Orbit,
    /// Pan by dragging.
    Pan,
    /// Zoom by dragging.
    Zoom,
}

/// The identifier of a touch, i.e. of a finger on the screen, e.g. `PointerEvent.pointerId`.
pub type TouchId = i64;

/// The navigation of a [`CameraController`] with the mouse, the keys and touch.
///
/// The viewer passes its input events, the modifiers and the pointer position in pixels from
/// the top left of the window, and applies the camera of the controller after them.
///
/// Touch: one finger orbits or pans, see [`crate::TouchControls`], two fingers pinch to zoom
/// and pan, a tap selects. More fingers are ignored. While
/// [`CameraControl::pointer_pinned`](crate::CameraControl::pointer_pinned), it passes the movement of the mouse, e.g. of a locked
/// pointer, also when the pointer position stays the same.
#[derive(Debug, Clone)]
pub struct Navigation {
    controller: CameraController,
    controls: Controls,
    /// The pressed buttons in their order.
    pressed: Vec<MouseButton>,
    action: Option<MouseAction>,
    /// The distance, which the mouse moved since the first button was pressed.
    moved: f32,
    /// Whether only a select action has been started since the first button was pressed.
    clicking: bool,
    /// The drag of the pressed buttons, which also select, until the mouse moves further than
    /// a click, e.g. panning with the left button.
    pending_drag: Option<MouseAction>,
    /// The movement since the pending drag, which is applied when it starts.
    pending_delta: Vec2,
    /// The position, where the buttons were pressed, e.g. to orbit around.
    press_position: DVec2,
    /// The touches with their positions in the order they started.
    touches: Vec<(TouchId, DVec2)>,
    /// The distance, which the touches moved since the first one started.
    touch_moved: f32,
    /// Whether only one finger touched since the first one started, e.g. for a tap.
    tapping: bool,
}

impl Navigation {
    /// The navigation for a window of `window_size` in pixels with `controls`.
    pub fn new(window_size: DVec2, controls: Controls) -> Self {
        let mut navigation = Self {
            controller: CameraController::new(window_size),
            controls: Controls::default(),
            pressed: Vec::new(),
            action: None,
            moved: 0.0,
            clicking: false,
            pending_drag: None,
            pending_delta: Vec2::ZERO,
            press_position: DVec2::ZERO,
            touches: Vec::new(),
            touch_moved: 0.0,
            tapping: false,
        };
        navigation.set_controls(controls);
        navigation
    }

    /// The camera controller.
    pub fn controller(&self) -> &CameraController {
        &self.controller
    }

    /// The camera controller, e.g. to set its bounds.
    pub fn controller_mut(&mut self) -> &mut CameraController {
        &mut self.controller
    }

    /// The controls.
    pub fn controls(&self) -> &Controls {
        &self.controls
    }

    /// Set the controls, e.g. after the configuration is changed.
    ///
    /// The projection mode becomes the one of the controls, e.g. at the start.
    pub fn set_controls(&mut self, controls: Controls) {
        self.controller.set_config(controls.camera);
        self.controller
            .set_projection_mode(controls.camera.projection);
        self.controls = controls;
    }

    /// Set the size of the window in pixels, e.g. after resizing it.
    pub fn resize(&mut self, window_size: DVec2) {
        self.controller.set_window_size(window_size);
    }

    /// The action of the pressed mouse buttons.
    pub fn mouse_action(&self) -> Option<MouseAction> {
        self.action
    }

    /// Press `button` with `modifiers` at `position`.
    pub fn mouse_pressed(&mut self, button: MouseButton, modifiers: Modifiers, position: DVec2) {
        if self.pressed.is_empty() {
            self.moved = 0.0;
            self.clicking = true;
        }
        if !self.pressed.contains(&button) {
            self.pressed.push(button);
        }
        self.update_action(modifiers, position);
    }

    /// Release `button` with `modifiers` at `position`, returns a command, e.g. after a click.
    pub fn mouse_released(
        &mut self,
        button: MouseButton,
        modifiers: Modifiers,
        position: DVec2,
    ) -> Option<Command> {
        let command = match self.action {
            Some(MouseAction::Select { multi })
                if self.clicking && self.moved <= CLICK_TOLERANCE =>
            {
                Some(Command::Select { position, multi })
            }
            _ => None,
        };
        self.pressed.retain(|pressed| *pressed != button);
        self.update_action(modifiers, position);
        command
    }

    /// Move the mouse by `delta` in pixels within the time `elapsed`, since the last frame.
    ///
    /// The delta is the movement of the pointer, so that panning follows it exactly, or of the
    /// mouse, while the pointer is pinned.
    pub fn mouse_moved(&mut self, delta: Vec2, elapsed: Duration) {
        if !self.pressed.is_empty() {
            self.moved += delta.length();
        }
        if let Some(drag) = self.pending_drag {
            self.pending_delta += delta;
            if self.moved <= CLICK_TOLERANCE {
                return;
            }
            // No click, the drag starts with the movement so far, e.g. the model stays under
            // the pointer while panning.
            self.pending_drag = None;
            self.set_action(Some(drag), self.press_position);
            let delta = std::mem::take(&mut self.pending_delta);
            self.drag(delta, elapsed);
            return;
        }
        self.drag(delta, elapsed);
    }

    /// Orbit, pan or zoom by `delta` with the action.
    fn drag(&mut self, delta: Vec2, elapsed: Duration) {
        match self.action {
            Some(MouseAction::Orbit) => self.controller.orbit_by_mouse(delta, elapsed),
            Some(MouseAction::Pan) => self.controller.translate_by_mouse(delta),
            Some(MouseAction::Zoom) => self.controller.zoom_by_mouse(delta, elapsed),
            Some(MouseAction::Select { .. }) | None => {}
        }
    }

    /// Turn the wheel by `delta` in pixels with `modifiers` at `position` within the time
    /// `elapsed`, since the last frame: a positive delta, away from the user, zooms in.
    ///
    /// Returns whether the wheel is bound, otherwise the viewer can use it, e.g. to scroll.
    pub fn wheel(
        &mut self,
        delta: f32,
        modifiers: Modifiers,
        position: DVec2,
        elapsed: Duration,
    ) -> bool {
        let bound =
            self.controls.mouse.zoom.iter().any(|binding| {
                binding.input == MouseInput::Wheel && binding.modifiers == modifiers
            });
        if bound {
            self.controller
                .set_zoom_point_from_screen_coordinates(position);
            self.controller
                .zoom_by_wheel(Vec2::new(0.0, delta), elapsed);
        }
        bound
    }

    /// Start the touch `id` at `position`.
    pub fn touch_started(&mut self, id: TouchId, position: DVec2) {
        if self.touches.is_empty() {
            self.touch_moved = 0.0;
            self.tapping = true;
        }
        self.touches.retain(|(touch, _)| *touch != id);
        self.touches.push((id, position));
        if self.touches.len() > 1 {
            self.tapping = false;
        }
        self.start_touch_gesture();
    }

    /// Move the touches to their positions within the time `elapsed`, since the last frame:
    /// one finger orbits or pans, two fingers pinch to zoom at their center and pan with it.
    ///
    /// The touches, which move together, e.g. of one `touchmove` event, are moved in one call,
    /// so that moving two fingers in parallel pans without zooming.
    pub fn touches_moved(&mut self, moved: &[(TouchId, DVec2)], elapsed: Duration) {
        let previous: Vec<DVec2> = self.touches.iter().map(|(_, position)| *position).collect();
        for (id, position) in moved {
            if let Some(touch) = self.touches.iter_mut().find(|(touch, _)| touch == id) {
                touch.1 = *position;
            }
        }
        let current: Vec<DVec2> = self.touches.iter().map(|(_, position)| *position).collect();
        self.touch_moved += current
            .iter()
            .zip(&previous)
            .map(|(current, previous)| current.distance(*previous) as f32)
            .sum::<f32>();
        match (current.as_slice(), previous.as_slice()) {
            ([current], [previous]) => {
                let delta = (*current - *previous).as_vec2();
                match self.controls.touch.one_finger {
                    TouchDrag::Orbit => self.controller.orbit_by_mouse(delta, elapsed),
                    TouchDrag::Pan => self.controller.translate_by_mouse(delta),
                }
            }
            ([a, b], [previous_a, previous_b]) => {
                let center = (*a + *b) / 2.0;
                let previous_center = (*previous_a + *previous_b) / 2.0;
                self.controller
                    .translate_by_mouse((center - previous_center).as_vec2());
                let distance = a.distance(*b);
                let previous_distance = previous_a.distance(*previous_b);
                if distance > 0.0 && previous_distance > 0.0 {
                    // Spreading the fingers zooms in.
                    self.controller
                        .zoom_at_screen_point(center, previous_distance / distance);
                }
            }
            _ => {}
        }
    }

    /// End the touch `id` at `position`, returns a command, e.g. after a tap.
    pub fn touch_ended(&mut self, id: TouchId, position: DVec2) -> Option<Command> {
        let tap = self.touches.len() == 1
            && self.touches[0].0 == id
            && self.tapping
            && self.touch_moved <= CLICK_TOLERANCE;
        self.touch_cancelled(id);
        tap.then_some(Command::Select {
            position,
            multi: false,
        })
    }

    /// Cancel the touch `id`, e.g. when the browser takes it over, without a command.
    pub fn touch_cancelled(&mut self, id: TouchId) {
        let count = self.touches.len();
        self.touches.retain(|(touch, _)| *touch != id);
        if self.touches.len() != count {
            self.start_touch_gesture();
        }
    }

    /// End the previous gesture and start the one of the touches, e.g. orbiting around the
    /// point under one finger.
    fn start_touch_gesture(&mut self) {
        self.controller.set_inactive();
        match self.touches.as_slice() {
            [(_, position)] => match self.controls.touch.one_finger {
                TouchDrag::Orbit => {
                    self.controller.set_pivot_from_screen_coordinates(*position);
                    self.controller.start_orbit();
                }
                TouchDrag::Pan => self.controller.start_translate(),
            },
            [_, _] => self.controller.start_translate(),
            _ => {}
        }
    }

    /// Press `key`, a logical key like `KeyboardEvent.key` of the web, with `modifiers`.
    ///
    /// Returns whether the key is bound, otherwise the viewer can use it.
    pub fn key_pressed(&mut self, key: &str, modifiers: Modifiers) -> bool {
        let keyboard = &self.controls.keyboard;
        let bound = |bindings: &[KeyBinding]| {
            bindings
                .iter()
                .any(|binding| binding.matches(key, modifiers))
        };
        let zoom_step = self.controls.zoom_step;
        let pan = self.controls.pan_step as f32 * self.controller.window_size().as_vec2();
        let rotate_step = self.controls.rotate_step;
        let view = [
            (&keyboard.view_isometric, StandardView::Isometric),
            (&keyboard.view_front, StandardView::Front),
            (&keyboard.view_top, StandardView::Top),
            (&keyboard.view_right, StandardView::Right),
            (&keyboard.view_rear, StandardView::Rear),
            (&keyboard.view_bottom, StandardView::Bottom),
            (&keyboard.view_left, StandardView::Left),
        ]
        .into_iter()
        .find_map(|(bindings, view)| bound(bindings).then_some(view));

        if bound(&keyboard.zoom_in) {
            self.controller.zoom_at_center(1.0 / zoom_step);
        } else if bound(&keyboard.zoom_out) {
            self.controller.zoom_at_center(zoom_step);
        } else if bound(&keyboard.zoom_fit) {
            self.controller.show_all();
        } else if bound(&keyboard.toggle_projection) {
            self.controller.toggle_projection();
        } else if bound(&keyboard.pan_left) {
            self.controller.translate(Vec2::new(-pan.x, 0.0));
        } else if bound(&keyboard.pan_right) {
            self.controller.translate(Vec2::new(pan.x, 0.0));
        } else if bound(&keyboard.pan_up) {
            self.controller.translate(Vec2::new(0.0, pan.y));
        } else if bound(&keyboard.pan_down) {
            self.controller.translate(Vec2::new(0.0, -pan.y));
        } else if bound(&keyboard.rotate_left) {
            self.rotate(-rotate_step);
        } else if bound(&keyboard.rotate_right) {
            self.rotate(rotate_step);
        } else if let Some(view) = view {
            self.controller.set_view(view.view_vector(), view.up());
        } else {
            return false;
        }
        true
    }

    /// Orbit around the pivot by `pan` in degrees like dragging the mouse to the right, and
    /// move the view center back to the center of the bounds like after orbiting with the mouse.
    fn rotate(&mut self, pan: f64) {
        self.controller.orbit(pan, 0.0);
        // The orthographic view center goes back to the center of the bounds like after
        // orbiting with the mouse, without ending a drag.
        if let Some(ortho) = self.controller.ortho_mut() {
            ortho.readjust_camera_position_and_detail();
        }
    }

    /// Start the action of the pressed buttons, and end the previous one.
    ///
    /// Buttons, which select and drag, select until the mouse moves further than a click,
    /// unless they drag after another drag, e.g. after releasing a button of a chord.
    fn update_action(&mut self, modifiers: Modifiers, position: DVec2) {
        let select = self.matching_select(modifiers);
        let drag = self.matching_drag(modifiers);
        let (action, pending_drag) = match (select, drag) {
            (Some(select), Some(drag)) if self.clicking => (Some(select), Some(drag)),
            (_, Some(drag)) => (Some(drag), None),
            (select, None) => (select, None),
        };
        self.pending_drag = pending_drag;
        self.pending_delta = Vec2::ZERO;
        self.press_position = position;
        self.set_action(action, position);
    }

    /// Start `action` at `position` and end the previous one.
    fn set_action(&mut self, action: Option<MouseAction>, position: DVec2) {
        if action == self.action {
            return;
        }
        if matches!(
            self.action,
            Some(MouseAction::Orbit | MouseAction::Pan | MouseAction::Zoom)
        ) {
            self.controller.set_inactive();
        }
        self.action = action;
        match action {
            Some(MouseAction::Orbit) => {
                self.controller.set_pivot_from_screen_coordinates(position);
                self.controller.start_orbit();
            }
            Some(MouseAction::Pan) => self.controller.start_translate(),
            Some(MouseAction::Zoom) => {
                self.controller
                    .set_zoom_point_from_screen_coordinates(position);
                self.controller.start_zoom();
            }
            Some(MouseAction::Select { .. }) | None => {}
        }
        if matches!(
            action,
            Some(MouseAction::Orbit | MouseAction::Pan | MouseAction::Zoom)
        ) {
            self.clicking = false;
        }
    }

    /// The drag, whose binding are the pressed buttons with `modifiers`.
    fn matching_drag(&self, modifiers: Modifiers) -> Option<MouseAction> {
        if self.pressed.is_empty() {
            return None;
        }
        let mouse = &self.controls.mouse;
        let buttons = MouseInput::Buttons(self.pressed.clone());
        [
            (&mouse.orbit, MouseAction::Orbit),
            (&mouse.pan, MouseAction::Pan),
            (&mouse.zoom, MouseAction::Zoom),
        ]
        .into_iter()
        .find_map(|(bindings, action)| {
            bindings
                .iter()
                .any(|binding| binding.input == buttons && binding.modifiers == modifiers)
                .then_some(action)
        })
    }

    /// The selection, whose binding are the pressed buttons with `modifiers`.
    fn matching_select(&self, modifiers: Modifiers) -> Option<MouseAction> {
        if self.pressed.is_empty() {
            return None;
        }
        let mouse = &self.controls.mouse;
        let buttons = MouseInput::Buttons(self.pressed.clone());
        // Selecting with the multi select modifiers adds to the selection.
        let multi_select = self.controls.keyboard.multi_select;
        mouse
            .select
            .iter()
            .filter(|binding| binding.input == buttons)
            .find_map(|binding| {
                if binding.modifiers == modifiers {
                    Some(MouseAction::Select { multi: false })
                } else if !multi_select.is_empty()
                    && !binding.modifiers.contains(multi_select)
                    && binding.modifiers.union(multi_select) == modifiers
                {
                    Some(MouseAction::Select { multi: true })
                } else {
                    None
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Mode, ProjectionMode};
    use MouseButton::*;

    const WINDOW: DVec2 = DVec2::new(400.0, 200.0);
    const CENTER: DVec2 = DVec2::new(200.0, 100.0);
    const FRAME: Duration = Duration::from_millis(16);

    fn navigation() -> Navigation {
        Navigation::new(WINDOW, Controls::default())
    }

    fn modifiers(s: &str) -> Modifiers {
        s.parse().expect("modifiers")
    }

    #[test]
    fn left_button_pans_and_selects() {
        let mut navigation = navigation();
        let world = navigation.controller().world_point(CENTER);
        navigation.mouse_pressed(Left, Modifiers::NONE, CENTER);
        // Within the click tolerance, it selects.
        navigation.mouse_moved(Vec2::new(2.0, 0.0), FRAME);
        assert_eq!(navigation.controller().mode(), Mode::Inactive);
        // Further, it pans, also by the movement so far.
        navigation.mouse_moved(Vec2::new(10.0, 5.0), FRAME);
        assert_eq!(navigation.controller().mode(), Mode::Translation);
        navigation.mouse_moved(Vec2::new(-4.0, 3.0), FRAME);
        let moved = navigation.controller().screen_coord(world);
        assert!(
            moved.abs_diff_eq(CENTER + DVec2::new(8.0, 8.0), 1e-2),
            "{moved}"
        );
        assert_eq!(
            navigation.mouse_released(Left, Modifiers::NONE, CENTER),
            None
        );
        assert_eq!(navigation.controller().mode(), Mode::Inactive);

        // A click selects without panning.
        navigation.mouse_pressed(Left, Modifiers::NONE, CENTER);
        navigation.mouse_moved(Vec2::new(1.0, 1.0), FRAME);
        assert!(
            navigation
                .mouse_released(Left, Modifiers::NONE, CENTER)
                .is_some()
        );

        // Middle and then left still orbits at once.
        navigation.mouse_pressed(Middle, Modifiers::NONE, CENTER);
        navigation.mouse_pressed(Left, Modifiers::NONE, CENTER);
        assert_eq!(navigation.controller().mode(), Mode::Orbit);
        // Releasing the middle button pans with the left one, without selecting.
        navigation.mouse_released(Middle, Modifiers::NONE, CENTER);
        assert_eq!(navigation.controller().mode(), Mode::Translation);
        assert_eq!(
            navigation.mouse_released(Left, Modifiers::NONE, CENTER),
            None
        );
    }

    #[test]
    fn click_selects() {
        let mut navigation = navigation();
        navigation.mouse_pressed(Left, Modifiers::NONE, CENTER);
        assert_eq!(
            navigation.mouse_released(Left, Modifiers::NONE, CENTER),
            Some(Command::Select {
                position: CENTER,
                multi: false
            })
        );

        navigation.mouse_pressed(Left, Modifiers::CTRL, CENTER);
        assert_eq!(
            navigation.mouse_released(Left, Modifiers::CTRL, CENTER),
            Some(Command::Select {
                position: CENTER,
                multi: true
            })
        );

        // Dragging is no click.
        navigation.mouse_pressed(Left, Modifiers::NONE, CENTER);
        navigation.mouse_moved(Vec2::new(10.0, 0.0), FRAME);
        assert_eq!(
            navigation.mouse_released(Left, Modifiers::NONE, CENTER),
            None
        );
    }

    #[test]
    fn buttons_in_order() {
        let mut navigation = navigation();
        let none = Modifiers::NONE;
        navigation.mouse_pressed(Middle, none, CENTER);
        assert_eq!(navigation.controller().mode(), Mode::Translation);
        navigation.mouse_pressed(Right, none, CENTER);
        assert_eq!(navigation.controller().mode(), Mode::Orbit);
        // Releasing the right button pans again, without selecting.
        assert_eq!(navigation.mouse_released(Right, none, CENTER), None);
        assert_eq!(navigation.controller().mode(), Mode::Translation);
        assert_eq!(navigation.mouse_released(Middle, none, CENTER), None);
        assert_eq!(navigation.controller().mode(), Mode::Inactive);

        // Middle and left orbit, but no click after it.
        navigation.mouse_pressed(Middle, none, CENTER);
        navigation.mouse_pressed(Left, none, CENTER);
        assert_eq!(navigation.controller().mode(), Mode::Orbit);
        navigation.mouse_released(Middle, none, CENTER);
        assert_eq!(navigation.mouse_released(Left, none, CENTER), None);

        // The right button and then the middle one are not bound.
        navigation.mouse_pressed(Right, none, CENTER);
        navigation.mouse_pressed(Middle, none, CENTER);
        assert_eq!(navigation.mouse_action(), None);
        navigation.mouse_released(Middle, none, CENTER);
        navigation.mouse_released(Right, none, CENTER);
    }

    #[test]
    fn modifiers_with_right() {
        let mut navigation = navigation();
        for (modifiers_name, mode) in [
            ("", Mode::Orbit),
            ("Ctrl", Mode::Translation),
            ("Ctrl+Shift", Mode::Zoom),
            ("Shift", Mode::Inactive),
        ] {
            let modifiers = modifiers(modifiers_name);
            navigation.mouse_pressed(Right, modifiers, CENTER);
            assert_eq!(navigation.controller().mode(), mode, "{modifiers_name}");
            navigation.mouse_released(Right, modifiers, CENTER);
            assert_eq!(navigation.controller().mode(), Mode::Inactive);
        }
    }

    #[test]
    fn drag() {
        let mut navigation = navigation();
        // Orbiting turns the view.
        let view = navigation.controller().camera().view_vector().normalize();
        navigation.mouse_pressed(Right, Modifiers::NONE, CENTER);
        navigation.mouse_moved(Vec2::new(50.0, 0.0), FRAME);
        navigation.mouse_released(Right, Modifiers::NONE, CENTER);
        let turned = navigation.controller().camera().view_vector().normalize();
        assert!(view.angle_between(turned) > 0.01);

        // Panning moves the view.
        let detail = navigation
            .controller()
            .ortho()
            .expect("Orthographic")
            .zoom_model()
            .detail();
        navigation.mouse_pressed(Middle, Modifiers::NONE, CENTER);
        navigation.mouse_moved(Vec2::new(10.0, 0.0), FRAME);
        navigation.mouse_released(Middle, Modifiers::NONE, CENTER);
        assert!(
            navigation
                .controller()
                .ortho()
                .expect("Orthographic")
                .zoom_model()
                .detail()
                .x
                < detail.x
        );

        // Dragging upwards zooms in.
        let zoom_level = navigation.controller().zoom_level();
        let ctrl_shift = modifiers("Ctrl+Shift");
        navigation.mouse_pressed(Right, ctrl_shift, CENTER);
        navigation.mouse_moved(Vec2::new(0.0, -50.0), FRAME);
        navigation.mouse_released(Right, ctrl_shift, CENTER);
        assert!(navigation.controller().zoom_level() > zoom_level);
    }

    #[test]
    fn wheel() {
        let mut navigation = navigation();
        assert!(navigation.wheel(100.0, Modifiers::NONE, CENTER, FRAME));
        assert!(navigation.controller().zoom_level() > 1.0);
        assert!(!navigation.wheel(100.0, Modifiers::SHIFT, CENTER, FRAME));
    }

    #[test]
    fn zoom_keys() {
        let mut navigation = navigation();
        assert!(navigation.key_pressed("PageUp", Modifiers::NONE));
        assert!((navigation.controller().zoom_level() - 1.25).abs() < 1e-9);
        // `+` needs Shift on some keyboards.
        assert!(navigation.key_pressed("+", modifiers("Ctrl+Shift")));
        assert!((navigation.controller().zoom_level() - 1.5625).abs() < 1e-9);
        assert!(navigation.key_pressed("-", Modifiers::CTRL));
        assert!(navigation.key_pressed("PageDown", Modifiers::NONE));
        assert!((navigation.controller().zoom_level() - 1.0).abs() < 1e-9);
        // Without Ctrl, `+` is not bound.
        assert!(!navigation.key_pressed("+", Modifiers::NONE));
        // `f` shows the whole scene.
        navigation.key_pressed("PageUp", Modifiers::NONE);
        navigation.key_pressed("ArrowLeft", Modifiers::NONE);
        assert!(navigation.key_pressed("f", Modifiers::NONE));
        let model = navigation
            .controller()
            .ortho()
            .expect("Orthographic")
            .zoom_model();
        assert_eq!(model.detail(), model.overview());
    }

    #[test]
    fn pan_keys() {
        let mut navigation = navigation();
        navigation.key_pressed("PageUp", Modifiers::NONE);
        let detail = navigation
            .controller()
            .ortho()
            .expect("Orthographic")
            .zoom_model()
            .detail();
        assert!(navigation.key_pressed("ArrowLeft", Modifiers::NONE));
        let moved = navigation
            .controller()
            .ortho()
            .expect("Orthographic")
            .zoom_model()
            .detail();
        // A tenth of the window.
        assert!((moved.x - (detail.x - 0.1 * detail.width)).abs() < 1e-3);
        assert!(navigation.key_pressed("ArrowUp", Modifiers::NONE));
        assert!(
            navigation
                .controller()
                .ortho()
                .expect("Orthographic")
                .zoom_model()
                .detail()
                .y
                > moved.y
        );
    }

    #[test]
    fn rotate_keys() {
        let mut navigation = navigation();
        let view = navigation.controller().camera().view_vector().normalize();
        assert!(navigation.key_pressed("ArrowLeft", Modifiers::SHIFT));
        let turned = navigation.controller().camera().view_vector().normalize();
        assert!((view.angle_between(turned).to_degrees() - 90.0).abs() < 1e-3);
        assert!(navigation.key_pressed("ArrowRight", Modifiers::SHIFT));
        let back = navigation.controller().camera().view_vector().normalize();
        assert!(back.abs_diff_eq(view, 1e-5));
    }

    #[test]
    fn view_keys() {
        let mut navigation = navigation();
        for (key, view) in [
            ("0", StandardView::Isometric),
            ("1", StandardView::Front),
            ("2", StandardView::Top),
            ("3", StandardView::Right),
            ("4", StandardView::Rear),
            ("5", StandardView::Bottom),
            ("6", StandardView::Left),
        ] {
            assert!(navigation.key_pressed(key, Modifiers::NONE), "{key}");
            let camera = navigation.controller().camera();
            let expected = -view.view_vector() * 1000.0;
            assert!(camera.position.abs_diff_eq(expected, 1e-2), "{key}");
            assert!(view.up().dot(view.view_vector()).abs() < 1e-6, "{key}");
        }
        assert!(!navigation.key_pressed("7", Modifiers::NONE));
    }

    #[test]
    fn toggle_projection() {
        let mut navigation = navigation();
        assert!(navigation.key_pressed("p", Modifiers::NONE));
        assert_eq!(
            navigation.controller().projection_mode(),
            ProjectionMode::Perspective
        );
        assert!(navigation.key_pressed("p", Modifiers::NONE));
        assert_eq!(
            navigation.controller().projection_mode(),
            ProjectionMode::Orthographic
        );

        let controls =
            Controls::from_toml("[camera]\nprojection = \"perspective\"\n").expect("TOML");
        let navigation = Navigation::new(WINDOW, controls);
        assert_eq!(
            navigation.controller().projection_mode(),
            ProjectionMode::Perspective
        );
    }

    #[test]
    fn configured() {
        let controls = Controls::from_toml(
            "[mouse]\norbit = [\"Middle+Right\"]\npan = [\"Right\"]\n\
                 [keyboard]\nview_top = [\"t\"]\n[touch]\none_finger = \"pan\"\n",
        )
        .expect("controls");
        let mut navigation = Navigation::new(WINDOW, controls);
        navigation.mouse_pressed(Right, Modifiers::NONE, CENTER);
        assert_eq!(navigation.controller().mode(), Mode::Translation);
        navigation.mouse_released(Right, Modifiers::NONE, CENTER);
        assert!(navigation.key_pressed("T", Modifiers::SHIFT));
        assert!(!navigation.key_pressed("2", Modifiers::NONE));
        navigation.touch_started(1, CENTER);
        assert_eq!(navigation.controller().mode(), Mode::Translation);
    }

    #[test]
    fn tap_selects() {
        let mut navigation = navigation();
        navigation.touch_started(7, CENTER);
        assert_eq!(navigation.controller().mode(), Mode::Orbit);
        assert_eq!(
            navigation.touch_ended(7, CENTER),
            Some(Command::Select {
                position: CENTER,
                multi: false
            })
        );
        assert_eq!(navigation.controller().mode(), Mode::Inactive);

        // Dragging is no tap.
        navigation.touch_started(7, CENTER);
        navigation.touches_moved(&[(7, CENTER + DVec2::new(10.0, 0.0))], FRAME);
        assert_eq!(navigation.touch_ended(7, CENTER), None);

        // Two fingers are no tap.
        navigation.touch_started(1, CENTER);
        navigation.touch_started(2, CENTER + DVec2::X);
        navigation.touch_ended(2, CENTER);
        assert_eq!(navigation.touch_ended(1, CENTER), None);
    }

    #[test]
    fn one_finger_orbits() {
        let mut navigation = navigation();
        let view = navigation.controller().camera().view_vector().normalize();
        navigation.touch_started(1, CENTER);
        navigation.touches_moved(&[(1, CENTER + DVec2::new(50.0, 0.0))], FRAME);
        navigation.touch_ended(1, CENTER);
        let turned = navigation.controller().camera().view_vector().normalize();
        assert!(view.angle_between(turned) > 0.01);
    }

    #[test]
    fn pinch_zooms() {
        let mut navigation = navigation();
        let left = DVec2::new(150.0, 100.0);
        let right = DVec2::new(250.0, 100.0);
        navigation.touch_started(1, left);
        navigation.touch_started(2, right);
        assert_eq!(navigation.controller().mode(), Mode::Translation);
        let center = DVec2::new(200.0, 100.0);
        let world = navigation.controller().world_point(center);
        // Spreading the fingers from 100 to 300 pixels zooms in three times.
        navigation.touches_moved(
            &[(1, DVec2::new(50.0, 100.0)), (2, DVec2::new(350.0, 100.0))],
            FRAME,
        );
        assert!((navigation.controller().zoom_level() - 3.0).abs() < 1e-6);
        // The point between the fingers stays between them.
        let moved = navigation.controller().screen_coord(world);
        assert!(moved.abs_diff_eq(center, 1e-2), "{moved}");

        // Releasing one finger orbits with the other.
        navigation.touch_ended(2, DVec2::new(350.0, 100.0));
        assert_eq!(navigation.controller().mode(), Mode::Orbit);
        navigation.touch_cancelled(1);
        assert_eq!(navigation.controller().mode(), Mode::Inactive);
    }

    #[test]
    fn two_fingers_pan() {
        let mut navigation = navigation();
        navigation.key_pressed("PageUp", Modifiers::NONE);
        let point = DVec2::new(200.0, 100.0);
        let world = navigation.controller().world_point(point);
        navigation.touch_started(1, DVec2::new(150.0, 100.0));
        navigation.touch_started(2, DVec2::new(250.0, 100.0));
        // Both fingers move by (20, 10), the scene follows their center.
        navigation.touches_moved(
            &[(1, DVec2::new(170.0, 110.0)), (2, DVec2::new(270.0, 110.0))],
            FRAME,
        );
        let moved = navigation.controller().screen_coord(world);
        assert!(
            moved.abs_diff_eq(point + DVec2::new(20.0, 10.0), 1e-2),
            "{moved}"
        );
        assert!((navigation.controller().zoom_level() - 1.25).abs() < 1e-6);
    }
}
