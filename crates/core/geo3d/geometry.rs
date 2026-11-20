// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    traits::{Center, TotalMemory, TransformAffine, VertexCount},
    *,
};

use derive_more::From;
use strum::IntoStaticStr;

use crate::geo3d::*;

/// 3D Geometry
#[derive(IntoStaticStr, From, Clone)]
pub enum Geometry3D {
    /// Triangle mesh.
    Mesh(TriangleMesh),
    /// Manifold.
    Manifold(Box<Manifold>),
    /// Collection.
    Collection(Geometries3D),
}

impl std::fmt::Debug for Geometry3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        write!(f, "{name}")
    }
}

impl Geometry3D {
    /// Return name of geometry.
    pub fn name(&self) -> &'static str {
        self.into()
    }

    /// Execute boolean operation.
    pub fn boolean_op(&self, other: &Geometry3D, op: BooleanOp) -> Option<Self> {
        let a: Manifold = self.clone().into();
        let b: Manifold = other.clone().into();
        Some(a.boolean(&b, op.into()).into())
    }

    /// Calculate contex hull.
    pub fn hull(&self) -> Self {
        match &self {
            Geometry3D::Mesh(triangle_mesh) => triangle_mesh.to_manifold().convex_hull().into(),
            Geometry3D::Manifold(manifold) => manifold.convex_hull().into(),
            Geometry3D::Collection(collection) => TriangleMesh::from(collection)
                .to_manifold()
                .convex_hull()
                .into(),
        }
    }

    /// Return this geometry with calculated bounds.
    pub fn with_bounds(self) -> WithBounds3D<Geometry3D> {
        let bounds = self.calc_bounds_3d();
        WithBounds3D {
            bounds,
            inner: self,
        }
    }
}

impl std::fmt::Display for Geometry3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl CalcBounds3D for Geometry3D {
    fn calc_bounds_3d(&self) -> Bounds3D {
        match self {
            Geometry3D::Mesh(triangle_mesh) => triangle_mesh.calc_bounds_3d(),
            Geometry3D::Manifold(manifold) => {
                TriangleMesh::from(manifold.get_mesh_gl(0)).calc_bounds_3d()
            }
            Geometry3D::Collection(collection) => collection.calc_bounds_3d(),
        }
    }
}

/// Convert a DMat4 into a matrix for manifold.
pub fn mat4_to_mat3x4(m: &DMat4) -> manifold_rust::linalg::Mat3x4 {
    use manifold_rust::linalg::{Mat3x4, Vec3};
    Mat3x4 {
        x: Vec3::new(m.x_axis.x, m.x_axis.y, m.x_axis.z),
        y: Vec3::new(m.y_axis.x, m.y_axis.y, m.y_axis.z),
        z: Vec3::new(m.z_axis.x, m.z_axis.y, m.z_axis.z),
        w: Vec3::new(m.w_axis.x, m.w_axis.y, m.w_axis.z),
    }
}

impl TransformAffine for Geometry3D {
    fn transform_affine(&mut self, m: &DMat4) {
        match self {
            Geometry3D::Mesh(triangle_mesh) => triangle_mesh.transform_affine(m),
            Geometry3D::Manifold(manifold) => {
                *self = manifold.transform(&mat4_to_mat3x4(m)).into();
            }
            Geometry3D::Collection(geo3d) => geo3d.transform_affine(m),
        }
    }
}

impl Transformed3D for Geometry3D {
    fn transformed_3d(&self, mat: &DMat4) -> Self {
        TriangleMesh::from(self.clone()).transformed_3d(mat).into()
    }
}

impl Center for Geometry3D {
    fn center(&self) -> Self {
        let d: DVec3 = self.calc_bounds_3d().center();
        self.transformed_3d(&DMat4::from_translation(-d))
    }
}

impl From<Geometry3D> for Manifold {
    fn from(geo: Geometry3D) -> Self {
        match geo {
            Geometry3D::Mesh(triangle_mesh) => triangle_mesh.to_manifold(),
            Geometry3D::Manifold(manifold) => *manifold,
            Geometry3D::Collection(ref collection) => TriangleMesh::from(collection).to_manifold(),
        }
    }
}

impl From<Manifold> for Geometry3D {
    fn from(manifold: Manifold) -> Self {
        Self::Manifold(Box::new(manifold))
    }
}

impl TotalMemory for Manifold {}

impl TotalMemory for Geometry3D {
    fn heap_memory(&self) -> usize {
        match &self {
            Geometry3D::Mesh(triangle_mesh) => triangle_mesh.heap_memory(),
            Geometry3D::Manifold(manifold) => manifold.heap_memory(),
            Geometry3D::Collection(collection) => collection.heap_memory(),
        }
    }
}

impl VertexCount for Manifold {
    fn vertex_count(&self) -> usize {
        0 // TODO: Get number of vertices for Manifold via C++ API
    }
}

impl VertexCount for Geometry3D {
    fn vertex_count(&self) -> usize {
        match &self {
            Geometry3D::Mesh(triangle_mesh) => triangle_mesh.vertex_count(),
            Geometry3D::Manifold(manifold) => manifold.vertex_count(),
            Geometry3D::Collection(collection) => collection.vertex_count(),
        }
    }
}
