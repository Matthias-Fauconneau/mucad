// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The shaders of the µcad viewers in WGSL, e.g. for Bevy, and converted to GLSL for WebGL.

/// The grid library with `#define_import_path mucad_viewer_utils::grid`: `GridConfig` and
/// `grid_color()`, without bindings, which the viewers import.
pub const GRID_WGSL: &str = include_str!("../shaders/grid.wgsl");

/// The lighting library with `#define_import_path mucad_viewer_utils::lighting`: `Lighting`
/// and `shade()`, without bindings, which the viewers import for the models.
pub const LIGHTING_WGSL: &str = include_str!("../shaders/lighting.wgsl");

/// The silhouette library with `#define_import_path mucad_viewer_utils::silhouette`:
/// `is_silhouette()`, `to_viewer()`, `in_front()` and `HIDDEN`, without bindings, which the vertex shaders of
/// the edge lines of the viewers import, see [`crate::EdgeLines`].
pub const SILHOUETTE_WGSL: &str = include_str!("../shaders/silhouette.wgsl");

/// The grid for WebGL2, e.g. three.js, with its vertex and fragment shader, which imports
/// [`GRID_WGSL`] and which `cargo xtask web` converts to GLSL.
pub const GRID_WEB_WGSL: &str = include_str!("../shaders/grid_web.wgsl");

/// The lit models for WebGL2, e.g. three.js, with their vertex and fragment shader, which import
/// [`LIGHTING_WGSL`] and which `cargo xtask web` converts to GLSL.
pub const MODEL_WEB_WGSL: &str = include_str!("../shaders/model_web.wgsl");
