// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Grid material.

use bevy::render::render_resource::*;

use mucad_viewer_utils::GridConfig;

use super::bevy_types::*;

/// A colored zoom-adaptive grid and fade out radius.
#[derive(Asset, AsBindGroup, Debug, Clone, Default, TypePath)]
pub struct Grid {
    /// Grid parameters, `GridConfig` of the grid shader.
    #[uniform(0)]
    pub parameters: GridConfig,
}

impl Grid {
    fn shader_ref() -> ShaderRef {
        super::shader_ref_from_str("grid.wgsl")
    }
}

impl Material for Grid {
    fn fragment_shader() -> ShaderRef {
        Self::shader_ref()
    }

    fn vertex_shader() -> ShaderRef {
        Self::shader_ref()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None; // ✅ Disable backface culling
        Ok(())
    }
}
