// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

// The lit models of `lighting.wgsl` for WebGL2, e.g. three.js, which `cargo xtask web` converts to
// GLSL. The positions and normals are world coordinates, the color is linear like in Bevy, three.js
// converts it to sRGB after blending.

#import mucad_viewer_utils::lighting::{Lighting, shade}

struct Camera {
    /// The projection and the view of the camera, e.g. of three.js.
    clip_from_world: mat4x4<f32>,
    /// The view of the camera, for the normals in view space.
    view_from_world: mat4x4<f32>,
}

struct Surface {
    /// The linear color with alpha.
    color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> lighting: Lighting;
@group(0) @binding(2) var<uniform> surface: Surface;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) view_normal: vec3<f32>,
}

@vertex
fn vertex(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.clip_from_world * vec4<f32>(position, 1.0);
    output.world_normal = normal;
    output.view_normal = (camera.view_from_world * vec4<f32>(normal, 0.0)).xyz;
    return output;
}

@fragment
fn fragment(input: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    // The normal faces the viewer, also on back faces, like in Bevy.
    let side = select(-1.0, 1.0, is_front);
    let color = shade(surface.color.rgb, input.world_normal * side, input.view_normal * side, lighting);
    return vec4<f32>(color, surface.color.a);
}
