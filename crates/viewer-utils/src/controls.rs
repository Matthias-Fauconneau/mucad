// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The configuration of the mouse, the keys and touch to control the camera, e.g. from TOML.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{
    CameraControllerConfig, KeyBinding, Modifiers, MouseBinding, MouseButton, MouseCommand,
    StandardView,
};

/// The controls of the camera in the µcad viewers with the mouse, the keys and touch.
///
/// Missing entries are the defaults, e.g. a file with only `[mouse]` and `pan = ["Middle"]`
/// changes only panning with the mouse.
///
/// A binding is its modifiers and its input joined by `+`, e.g. `Ctrl+Shift+Right`.
/// Modifiers: `Ctrl`, `Shift`, `Alt` (`Option`), `Meta` (`Cmd`, `Super`), which must be pressed
/// exactly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Controls {
    /// The factor, which a zoom key scales the view by, e.g. 1.25.
    pub zoom_step: f64,
    /// The part of the window, which a pan key moves the view by, e.g. 0.1.
    pub pan_step: f64,
    /// The angle in degrees, which a rotate key turns the view by, e.g. 90.
    pub rotate_step: f64,
    /// The speeds of the camera with the mouse.
    pub camera: CameraControllerConfig,
    /// The navigation style with the mouse.
    pub mouse: MouseControls,
    /// The keys, which are available with any navigation style.
    pub keyboard: KeyboardControls,
    /// The gestures of touch screens.
    pub touch: TouchControls,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            zoom_step: 1.25,
            pan_step: 0.1,
            rotate_step: 90.0,
            camera: CameraControllerConfig::default(),
            mouse: MouseControls::default(),
            keyboard: KeyboardControls::default(),
            touch: TouchControls::default(),
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
/// Buttons: `Left`, `Middle` (pressing the wheel) and `Right`, which are pressed in their
/// order, e.g. `Middle+Right` is first the middle and then the right button. `Wheel` is turning
/// the wheel, which only zooms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MouseControls {
    /// Select an object by clicking.
    ///
    /// A binding, which also drags, e.g. `Left` of `pan`, selects by clicking and drags, when
    /// the mouse moves more than a few pixels.
    pub select: Vec<MouseBinding>,
    /// Zoom with the wheel at the pointer, or by dragging upwards (in) and downwards (out).
    pub zoom: Vec<MouseBinding>,
    /// Orbit around the point under the pointer by dragging.
    pub orbit: Vec<MouseBinding>,
    /// Pan the view by dragging.
    pub pan: Vec<MouseBinding>,
}

impl MouseControls {
    /// The bindings of `action`.
    pub fn bindings(&self, action: MouseCommand) -> &[MouseBinding] {
        match action {
            MouseCommand::Select { .. } => &self.select,
            MouseCommand::Orbit => &self.orbit,
            MouseCommand::Pan => &self.pan,
            MouseCommand::Zoom => &self.zoom,
        }
    }
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
                MouseBinding::buttons(none, &[Right]),
            ],
            pan: vec![
                MouseBinding::buttons(none, &[Left]),
                MouseBinding::buttons(none, &[Middle]),
                MouseBinding::buttons(ctrl, &[Right]),
            ],
        }
    }
}

