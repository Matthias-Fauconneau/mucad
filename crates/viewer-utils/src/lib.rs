// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Utilities shared by the µcad viewers, e.g. the Bevy viewer and the web viewer.
//!
//! The utilities are independent of a renderer, they compute with [`glam`] vectors and
//! matrices, which a viewer applies to its camera.

/// A scalar like in µcad, e.g. a length in millimeters.
pub type Scalar = f64;

mod bounding_sphere;
mod camera;
mod camera_controller;
mod controls;
mod edges;
mod grid;
mod input;
mod lighting;
mod navigation;
mod rect;
pub mod shaders;
mod theme;
mod zoom_rect;

pub use bounding_sphere::BoundingSphere;
pub use camera::{Camera, ProjectionType};
pub use camera_controller::{
    CameraControl, CameraController, CameraControllerConfig, CameraControllerState, CameraEvent,
    Mode, OrthoCameraController, PerspectiveCameraController, ProjectionMode, Ray,
};
pub use controls::{
    Controls, DEFAULT_CONTROLS_TOML, KeyboardControls, MouseControls, TouchControls, TouchDrag,
};
pub use edges::{CREASE_ANGLE_DEGREES, EdgeLines, edge_lines};
pub use grid::GridConfig;
pub use input::{KeyBinding, Modifiers, MouseBinding, MouseButton, MouseInput, ParseError};
pub use lighting::Lighting;
pub use navigation::{Command, MouseAction, Navigation, StandardView, TouchId};
pub use rect::Rect;
pub use theme::Theme;
pub use zoom_rect::{ZoomEvent, ZoomRectModel, ZoomState, zoomed_rect};
