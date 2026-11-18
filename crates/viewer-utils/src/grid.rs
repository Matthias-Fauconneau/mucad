// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The parameters of the grid shader of the viewers, see [`crate::GRID_WGSL`].

use glam::Vec3;

use crate::{CameraController, Theme};

/// The parameters of the grid shader, `GridConfig` of [`crate::GRID_WGSL`] with the same
/// layout, which a test checks.
///
/// With the feature `encase`, it is a uniform of Bevy, e.g. `#[uniform(0)]` of a material.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "encase", derive(encase::ShaderType))]
pub struct GridConfig {
    /// The radius, at which the grid fades out.
    pub radius: f32,
    /// The zoom level, which defines the spacing of the grid lines.
    pub zoom_level: f32,
    /// The view direction of the camera, a flat view fades the grid out.
    pub view_angle: Vec3,
    /// The color of the grid lines, linear RGB.
    pub grid_color: Vec3,
    /// The color of the x axis, linear RGB.
    pub x_axis_color: Vec3,
    /// The color of the y axis, linear RGB.
    pub y_axis_color: Vec3,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            radius: 1.0,
            zoom_level: 1.0,
            view_angle: Vec3::Z,
            grid_color: Vec3::new(0.7, 0.7, 0.7),
            x_axis_color: Vec3::X,
            y_axis_color: Vec3::Y,
        }
    }
}

impl GridConfig {
    /// The grid for the camera of `controller` with the colors of `theme`: it fades out at the
    /// radius of the bounds, and its spacing follows the zoom.
    ///
    /// The theme colors are passed as they are, i.e. as linear RGB, like the viewers did before.
    pub fn new(controller: &CameraController, theme: &Theme) -> Self {
        let radius = controller.bounds().radius as f32;
        let window = controller.window_size().as_vec2();
        let color = |color: mucad_core::Color| Vec3::new(color.r, color.g, color.b);
        Self {
            radius,
            zoom_level: window.max_element() / 20.0 * controller.zoom_level() as f32 / radius,
            view_angle: controller.camera().view_vector().normalize(),
            grid_color: color(theme.darker),
            x_axis_color: color(theme.brighter),
            y_axis_color: color(theme.brighter),
        }
    }
}