/// A command of the keys, written in snake case in TOML, e.g. `zoom_in`.
///
/// It is an [`Action`](crate::Action): its message is the label, its doc comment the description and its
/// property `group` the group in the help.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumIter,
    strum::EnumMessage,
    strum::EnumProperty,
    strum::IntoStaticStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum KeyCommand {
    /// Zoom in at the center of the view.
    #[strum(message = "Zoom in", props(group = "Zoom"))]
    ZoomIn,
    /// Zoom out at the center of the view.
    #[strum(message = "Zoom out", props(group = "Zoom"))]
    ZoomOut,
    /// Show the whole scene in the middle of the view.
    #[strum(message = "Zoom to fit")]
    ZoomFit,
    /// Change between the orthographic and the perspective projection.
    #[strum(message = "Toggle projection")]
    ToggleProjection,
    /// Pan the view to the left, i.e. the model moves to the right.
    #[strum(message = "Pan left", props(group = "Pan"))]
    PanLeft,
    /// Pan the view to the right, i.e. the model moves to the left.
    #[strum(message = "Pan right", props(group = "Pan"))]
    PanRight,
    /// Pan the view upwards, i.e. the model moves downwards.
    #[strum(message = "Pan up", props(group = "Pan"))]
    PanUp,
    /// Pan the view downwards, i.e. the model moves upwards.
    #[strum(message = "Pan down", props(group = "Pan"))]
    PanDown,
    /// Rotate the view to the left around the vertical axis of the view.
    #[strum(message = "Rotate left", props(group = "Rotate"))]
    RotateLeft,
    /// Rotate the view to the right around the vertical axis of the view.
    #[strum(message = "Rotate right", props(group = "Rotate"))]
    RotateRight,
    /// Show the isometric view.
    #[strum(message = "Isometric view", props(group = "Views"))]
    ViewIsometric,
    /// Show the front view.
    #[strum(message = "Front view", props(group = "Views"))]
    ViewFront,
    /// Show the top view.
    #[strum(message = "Top view", props(group = "Views"))]
    ViewTop,
    /// Show the right view.
    #[strum(message = "Right view", props(group = "Views"))]
    ViewRight,
    /// Show the rear view.
    #[strum(message = "Rear view", props(group = "Views"))]
    ViewRear,
    /// Show the bottom view.
    #[strum(message = "Bottom view", props(group = "Views"))]
    ViewBottom,
    /// Show the left view.
    #[strum(message = "Left view", props(group = "Views"))]
    ViewLeft,
}

impl KeyCommand {
    /// The standard view, which the command shows, if it is a view command.
    pub(crate) fn view(self) -> Option<StandardView> {
        use KeyCommand::*;
        match self {
            ViewIsometric => Some(StandardView::Isometric),
            ViewFront => Some(StandardView::Front),
            ViewTop => Some(StandardView::Top),
            ViewRight => Some(StandardView::Right),
            ViewRear => Some(StandardView::Rear),
            ViewBottom => Some(StandardView::Bottom),
            ViewLeft => Some(StandardView::Left),
            _ => None,
        }
    }

    /// The default bindings of the command.
    pub fn default_bindings(self) -> Vec<KeyBinding> {
        use KeyCommand::*;
        let (ctrl, shift, none) = (Modifiers::CTRL, Modifiers::SHIFT, Modifiers::NONE);
        let bindings: &[(Modifiers, &str)] = match self {
            ZoomIn => &[(ctrl, "+"), (none, "PageUp")],
            ZoomOut => &[(ctrl, "-"), (none, "PageDown")],
            ZoomFit => &[(none, "f")],
            ToggleProjection => &[(none, "p")],
            PanLeft => &[(none, "ArrowLeft")],
            PanRight => &[(none, "ArrowRight")],
            PanUp => &[(none, "ArrowUp")],
            PanDown => &[(none, "ArrowDown")],
            RotateLeft => &[(shift, "ArrowLeft")],
            RotateRight => &[(shift, "ArrowRight")],
            ViewIsometric => &[(none, "0")],
            ViewFront => &[(none, "1")],
            ViewTop => &[(none, "2")],
            ViewRight => &[(none, "3")],
            ViewRear => &[(none, "4")],
            ViewBottom => &[(none, "5")],
            ViewLeft => &[(none, "6")],
        };
        bindings
            .iter()
            .map(|(modifiers, key)| KeyBinding::new(*modifiers, key))
            .collect()
    }
}

/// The bindings of the keys, which are available with any navigation style.
///
/// Keys are logical keys, like `KeyboardEvent.key` of the web: named keys like `ArrowLeft`,
/// `PageUp`, or characters like `0` or `a`, with `Plus`, `Minus` and `Space` for `+`, `-` and
/// ` `. Shift is ignored for characters, because keyboard layouts need it for some of them.
///
/// In TOML, the bindings of each command are an entry, e.g. `zoom_in = ["Ctrl+Plus"]`, the
/// missing commands have their defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "PartialKeyboardControls")]
pub struct KeyboardControls {
    /// The bindings of the commands, all commands have an entry.
    #[serde(flatten)]
    pub commands: BTreeMap<KeyCommand, Vec<KeyBinding>>,
    /// The modifiers, which select more than one object, when they are pressed while selecting.
    pub multi_select: Modifiers,
}

