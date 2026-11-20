// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Viewer materials.

use std::marker::PhantomData;

use mucad_driver::prelude as mu;

use bevy::{
    app::{App, Plugin, Startup},
    asset::{Assets, Handle, uuid::Uuid},
    color::Alpha,
    ecs::system::ResMut,
    shader::{Shader, ShaderRef},
};

mod angle;
mod edge;
mod grid;
mod model;
mod ruler;

pub use angle::Angle;
pub use edge::{ATTRIBUTE_MIDPOINT, ATTRIBUTE_NORMAL_A, ATTRIBUTE_NORMAL_B, EdgeMaterial};
pub use grid::Grid;
pub use model::{ModelLighting, ModelMaterial, model_material};
pub use ruler::Ruler;

/// Common types to be used from bevy.
pub mod bevy_types {
    pub use bevy::prelude::{AlphaMode, TypePath, Vec3};
    pub use bevy::{
        asset::Asset,
        pbr::{Material, StandardMaterial},
        render::render_resource::AsBindGroup,
        shader::ShaderRef,
    };
}

/// The shader material asset id. This assures that we can query the correct material from a model node.
pub static BUILTIN_MATERIAL_ASSET_ID: u64 = 0x1234123412341234;

fn asset_uuid_from_str(s: &str) -> Uuid {
    use std::hash::{Hash, Hasher};
    let mut hasher = rustc_hash::FxHasher::default();
    s.hash(&mut hasher);
    Uuid::from_u64_pair(BUILTIN_MATERIAL_ASSET_ID, hasher.finish())
}

/// Build a shader ref from a string (mostly the shader filename).
pub fn shader_ref_from_str(s: &'static str) -> ShaderRef {
    ShaderRef::Handle(Handle::Uuid(asset_uuid_from_str(s), PhantomData))
}

/// Get correct alpha mode for colors.
pub fn alpha_mode_for_color(color: &bevy::prelude::Color) -> bevy_types::AlphaMode {
    if color.alpha() >= 1.0 {
        bevy_types::AlphaMode::Opaque
    } else {
        bevy_types::AlphaMode::Blend
    }
}

/// Create a 2D material (unlit) from render attributes.
pub fn create_2d_material(base_color: bevy::prelude::Color) -> bevy_types::StandardMaterial {
    bevy_types::StandardMaterial {
        alpha_mode: alpha_mode_for_color(&base_color),
        base_color,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..Default::default()
    }
}

/// Create a 3D material (lit) from a color.
pub fn create_3d_material(base_color: bevy::prelude::Color) -> bevy_types::StandardMaterial {
    bevy_types::StandardMaterial {
        alpha_mode: alpha_mode_for_color(&base_color),
        base_color,
        metallic: 0.1,
        unlit: false,
        perceptual_roughness: 0.1,
        reflectance: 0.4,
        ..Default::default()
    }
}

/// The uuid of the material of the edge lines of 3D models, which all models share.
pub const EDGE_MATERIAL_UUID: Uuid = Uuid::from_u64_pair(BUILTIN_MATERIAL_ASSET_ID, 0xED6E);

/// Material plugin.
pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        type Material<T> = bevy::prelude::MaterialPlugin<T>;
        app.add_plugins(Material::<Grid>::default())
            .add_plugins(Material::<ModelMaterial>::default())
            .add_plugins(Material::<EdgeMaterial>::default())
            .add_plugins(Material::<Angle>::default())
            .add_plugins(Material::<Ruler>::default())
            .add_systems(Startup, load_materials);
    }
}

macro_rules! add_shader_asset {
    ($shaders:ident, $s:literal) => {
        $shaders
            .insert(
                asset_uuid_from_str($s),
                Shader::from_wgsl(include_str!($s), $s.to_string()),
            )
            .ok();
    };
}

fn load_materials(mut shaders: ResMut<Assets<Shader>>) {
    // The libraries of `mucad_viewer_utils`, which the shaders import, e.g. `grid.wgsl`.
    for library in mu::viewer::shaders::LIBRARIES {
        let path = format!("mucad_viewer_utils/{}", library.name);
        shaders
            .insert(
                asset_uuid_from_str(&path),
                Shader::from_wgsl(library.source, path),
            )
            .ok();
    }
    add_shader_asset!(shaders, "model.wgsl");
    add_shader_asset!(shaders, "edge.wgsl");
    //add_shader_asset!(shaders, "angle.wgsl");
    //add_shader_asset!(shaders, "arrow.wgsl");
    add_shader_asset!(shaders, "grid.wgsl");
    //add_shader_asset!(shaders, "ruler.wgsl");
}
