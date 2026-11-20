// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

// The model material of Bevy: the standard material with the lighting of
// `mucad_viewer_utils::lighting`, which the web viewer shares, instead of the lights of Bevy.

#import bevy_pbr::{
    forward_io::{FragmentOutput, VertexOutput},
    mesh_view_bindings::view,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#import mucad_viewer_utils::lighting::{Lighting, shade}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> lighting: Lighting;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // The color, alpha and the normal of the standard material, flipped on back faces.
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) != 0u {
        // E.g. 2D models in their flat color.
        out.color = base_color;
    } else {
        let view_normal = (view.view_from_world * vec4<f32>(pbr_input.N, 0.0)).xyz;
        out.color = vec4<f32>(shade(base_color.rgb, pbr_input.N, view_normal, lighting), base_color.a);
    }
    return out;
}