impl KeyboardControls {
    /// The bindings of `command`.
    pub fn bindings(&self, command: KeyCommand) -> &[KeyBinding] {
        self.commands.get(&command).map_or(&[], Vec::as_slice)
    }

    /// The command of `key` pressed with `modifiers`, the first one in the order of
    /// [`KeyCommand`], if it is bound.
    pub fn command(&self, key: &str, modifiers: Modifiers) -> Option<KeyCommand> {
        self.commands.iter().find_map(|(command, bindings)| {
            bindings
                .iter()
                .any(|binding| binding.matches(key, modifiers))
                .then_some(*command)
        })
    }
}

impl Default for KeyboardControls {
    fn default() -> Self {
        Self {
            commands: KeyCommand::iter()
                .map(|command| (command, command.default_bindings()))
                .collect(),
            multi_select: Modifiers::CTRL,
        }
    }
}

/// The keyboard controls in TOML, whose missing entries are the defaults.
#[derive(Deserialize)]
struct PartialKeyboardControls {
    #[serde(flatten)]
    commands: BTreeMap<KeyCommand, Vec<KeyBinding>>,
    multi_select: Option<Modifiers>,
}

impl From<PartialKeyboardControls> for KeyboardControls {
    fn from(partial: PartialKeyboardControls) -> Self {
        let mut controls = Self::default();
        controls.commands.extend(partial.commands);
        if let Some(multi_select) = partial.multi_select {
            controls.multi_select = multi_select;
        }
        controls
    }
}

/// What dragging with one finger does.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TouchDrag {
    /// Orbit around the point under the finger.
    #[default]
    Orbit,
    /// Pan the view.
    Pan,
}

/// The gestures of touch screens: two fingers always pinch to zoom and pan, a tap selects.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TouchControls {
    /// What dragging with one finger does: `orbit` or `pan`.
    pub one_finger: TouchDrag,
}

#[cfg(test)]
mod tests {
    use super::*;

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
            "[camera]\nzoom_speed = -25.0\n[mouse]\npan = [\"Middle\"]\n[keyboard]\nmulti_select = \"Shift\"\n",
        )
        .expect("TOML");
        assert_eq!(controls.camera.zoom_speed, -25.0);
        assert_eq!(controls.camera.orbit_speed, 25.0);
        assert_eq!(controls.mouse.pan, ["Middle".parse().expect("binding")]);
        assert_eq!(controls.mouse.orbit, MouseControls::default().orbit);
        assert_eq!(controls.keyboard.multi_select, Modifiers::SHIFT);
        assert_eq!(controls.touch, TouchControls::default());

        // A command replaces only its own bindings.
        let controls = Controls::from_toml("[keyboard]\nzoom_in = [\"i\"]\n").expect("TOML");
        let keyboard = &controls.keyboard;
        assert_eq!(
            keyboard.bindings(KeyCommand::ZoomIn),
            ["i".parse().expect("binding")]
        );
        assert_eq!(
            keyboard.command("PageDown", Modifiers::NONE),
            Some(KeyCommand::ZoomOut)
        );
        assert_eq!(keyboard.multi_select, Modifiers::CTRL);

        let controls = Controls::from_toml("[touch]\none_finger = \"pan\"\n").expect("TOML");
        assert_eq!(controls.touch.one_finger, TouchDrag::Pan);
    }

    #[test]
    fn errors() {
        assert!(Controls::from_toml("[mouse]\npan = [\"Middle+Up\"]\n").is_err());
        assert!(Controls::from_toml("[mouse]\nspin = [\"Middle\"]\n").is_err());
        assert!(Controls::from_toml("[touch]\none_finger = \"spin\"\n").is_err());
        assert!(Controls::from_toml("[keyboard]\nzoom_sideways = [\"x\"]\n").is_err());
    }
}
