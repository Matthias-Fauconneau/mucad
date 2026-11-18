// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Grid entity.

use bevy::{
    asset::Assets,
    camera::visibility::Visibility,
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    input::{ButtonInput, keyboard::KeyCode},
    math::{Vec2, Vec3, primitives::Plane3d},
    mesh::{Mesh, Mesh3d},
    pbr::MeshMaterial3d,
};
use mu::viewer::GridConfig;
use mucad_driver::prelude as mu;

#[derive(Component)]
pub struct ToggleMe;

use crate::{scene::CameraNavigation, view_model::ViewModel, *};

pub fn toggle_grid(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Visibility, With<ToggleMe>>,
) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        let mut visibility = query.single_mut().expect("Visible");

        *visibility = match *visibility {
            Visibility::Visible => Visibility::Hidden,
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Visible,
        };
    }
}

pub fn spawn_grid_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<material::Grid>>,
    mut state: ResMut<ViewModel>,
) {
    let plane = Mesh::from(Plane3d::new(Vec3::Z, Vec2::new(1000.0, 1000.0)));
    let mesh_handle = meshes.add(plane);

    state.scene.grid_entity = Some(
        commands
            .spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(materials.add(material::Grid::default())),
                bevy::picking::Pickable::IGNORE,
                Visibility::Visible,
                ToggleMe,
            ))
            .id(),
    );
}

/// Update the grid material to the camera: its radius, zoom level and view angle, and the
/// colors of the theme.
pub fn update_grid(
    mut materials: ResMut<Assets<material::Grid>>,
    state: Res<ViewModel>,
    navigations: Query<&CameraNavigation>,
    mat_query: Query<&MeshMaterial3d<material::Grid>>,
) {
    let Ok(navigation) = navigations.single() else {
        return;
    };
    if let Some(grid) = state.scene.grid_entity
        && let Ok(material) = mat_query.get(grid)
        && let Some(material) = materials.get_mut(material)
    {
        material.parameters = GridConfig::new(navigation.controller(), &state.config.theme);
    }
}
