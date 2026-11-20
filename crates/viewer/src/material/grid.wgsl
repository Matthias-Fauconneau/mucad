// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

// The grid material of Bevy with the grid of `mucad_viewer_utils::grid`, which the web viewer
// shares.

#import bevy_pbr::forward_io::{Vertex, VertexOutput};
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import mucad_viewer_utils::grid::{GridConfig, grid_color}

@group(3) @binding(0) var<uniform> grid_config: GridConfig;

@vertex
fn vertex(input: Vertex) -> VertexOutput {
    var output: VertexOutput;

    let matrix = get_world_from_local(input.instance_index);
    // Translate vertex position -0.05 in Z
    let world_pos = vec4<f32>(input.position + vec3<f32>(0.0, 0.0, -0.05 / grid_config.zoom_level), 1.0);

    output.position = mesh_position_local_to_clip(matrix, world_pos);
    output.world_position = vec4<f32>(input.position, 1.0);
    output.uv = input.uv;
    return output;
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return grid_color(mesh.world_position.xy, grid_config);
}
