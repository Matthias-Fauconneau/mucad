// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod camera_controller;

use bevy::{prelude::*, window::PrimaryWindow};
use camera_controller::CameraNavigation;
use mu::viewer::BoundingSphere;
use mucad_driver::prelude as mu;

use std::sync::Arc;

use crate::{ViewModel, actions::Actions};

/// The view of the camera at the start, e.g. from the command line.
#[derive(Clone, Debug, PartialEq)]
pub struct CameraView {
    /// The direction from the model to the camera, e.g. `(1, -1, 1)` from the front right above.
    pub direction: Vec3,
    /// The zoom, `1.0` shows the whole scene, `2.0` the model twice as big.
    pub zoom: f32,
}

impl Default for CameraView {
    fn default() -> Self {
        Self {
            direction: Vec3::new(1.0, -1.0, 1.0),
            zoom: 1.0,
        }
    }
}

/// The size of a window, which is not open yet.
const DEFAULT_WINDOW_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

/// Spawn the camera, which shows the scene from the direction and with the zoom of the view.
///
/// The bindings of the controls replace the ones of the actions, invalid bindings are logged and
/// keep the defaults.
pub fn setup_camera(
    mut commands: Commands,
    state: Res<ViewModel>,
    mut actions: ResMut<Actions>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window_size = windows
        .single()
        .map_or(DEFAULT_WINDOW_SIZE, |window| window.size());
    let controls = state.config.controls.clone();
    if let Err(error) = Arc::make_mut(&mut actions.0).set_bindings(&controls.bindings) {
        log::error!("Invalid controls: {error}");
    }
    let mut navigation = CameraNavigation::new(window_size, controls, actions.0.clone());
    show_view(
        &mut navigation,
        state.scene.radius,
        &state.config.camera_view,
    );

    commands.spawn((
        Camera3d::default(),
        // The colors are shown as they are, e.g. of the theme, like in the web viewer.
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        navigation,
        Projection::from(OrthographicProjection::default_3d()),
        Transform::default(),
    ));
}

/// Show the whole scene of `radius` around the origin from the direction and with the zoom of
/// `view`.
fn show_view(navigation: &mut CameraNavigation, radius: f32, view: &CameraView) {
    let direction = view
        .direction
        .normalize_or(Vec3::new(1.0, -1.0, 1.0).normalize());
    // Z is up, unless the camera looks along it.
    let up = if direction.cross(Vec3::Z).length_squared() < 1e-6 {
        Vec3::Y
    } else {
        Vec3::Z
    };
    let controller = navigation.controller_mut();
    controller.set_bounds(BoundingSphere::new(Vec3::ZERO, radius as f64));
    controller.set_view(-direction, up);
    controller.show_all();
    controller.set_zoom_level(view.zoom as f64);
    controller.reset_pivot();
}

/// Set the bounds of the camera to the radius of the scene, e.g. of a new model, which keeps the
/// zoom level, i.e. the model is shown in the same size.
pub fn update_bounds(state: Res<ViewModel>, mut navigations: Query<&mut CameraNavigation>) {
    let radius = state.scene.radius as f64;
    for mut navigation in &mut navigations {
        let bounds = navigation.controller().bounds();
        if bounds.radius == radius {
            continue;
        }
        let controller = navigation.controller_mut();
        let zoom_level = controller.zoom_level();
        controller.set_bounds(BoundingSphere::new(bounds.pos, radius));
        controller.set_zoom_level(zoom_level);
    }
}
