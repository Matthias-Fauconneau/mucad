// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The configuration of the mouse, the keys and touch to control the camera, e.g. from TOML.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{CameraControllerConfig, Modifiers, StandardView};

/// The controls of the camera in the µcad viewers with the mouse, the keys and touch.
///
/// Missing entries are the defaults, e.g. a file with only `zoom_step = 2.0` changes only the
/// step of the zoom keys.
///
/// The `[bindings]` are the bindings of the actions by their ids, e.g.
/// `orbit = ["Middle+Right"]` or `zoom_fit = ["Home"]`, see [`Binding`](crate::Binding) for
/// their syntax. They replace the bindings of the actions in an
/// [`ActionRegistry`](crate::ActionRegistry) with
/// [`set_bindings`](crate::ActionRegistry::set_bindings), the other actions keep their
/// defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Controls {
    /// The factor, which a zoom key scales the view by, e.g. 1.25.
    pub zoom_step: f64,
    /// The part of the window, which a pan key moves the view by, e.g. 0.1.
    pub pan_step: f64,
    /// The angle in degrees, which a rotate key turns the view by, e.g. 90.
    pub rotate_step: f64,
    /// The modifiers, which select more than one object, when they are pressed while selecting.
    pub multi_select: Modifiers,
    /// The speeds of the camera with the mouse.
    pub camera: CameraControllerConfig,
    /// The gestures of touch screens.
    pub touch: TouchControls,
    /// The bindings of the actions by their ids, which replace their defaults.
    pub bindings: BTreeMap<String, Vec<String>>,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            zoom_step: 1.25,
            pan_step: 0.1,
            rotate_step: 90.0,
            multi_select: Modifiers::CTRL,
            camera: CameraControllerConfig::default(),
            touch: TouchControls::default(),
            bindings: BTreeMap::new(),
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

/// A command of the keys, whose id is its name in snake case, e.g. `zoom_in`.
///
/// It is an [`Action`](crate::Action): its message is the label, its doc comment the
/// description and its property `group` the group in the help.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum::Display,
    strum::EnumIter,
    strum::EnumMessage,
    strum::EnumProperty,
    strum::IntoStaticStr,
)]
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
        let mut controls = Controls::default();
        controls
            .bindings
            .insert("zoom_fit".into(), vec!["Home".into()]);
        let toml = controls.to_toml().expect("TOML");
        assert_eq!(Controls::from_toml(&toml).expect("TOML"), controls);
    }

    #[test]
    fn partial() {
        let controls = Controls::from_toml(
            "multi_select = \"Shift\"\n[camera]\nzoom_speed = -25.0\n[bindings]\npan = [\"Middle\"]\n",
        )
        .expect("TOML");
        assert_eq!(controls.camera.zoom_speed, -25.0);
        assert_eq!(controls.camera.orbit_speed, 25.0);
        assert_eq!(controls.multi_select, Modifiers::SHIFT);
        assert_eq!(controls.bindings["pan"], ["Middle"]);
        assert_eq!(controls.touch, TouchControls::default());
        assert_eq!(controls.zoom_step, Controls::default().zoom_step);

        let controls = Controls::from_toml("[touch]\none_finger = \"pan\"\n").expect("TOML");
        assert_eq!(controls.touch.one_finger, TouchDrag::Pan);
    }

    #[test]
    fn errors() {
        assert!(Controls::from_toml("[touch]\none_finger = \"spin\"\n").is_err());
        assert!(Controls::from_toml("zoom_stepp = 2.0\n").is_err());
        // The bindings are the `[bindings]` of the actions.
        assert!(Controls::from_toml("[mouse]\npan = [\"Middle\"]\n").is_err());
    }
}
