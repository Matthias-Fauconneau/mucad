// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The shaders of the µcad viewers in WGSL, e.g. for Bevy, and converted to GLSL for WebGL.

/// The grid library with `#define_import_path mucad_viewer_utils::grid`: `GridConfig` and
/// `grid_color()`, without bindings, which the viewers import.
pub const GRID_WGSL: &str = include_str!("../shaders/grid.wgsl");

/// The grid for WebGL2, e.g. three.js, with its vertex and fragment shader, which imports
/// [`GRID_WGSL`] and which `cargo xtask web` converts to GLSL.
pub const GRID_WEB_WGSL: &str = include_str!("../shaders/grid_web.wgsl");
