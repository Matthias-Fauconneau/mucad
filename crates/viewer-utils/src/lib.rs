// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Utilities shared by the µcad viewers, e.g. the Bevy viewer and the web viewer.
//!
//! The utilities are independent of a renderer, they compute with [`glam`] vectors and
//! matrices, which a viewer applies to its camera.

mod camera;
mod camera_controller;
mod controls;
mod input;
mod navigation;
mod rect;
mod zoom_rect;

pub use camera::{Camera, ProjectionType};
pub use camera_controller::{CameraController, CameraEvent, CameraState, Mode, Ray};
pub use controls::{Controls, KeyboardControls, MouseControls};
pub use input::{KeyBinding, Modifiers, MouseBinding, MouseButton, MouseInput, ParseError};
pub use navigation::{Command, MouseAction, Navigation, StandardView};
pub use rect::Rect;
pub use zoom_rect::{ZoomEvent, ZoomRectModel, ZoomState, zoomed_rect};
