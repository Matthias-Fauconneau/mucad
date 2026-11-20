// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The material of the models: the standard material of Bevy with the lighting of
//! `mucad_viewer_utils::lighting`, which the web viewer shares.

use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};
use mu::viewer::Lighting;
use mucad_driver::prelude as mu;

/// The material of the models: its colors, alpha and flags are the ones of the standard
/// material, its lighting is the one of [`ModelLighting`], unless it is unlit, e.g. of 2D models.
pub type ModelMaterial = ExtendedMaterial<StandardMaterial, ModelLighting>;

/// The lighting of a [`ModelMaterial`], which replaces the lights of Bevy.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
pub struct ModelLighting {
    /// The lighting, `Lighting` of the shader.
    #[uniform(100)]
    pub lighting: Lighting,
}

impl MaterialExtension for ModelLighting {
    fn fragment_shader() -> ShaderRef {
        super::shader_ref_from_str("model.wgsl")
    }
}

/// The model material of `base` with the default lighting.
pub fn model_material(base: StandardMaterial) -> ModelMaterial {
    ExtendedMaterial {
        base,
        extension: ModelLighting::default(),
    }
}
