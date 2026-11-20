// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

// The silhouette edges of the µcad viewers: a smooth edge is on the silhouette, if one of its two
// triangles faces the viewer and the other one faces away, e.g. the left and right side of a
// cylinder. The vertex shaders of the edge lines hide the other edges, the viewers import it,
// e.g. in Bevy and WebGL. Contour edges have zero normals, so they are always drawn.

#define_import_path mucad_viewer_utils::silhouette

/// A clip position outside of the view: a line, whose vertices both have it, is not drawn.
const HIDDEN: vec4<f32> = vec4<f32>(2.0, 2.0, 2.0, 1.0);

/// The direction from `point` towards the viewer in world coordinates: the direction of the
/// camera in an orthographic view, towards its position in a perspective one.
fn to_viewer(point: vec3<f32>, camera_position: vec3<f32>, camera_back: vec3<f32>, orthographic: bool) -> vec3<f32> {
    return select(camera_position - point, camera_back, orthographic);
}

/// The distance in pixels, by which the edge lines are moved towards the viewer, see `in_front`.
const IN_FRONT_PIXELS: f32 = 4.0;

/// `point` of an edge line moved towards the viewer by `IN_FRONT_PIXELS`, so that the line is in
/// front of its faces, even of a face, which is seen nearly edge-on, e.g. next to a silhouette
/// edge. It is moved along the view ray, so it stays at the same place on the screen.
///
/// `pixel_size` is the size of a pixel in world units in an orthographic view, at the distance 1
/// in a perspective one: `2 / (clip_from_view[1][1] * viewport_height)`.
fn in_front(point: vec3<f32>, camera_position: vec3<f32>, camera_back: vec3<f32>, orthographic: bool, pixel_size: f32) -> vec3<f32> {
    let direction = to_viewer(point, camera_position, camera_back, orthographic);
    let distance = select(length(direction), 1.0, orthographic);
    return point + normalize(direction) * IN_FRONT_PIXELS * pixel_size * distance;
}

/// Whether an edge between two triangles with the normals `normal_a` and `normal_b` is on the
/// silhouette, seen from `to_viewer`, e.g. of its midpoint, always with zero normals.
fn is_silhouette(normal_a: vec3<f32>, normal_b: vec3<f32>, to_viewer: vec3<f32>) -> bool {
    return dot(normal_a, to_viewer) * dot(normal_b, to_viewer) <= 0.0;
}
