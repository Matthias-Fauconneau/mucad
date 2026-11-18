// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The material of the edge lines of 3D models, see
//! [`mucad_driver::prelude::viewer::EdgeLines`]: the contour edges are always drawn, the smooth
//! edges on the silhouette.

use bevy::{
    mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

/// The normal of the first triangle of the edge of a vertex.
pub const ATTRIBUTE_NORMAL_A: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_NormalA", 0xED6E_0001, VertexFormat::Float32x3);

/// The normal of the second triangle of the edge of a vertex.
pub const ATTRIBUTE_NORMAL_B: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_NormalB", 0xED6E_0002, VertexFormat::Float32x3);

/// The midpoint of the edge of a vertex.
pub const ATTRIBUTE_MIDPOINT: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Midpoint", 0xED6E_0003, VertexFormat::Float32x3);

/// The material of the edge lines in one color, which the vertex shader moves in front of their
/// faces, see `in_front` of [`mucad_driver::prelude::viewer::shaders::Silhouette`].
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct EdgeMaterial {
    /// The color of the lines.
    #[uniform(0)]
    pub color: LinearRgba,
}

impl EdgeMaterial {
    fn shader_ref() -> ShaderRef {
        super::shader_ref_from_str("edge.wgsl")
    }
}

impl Material for EdgeMaterial {
    fn vertex_shader() -> ShaderRef {
        Self::shader_ref()
    }

    fn fragment_shader() -> ShaderRef {
        Self::shader_ref()
    }

    // The prepasses and shadows would draw all lines with the default vertex shader.
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.buffers = vec![layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_NORMAL_A.at_shader_location(1),
            ATTRIBUTE_NORMAL_B.at_shader_location(2),
            ATTRIBUTE_MIDPOINT.at_shader_location(3),
        ])?];
        Ok(())
    }
}
