// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

// The lighting of the µcad viewers for CAD models: a hemisphere in world space, which keeps up
// and down recognizable, and a key and a fill light in view space, which follow the camera, so
// that the faces towards the viewer are always lit. Its intensities add up to about 1, so that
// the brightest faces have their base color. The viewers import it, e.g. in Bevy and WebGL.

#define_import_path mucad_viewer_utils::lighting

/// The parameters of the lighting, the directions are towards the lights.
struct Lighting {
    /// The direction of the key light in view space, e.g. from the upper left.
    key_direction: vec3<f32>,
    /// The intensity of the key light.
    key: f32,
    /// The direction of the fill light in view space, e.g. from the right.
    fill_direction: vec3<f32>,
    /// The intensity of the fill light.
    fill: f32,
    /// The intensity of the hemisphere from above, i.e. +Z in world space.
    sky: f32,
    /// The intensity of the hemisphere from below, i.e. -Z in world space.
    ground: f32,
    /// The intensity of the highlight of the key light.
    specular: f32,
    /// The exponent of the highlight, a smaller one is broader.
    shininess: f32,
}

/// The color of a surface of `base_color` with its normal in world and view space, which faces
/// the viewer, e.g. flipped on back faces. The colors are linear.
fn shade(
    base_color: vec3<f32>,
    world_normal: vec3<f32>,
    view_normal: vec3<f32>,
    lighting: Lighting,
) -> vec3<f32> {
    let n_world = normalize(world_normal);
    let n_view = normalize(view_normal);
    let key_direction = normalize(lighting.key_direction);
    let fill_direction = normalize(lighting.fill_direction);

    let hemisphere = mix(lighting.ground, lighting.sky, 0.5 + 0.5 * n_world.z);
    let key = lighting.key * max(dot(n_view, key_direction), 0.0);
    let fill = lighting.fill * max(dot(n_view, fill_direction), 0.0);

    // Blinn-Phong of the key light, the viewer looks along -Z in view space.
    let half_vector = normalize(key_direction + vec3<f32>(0.0, 0.0, 1.0));
    let highlight = lighting.specular * pow(max(dot(n_view, half_vector), 0.0), lighting.shininess);

    return base_color * (hemisphere + key + fill) + vec3<f32>(highlight);
}
