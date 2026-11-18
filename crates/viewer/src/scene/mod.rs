// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! mucad viewer scene elements and routines.

use crate::*;
use bevy::prelude::*;
use mu::viewer::CameraControl;
use mucad_driver::prelude as mu;

mod angle;
mod camera;
pub use camera::{
    CameraView,
    camera_controller::{
        CameraControllerPlugin, CameraControllerSystems, CameraNavigation, SelectAt,
    },
};
mod grid;
mod overlay;
mod ruler;

/// The inverse zoom level of the camera, e.g. like the scale of an orthographic projection:
/// 1 shows the whole scene, a smaller one shows the model bigger.
pub fn get_current_zoom_level(controller: &dyn CameraControl) -> f32 {
    1.0 / controller.zoom_level() as f32
}

/// The current resolution in mm: the larger side of the visible area at the pivot per the
/// larger side of the window, times 10.
pub fn get_current_resolution(controller: &dyn CameraControl) -> f32 {
    let height = controller.visible_area().1;
    let area_size = height.max(height * controller.aspect_ratio());
    let window = controller.window_size();
    (area_size / window.x.max(window.y) * 10.0) as f32
}

/// A system that draws hit indicators for every pointer.
pub fn draw_mesh_intersections(
    pointers: Query<&bevy::picking::pointer::PointerInteraction>,
    mut gizmos: Gizmos,
    view_model: Res<ViewModel>,
    navigations: Query<&CameraNavigation>,
) {
    for (_entity, hit) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
    {
        if let Some(position) = hit.position {
            let Ok(navigation) = navigations.single() else {
                return;
            };
            let zoom = get_current_zoom_level(&**navigation.controller());
            let color: Color = view_model.config.theme.guide.to_bevy();
            gizmos.sphere(position, zoom * 0.1, color);
        }
    }
}

/// Scene radius.
pub struct Scene {
    /// Radius of scene's bounding sphere.
    pub radius: f32,
    /// The scene grid (will be `Some` after the grid has been spawned).
    pub grid_entity: Option<Entity>,
    /// Model entities.
    pub model_entities: Vec<Entity>,
}

impl Scene {
    /// Minimum radius in mm for a scene.
    pub const MINIMUM_RADIUS: f32 = 10.0;
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            radius: 100.0,
            grid_entity: Default::default(),
            model_entities: Default::default(),
        }
    }
}

/// The bevy scene plugin contains all systems that are to be rendered except the µcad models.
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraControllerPlugin)
            .add_systems(
                Update,
                camera::update_bounds.before(CameraControllerSystems),
            )
            .add_systems(Startup, grid::spawn_grid_plane)
            //.add_systems(Startup, angle::spawn_angle_plane)
            //.add_systems(Startup, ruler::spawn_ruler_plane)
            .add_systems(Startup, camera::setup_camera)
            .add_systems(Startup, overlay::setup_overlay)
            .add_systems(Update, draw_mesh_intersections)
            .add_systems(Update, overlay::update_overlay)
            .add_systems(Update, grid::update_grid.after(CameraControllerSystems))
            .add_systems(Update, grid::toggle_grid);
    }
}
