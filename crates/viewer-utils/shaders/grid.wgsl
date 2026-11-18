// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

// The grid of the µcad viewers: a zoom-adaptive grid with colored axes, which fades out at a
// radius. It has no bindings, the viewers import it, e.g. in Bevy, or into the WebGL shader of
// `grid_web.wgsl`.

#define_import_path mucad_viewer_utils::grid

/// The parameters of the grid, the colors are linear RGB.
struct GridConfig {
    /// The radius, at which the grid fades out.
    radius: f32,
    /// The zoom level, which defines the spacing of the grid lines.
    zoom_level: f32,
    /// The view direction of the camera, a flat view fades the grid out.
    view_angle: vec3<f32>,
    grid_color: vec3<f32>,
    x_axis_color: vec3<f32>,
    y_axis_color: vec3<f32>,
}

fn grid_opacity(pos: vec2<f32>, zoom: f32, config: GridConfig) -> vec4<f32> {
    let angle_z = pow(abs(config.view_angle.z), 1.0 / 3.0);

    let logz = log2(zoom / angle_z) / log2(10.0); // log10(zoom)
    let spacing = pow(10.0, floor(logz));
    let next_spacing = spacing * 10.0;
    let blend = fract(logz); // fade factor between levels

    // World-space UVs
    let uv_a = pos / spacing;
    let uv_b = pos / next_spacing;

    // Distance to nearest grid line (centered)
    let d_a = min(
        abs(fract(uv_a.x - 0.5) - 0.5),
        abs(fract(uv_a.y - 0.5) - 0.5),
    );
    let d_b = min(
        abs(fract(uv_b.x - 0.5) - 0.5),
        abs(fract(uv_b.y - 0.5) - 0.5),
    );

    // Screen-space derivatives (anti-aliasing width)
    let fw_a = length(fwidth(uv_a)) * angle_z;
    let fw_b = length(fwidth(uv_b)) * angle_z;

    // Anti-aliased line intensities
    let line_a = 1.0 - smoothstep(0.0, fw_a, d_a);
    let line_b = 1.0 - smoothstep(0.0, fw_b, d_b);

    // Weighting to ensure brightness doesn't vary
    let w_a = 1.0 - sqrt(blend);
    let w_b = sqrt(blend);

    let grid = (line_a * w_a + line_b * w_b) / (w_a + w_b); // Always adds to 1.0 total brightness

    var color = config.grid_color;
    if fw_a < abs(uv_a.x) && fw_a > abs(uv_a.y)  {
        color = config.x_axis_color;
    }
    if fw_a < abs(uv_a.y) && fw_a > abs(uv_a.x) {
        color = config.y_axis_color;
    }

    let fade_z = (max(angle_z / 0.8 - 0.2, 0.0));

    return vec4<f32>(color, grid * fade_z);
}

/// The color of the grid at `p` in world coordinates with its opacity, faded out at the radius.
fn grid_color(p: vec2<f32>, config: GridConfig) -> vec4<f32> {
    let grid = grid_opacity(p, 8.0 / config.zoom_level, config);
    // Not negative beyond the double radius, e.g. in a float render target, which does not clamp.
    let fade = clamp(2.0 - length(p) / config.radius, 0.0, 1.0);
    return vec4(grid.xyz, grid.w * fade);
}
