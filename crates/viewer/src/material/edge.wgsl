// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

// The edge material of Bevy: the contour edges are always drawn, the smooth edges only on the
// silhouette of `mucad_viewer_utils::silhouette`, which the web viewer shares.

#import bevy_pbr::{
    mesh_functions::{get_world_from_local, mesh_normal_local_to_world, mesh_position_local_to_world},
    view_transformations::position_world_to_clip,
    mesh_view_bindings::view,
}
#import mucad_viewer_utils::silhouette::{HIDDEN, in_front, is_silhouette, to_viewer}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> color: vec4<f32>;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal_a: vec3<f32>,
    @location(2) normal_b: vec3<f32>,
    @location(3) midpoint: vec3<f32>,
};

@vertex
fn vertex(input: Vertex) -> @builtin(position) vec4<f32> {
    let world_from_local = get_world_from_local(input.instance_index);
    let midpoint = (world_from_local * vec4<f32>(input.midpoint, 1.0)).xyz;
    // The projection is orthographic, if the clip W does not depend on the depth.
    let orthographic = view.clip_from_view[3][3] == 1.0;
    let camera_back = view.world_from_view[2].xyz;
    let direction = to_viewer(midpoint, view.world_position, camera_back, orthographic);
    let normal_a = mesh_normal_local_to_world(input.normal_a, input.instance_index);
    let normal_b = mesh_normal_local_to_world(input.normal_b, input.instance_index);
    if !is_silhouette(normal_a, normal_b, direction) {
        return HIDDEN;
    }
    let position = mesh_position_local_to_world(world_from_local, vec4<f32>(input.position, 1.0)).xyz;
    let pixel_size = 2.0 / (view.clip_from_view[1][1] * view.viewport.w);
    return position_world_to_clip(in_front(position, view.world_position, camera_back, orthographic, pixel_size));
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return color;
}
