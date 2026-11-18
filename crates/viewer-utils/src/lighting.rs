// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The lighting of the models in the viewers, see [`crate::LIGHTING_WGSL`].

use glam::Vec3;

/// The parameters of the lighting, `Lighting` of [`crate::LIGHTING_WGSL`] with the same layout.
///
/// A hemisphere in world space keeps up and down recognizable, a key and a fill light in view
/// space follow the camera, so that the faces towards the viewer are always lit. The default
/// intensities add up to about 1, so that the brightest faces have their base color.
///
/// With the feature `encase`, it is a uniform of Bevy, e.g. `#[uniform(100)]` of a material.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "encase", derive(encase::ShaderType))]
pub struct Lighting {
    /// The direction towards the key light in view space, e.g. from the upper left.
    pub key_direction: Vec3,
    /// The intensity of the key light.
    pub key: f32,
    /// The direction towards the fill light in view space, e.g. from the right.
    pub fill_direction: Vec3,
    /// The intensity of the fill light.
    pub fill: f32,
    /// The intensity of the hemisphere from above, i.e. +Z in world space.
    pub sky: f32,
    /// The intensity of the hemisphere from below, i.e. -Z in world space.
    pub ground: f32,
    /// The intensity of the highlight of the key light.
    pub specular: f32,
    /// The exponent of the highlight, a smaller one is broader.
    pub shininess: f32,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            key_direction: Vec3::new(-1.0, 1.0, 2.0).normalize(),
            key: 0.55,
            fill_direction: Vec3::new(1.0, -0.3, 1.0).normalize(),
            fill: 0.2,
            sky: 0.35,
            ground: 0.15,
            specular: 0.1,
            shininess: 32.0,
        }
    }
}
