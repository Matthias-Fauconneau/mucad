// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The navigation with the mouse and the keys: input events of a viewer control the camera
//! by the configured [`Controls`].

use std::time::Duration;

use glam::{DVec2, Vec2, Vec3};

use crate::{CameraController, Controls, KeyBinding, Modifiers, MouseButton, MouseInput};

/// The distance in pixels, which the mouse can move while clicking.
const CLICK_TOLERANCE: f32 = 4.0;

/// A standard view of the scene with Z upwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// The action of the pressed mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// The navigation of a [`CameraController`] with the mouse and the keys.
///
/// The viewer passes its input events, the modifiers and the pointer position in pixels from
/// the top left of the window, and applies the camera of the controller after them. While
/// [`CameraController::pointer_pinned`], it passes the movement of the mouse, e.g. of a locked
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
        };
        navigation.set_controls(controls);
        navigation
    }

    /// The camera controller.
    pub fn controller(&self) -> &CameraController {
        &self.controller
    }

    /// The camera controller, e.g. to set the scene.
    pub fn controller_mut(&mut self) -> &mut CameraController {
        &mut self.controller
    }

    /// The controls.
    pub fn controls(&self) -> &Controls {
        &self.controls
    }

    /// Set the controls, e.g. after the configuration is changed.
    pub fn set_controls(&mut self, controls: Controls) {
        self.controller.set_orbit_speed(controls.orbit_speed);
        self.controller.set_zoom_speed(controls.zoom_speed);
        self.controller
            .set_translate_speed(controls.translate_speed);
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
    pub fn mouse_moved(&mut self, delta: Vec2, elapsed: Duration) {
        if !self.pressed.is_empty() {
            self.moved += delta.length();
        }
        match self.action {
            Some(MouseAction::Orbit) => self.controller.orbit_by_mouse(delta, elapsed),
            Some(MouseAction::Pan) => self.controller.translate_by_mouse(delta),
            // Dragging upwards zooms in, like turning the wheel away.
            Some(MouseAction::Zoom) => self
                .controller
                .zoom_by_wheel(Vec2::new(0.0, -delta.y), elapsed),
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
    /// move the view center back to the scene center like after orbiting with the mouse.
    fn rotate(&mut self, pan: f64) {
        self.controller.orbit(pan, 0.0);
        self.controller.readjust_camera_position_and_detail();
    }

    /// Start the action of the pressed buttons, and end the previous one.
    fn update_action(&mut self, modifiers: Modifiers, position: DVec2) {
        let action = self.matching_action(modifiers);
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

    /// The action, whose binding are the pressed buttons with `modifiers`.
    fn matching_action(&self, modifiers: Modifiers) -> Option<MouseAction> {
        if self.pressed.is_empty() {
            return None;
        }
        let mouse = &self.controls.mouse;
        let buttons = MouseInput::Buttons(self.pressed.clone());
        let drags = [
            (&mouse.orbit, MouseAction::Orbit),
            (&mouse.pan, MouseAction::Pan),
            (&mouse.zoom, MouseAction::Zoom),
        ];
        for (bindings, action) in drags {
            if bindings
                .iter()
                .any(|binding| binding.input == buttons && binding.modifiers == modifiers)
            {
                return Some(action);
            }
        }
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
    use crate::Mode;
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

        // The right button alone is not bound.
        navigation.mouse_pressed(Right, none, CENTER);
        assert_eq!(navigation.mouse_action(), None);
        navigation.mouse_released(Right, none, CENTER);
    }

    #[test]
    fn modifiers_with_right() {
        let mut navigation = navigation();
        for (modifiers_name, mode) in [
            ("Shift", Mode::Orbit),
            ("Ctrl", Mode::Translation),
            ("Ctrl+Shift", Mode::Zoom),
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
        navigation.mouse_pressed(Right, Modifiers::SHIFT, CENTER);
        navigation.mouse_moved(Vec2::new(50.0, 0.0), FRAME);
        navigation.mouse_released(Right, Modifiers::SHIFT, CENTER);
        let turned = navigation.controller().camera().view_vector().normalize();
        assert!(view.angle_between(turned) > 0.01);

        // Panning moves the view.
        let detail = navigation.controller().zoom_model().detail();
        navigation.mouse_pressed(Middle, Modifiers::NONE, CENTER);
        navigation.mouse_moved(Vec2::new(10.0, 0.0), FRAME);
        navigation.mouse_released(Middle, Modifiers::NONE, CENTER);
        assert!(navigation.controller().zoom_model().detail().x < detail.x);

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
    }

    #[test]
    fn pan_keys() {
        let mut navigation = navigation();
        navigation.key_pressed("PageUp", Modifiers::NONE);
        let detail = navigation.controller().zoom_model().detail();
        assert!(navigation.key_pressed("ArrowLeft", Modifiers::NONE));
        let moved = navigation.controller().zoom_model().detail();
        // A tenth of the window.
        assert!((moved.x - (detail.x - 0.1 * detail.width)).abs() < 1e-3);
        assert!(navigation.key_pressed("ArrowUp", Modifiers::NONE));
        assert!(navigation.controller().zoom_model().detail().y > moved.y);
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
            assert!(camera.position().abs_diff_eq(expected, 1e-2), "{key}");
            assert!(view.up().dot(view.view_vector()).abs() < 1e-6, "{key}");
        }
        assert!(!navigation.key_pressed("7", Modifiers::NONE));
    }

    #[test]
    fn configured() {
        let controls =
            Controls::from_toml("[mouse]\npan = [\"Right\"]\n[keyboard]\nview_top = [\"t\"]\n")
                .expect("controls");
        let mut navigation = Navigation::new(WINDOW, controls);
        navigation.mouse_pressed(Right, Modifiers::NONE, CENTER);
        assert_eq!(navigation.controller().mode(), Mode::Translation);
        navigation.mouse_released(Right, Modifiers::NONE, CENTER);
        assert!(navigation.key_pressed("T", Modifiers::SHIFT));
        assert!(!navigation.key_pressed("2", Modifiers::NONE));
    }
}
