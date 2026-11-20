// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

// The edge lines of `silhouette.wgsl` for WebGL2, e.g. three.js, which `cargo xtask web` converts
// to GLSL: the contour edges are always drawn, the smooth edges only on the silhouette, like in
// Bevy. The positions, normals and midpoints are world coordinates, the color is linear.

#import mucad_viewer_utils::silhouette::{HIDDEN, in_front, is_silhouette, to_viewer}

struct Camera {
    /// The projection and the view of the camera, e.g. of three.js.
    clip_from_world: mat4x4<f32>,
    /// The position of the camera.
    position: vec3<f32>,
    /// The size of a pixel, see `in_front`.
    pixel_size: f32,
    /// The direction from the camera backwards, i.e. towards the viewer.
    back: vec3<f32>,
    /// 1 for an orthographic projection, 0 for a perspective one.
    orthographic: f32,
}

struct Edge {
    /// The linear color with alpha.
    color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> edge: Edge;

@vertex
fn vertex(
    @location(0) position: vec3<f32>,
    @location(1) normal_a: vec3<f32>,
    @location(2) normal_b: vec3<f32>,
    @location(3) midpoint: vec3<f32>,
) -> @builtin(position) vec4<f32> {
    let orthographic = camera.orthographic > 0.5;
    let direction = to_viewer(midpoint, camera.position, camera.back, orthographic);
    if !is_silhouette(normal_a, normal_b, direction) {
        return HIDDEN;
    }
    let point = in_front(position, camera.position, camera.back, orthographic, camera.pixel_size);
    return camera.clip_from_world * vec4<f32>(point, 1.0);
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return edge.color;
}
