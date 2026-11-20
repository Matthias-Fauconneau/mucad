// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

// The grid of `grid.wgsl` for WebGL2, e.g. three.js, which `cargo xtask web` converts to GLSL.
//
// The grid plane lies in Z = 0 and is drawn a little below it, so that 2D models on it stay
// visible. The color is linear like in Bevy, three.js converts it to sRGB after blending.

#import mucad_viewer_utils::grid::{GridConfig, grid_color}

struct Camera {
    /// The projection and the view of the camera, e.g. of three.js.
    clip_from_world: mat4x4<f32>,
    /// The offset of the grid plane in Z, e.g. `-0.05 / zoom_level`.
    z_offset: f32,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> config: GridConfig;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec2<f32>,
}

@vertex
fn vertex(@location(0) position: vec3<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.clip_from_world * vec4<f32>(position + vec3<f32>(0.0, 0.0, camera.z_offset), 1.0);
    output.world_position = position.xy;
    return output;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    return grid_color(input.world_position, config);
}
