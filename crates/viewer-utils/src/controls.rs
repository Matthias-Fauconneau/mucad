// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The configuration of the mouse and the keys to control the camera, e.g. from TOML.

use serde::{Deserialize, Serialize};

use crate::{KeyBinding, Modifiers, MouseBinding, MouseButton};

/// The controls of the camera with the mouse and the keys and their speeds.
///
/// It is read from TOML like `controls.toml` of this crate, which has the defaults. Missing
/// entries are the defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Controls {
    /// The speed of orbiting with the mouse.
    pub orbit_speed: f64,
    /// The speed of zooming with the mouse, a negative speed inverts zooming.
    pub zoom_speed: f64,
    /// The speed of translating with the mouse.
    pub translate_speed: f64,
    /// The factor, which a zoom key scales the view by, e.g. 1.25.
    pub zoom_step: f64,
    /// The part of the window, which a pan key moves the view by, e.g. 0.1.
    pub pan_step: f64,
    /// The angle in degrees, which a rotate key turns the view by, e.g. 90.
    pub rotate_step: f64,
    /// The navigation style with the mouse.
    pub mouse: MouseControls,
    /// The keys, which are available with any navigation style.
    pub keyboard: KeyboardControls,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            orbit_speed: 25.0,
            zoom_speed: 25.0,
            translate_speed: 15.0,
            zoom_step: 1.25,
            pan_step: 0.1,
            rotate_step: 90.0,
            mouse: MouseControls::default(),
            keyboard: KeyboardControls::default(),
        }
    }
}

impl Controls {
    /// The controls in TOML, missing entries are the defaults.
    pub fn from_toml(toml: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml)
    }

    /// The controls as TOML.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }
}

/// The navigation style with the mouse: the bindings of the actions.
///
/// Dragging with buttons orbits, pans or zooms, clicking selects. Only zooming is bound to
/// the wheel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MouseControls {
    /// Select an object by clicking.
    pub select: Vec<MouseBinding>,
    /// Zoom with the wheel at the pointer, or by dragging upwards (in) and downwards (out).
    pub zoom: Vec<MouseBinding>,
    /// Orbit around the point under the pointer by dragging.
    pub orbit: Vec<MouseBinding>,
    /// Pan the view by dragging.
    pub pan: Vec<MouseBinding>,
}

impl Default for MouseControls {
    fn default() -> Self {
        use MouseButton::*;
        let none = Modifiers::NONE;
        let ctrl = Modifiers::CTRL;
        let shift = Modifiers::SHIFT;
        Self {
            select: vec![MouseBinding::buttons(none, &[Left])],
            zoom: vec![
                MouseBinding::wheel(none),
                MouseBinding::buttons(ctrl.union(shift), &[Right]),
            ],
            orbit: vec![
                MouseBinding::buttons(none, &[Middle, Right]),
                MouseBinding::buttons(none, &[Middle, Left]),
                MouseBinding::buttons(shift, &[Right]),
            ],
            pan: vec![
                MouseBinding::buttons(none, &[Middle]),
                MouseBinding::buttons(ctrl, &[Right]),
            ],
        }
    }
}

/// The bindings of the keys, which are available with any navigation style.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct KeyboardControls {
    /// Zoom in at the center of the view.
    pub zoom_in: Vec<KeyBinding>,
    /// Zoom out at the center of the view.
    pub zoom_out: Vec<KeyBinding>,
    /// Pan the view to the left, i.e. the model moves to the right.
    pub pan_left: Vec<KeyBinding>,
    /// Pan the view to the right, i.e. the model moves to the left.
    pub pan_right: Vec<KeyBinding>,
    /// Pan the view upwards, i.e. the model moves downwards.
    pub pan_up: Vec<KeyBinding>,
    /// Pan the view downwards, i.e. the model moves upwards.
    pub pan_down: Vec<KeyBinding>,
    /// Rotate the view to the left around the vertical axis of the view.
    pub rotate_left: Vec<KeyBinding>,
    /// Rotate the view to the right around the vertical axis of the view.
    pub rotate_right: Vec<KeyBinding>,
    /// Show the isometric view.
    pub view_isometric: Vec<KeyBinding>,
    /// Show the front view.
    pub view_front: Vec<KeyBinding>,
    /// Show the top view.
    pub view_top: Vec<KeyBinding>,
    /// Show the right view.
    pub view_right: Vec<KeyBinding>,
    /// Show the rear view.
    pub view_rear: Vec<KeyBinding>,
    /// Show the bottom view.
    pub view_bottom: Vec<KeyBinding>,
    /// Show the left view.
    pub view_left: Vec<KeyBinding>,
    /// The modifiers, which select more than one object, when they are pressed while selecting.
    pub multi_select: Modifiers,
}

impl Default for KeyboardControls {
    fn default() -> Self {
        let none = Modifiers::NONE;
        let keys = |bindings: &[(Modifiers, &str)]| -> Vec<KeyBinding> {
            bindings
                .iter()
                .map(|(modifiers, key)| KeyBinding::new(*modifiers, key))
                .collect()
        };
        Self {
            zoom_in: keys(&[(Modifiers::CTRL, "+"), (none, "PageUp")]),
            zoom_out: keys(&[(Modifiers::CTRL, "-"), (none, "PageDown")]),
            pan_left: keys(&[(none, "ArrowLeft")]),
            pan_right: keys(&[(none, "ArrowRight")]),
            pan_up: keys(&[(none, "ArrowUp")]),
            pan_down: keys(&[(none, "ArrowDown")]),
            rotate_left: keys(&[(Modifiers::SHIFT, "ArrowLeft")]),
            rotate_right: keys(&[(Modifiers::SHIFT, "ArrowRight")]),
            view_isometric: keys(&[(none, "0")]),
            view_front: keys(&[(none, "1")]),
            view_top: keys(&[(none, "2")]),
            view_right: keys(&[(none, "3")]),
            view_rear: keys(&[(none, "4")]),
            view_bottom: keys(&[(none, "5")]),
            view_left: keys(&[(none, "6")]),
            multi_select: Modifiers::CTRL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The documented defaults.
    const CONTROLS_TOML: &str = include_str!("../controls.toml");

    #[test]
    fn defaults_are_documented() {
        let controls = Controls::from_toml(CONTROLS_TOML).expect("controls.toml");
        assert_eq!(controls, Controls::default());
    }

    #[test]
    fn round_trip() {
        let toml = Controls::default().to_toml().expect("TOML");
        assert_eq!(
            Controls::from_toml(&toml).expect("TOML"),
            Controls::default()
        );
    }

    #[test]
    fn partial() {
        let controls = Controls::from_toml(
            "zoom_speed = -25.0\n[mouse]\npan = [\"Middle\"]\n[keyboard]\nmulti_select = \"Shift\"\n",
        )
        .expect("TOML");
        assert_eq!(controls.zoom_speed, -25.0);
        assert_eq!(controls.mouse.pan, ["Middle".parse().expect("binding")]);
        assert_eq!(controls.mouse.orbit, MouseControls::default().orbit);
        assert_eq!(controls.keyboard.multi_select, Modifiers::SHIFT);
    }

    #[test]
    fn errors() {
        assert!(Controls::from_toml("[mouse]\npan = [\"Middle+Up\"]\n").is_err());
        assert!(Controls::from_toml("[mouse]\nspin = [\"Middle\"]\n").is_err());
    }
}
