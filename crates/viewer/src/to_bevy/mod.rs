// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Conversions from mucad core types to bevy types.

pub mod to_mesh;

pub use to_mesh::ToBevyMesh;

use bevy::{asset::RenderAssetUsages, math::Vec3, platform::collections::HashMap};
use bevy::{
    mesh::{Indices, Mesh},
    transform::components::Transform,
};

pub use mucad_driver::prelude as mu;

/// A trait to convert a µcad type into a bevy type.
pub trait ToBevy<T> {
    /// Convert into bevy type.
    fn to_bevy(self) -> T;
}

/// Convert a µcad color into a bevy color.
impl ToBevy<bevy::prelude::Color> for mu::core::Color {
    fn to_bevy(self) -> bevy::prelude::Color {
        bevy::prelude::Color::srgba(self.r, self.g, self.b, self.a)
    }
}

/// Convert the color of a model attribute (e.g. `#[color = "red"]`) into a bevy color.
impl ToBevy<bevy::prelude::Color> for mu::types::Color {
    fn to_bevy(self) -> bevy::prelude::Color {
        bevy::prelude::Color::srgba(self.r.into(), self.g.into(), self.b.into(), self.a.into())
    }
}

impl ToBevy<bevy::prelude::Vec3> for mu::core::Color {
    fn to_bevy(self) -> bevy::prelude::Vec3 {
        bevy::prelude::Vec3::new(self.r, self.g, self.b)
    }
}

impl ToBevy<bevy::prelude::Mat4> for mu::core::DMat4 {
    fn to_bevy(self) -> bevy::prelude::Mat4 {
        self.as_mat4()
    }
}

impl ToBevy<bevy::prelude::Transform> for mu::core::DMat4 {
    fn to_bevy(self) -> bevy::prelude::Transform {
        Transform::from_matrix(self.to_bevy())
    }
}

#[test]
fn matrix_to_transform() {
    let matrix = mu::core::DMat4::from_translation(mu::core::DVec3::new(1.0, 2.0, 3.0))
        * mu::core::DMat4::from_rotation_z(std::f64::consts::FRAC_PI_2);
    let transform: Transform = matrix.to_bevy();
    assert_eq!(
        transform.translation,
        bevy::prelude::Vec3::new(1.0, 2.0, 3.0)
    );
    assert!(
        (transform.rotation * bevy::prelude::Vec3::X).abs_diff_eq(bevy::prelude::Vec3::Y, 1e-6)
    );
}
