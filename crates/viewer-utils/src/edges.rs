// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The contour edges of triangle meshes, which the viewers draw, and the crease angle.

use std::collections::HashMap;

use glam::Vec3;
use mucad_core::TriangleMesh;

/// The angle in degrees between the normals of two faces, above which their edge is sharp: it
/// is a contour edge, and the normals are not smoothed across it.
pub const CREASE_ANGLE_DEGREES: f64 = 30.0;

/// An edge between two vertices with its first two triangles and the number of its triangles.
struct Edge {
    vertices: [u32; 2],
    triangles: [u32; 2],
    count: u32,
}

/// The edges of a mesh with the normals of its triangles.
#[derive(Default)]
struct Edges {
    /// The edges in the order they are found.
    edges: Vec<Edge>,
    /// The index of an edge by its vertices.
    index: HashMap<[u32; 2], usize>,
    /// The normals of the triangles, which are indexed by the edges.
    normals: Vec<Vec3>,
}

impl Edges {
    /// Add a triangle with `normal` to its three edges.
    fn add(&mut self, [a, b, c]: [u32; 3], normal: Vec3) {
        let triangle = self.normals.len() as u32;
        self.normals.push(normal);
        for (v1, v2) in [(a, b), (b, c), (c, a)] {
            let vertices = [v1.min(v2), v1.max(v2)];
            let edges = &mut self.edges;
            let index = *self.index.entry(vertices).or_insert_with(|| {
                edges.push(Edge {
                    vertices,
                    triangles: [triangle; 2],
                    count: 0,
                });
                edges.len() - 1
            });
            let edge = &mut self.edges[index];
            if edge.count < 2 {
                edge.triangles[edge.count as usize] = triangle;
            }
            edge.count += 1;
        }
    }

    /// The normal of the only triangle of the edge between `vertices`, if it has one triangle.
    fn only_normal(&self, vertices: [u32; 2]) -> Option<Vec3> {
        let key = [vertices[0].min(vertices[1]), vertices[0].max(vertices[1])];
        let edge = &self.edges[*self.index.get(&key)?];
        (edge.count == 1).then(|| self.normals[edge.triangles[0] as usize])
    }
}

/// The contour edges of `mesh` as pairs of vertex indices, e.g. to draw its outlines.
///
/// An edge is a contour, if the normals of its two triangles differ more than
/// `crease_angle_degrees`, or if it has not exactly two triangles, e.g. at an open border.
/// Vertices at the same position are the same.
///
/// A triangle without area, e.g. with three points on a line, has no normal of its own. If it
/// bridges its neighbors, i.e. its longest edge has one triangle and its other edges are next
/// to other triangles, it gets the normal of the triangle at its longest edge. Then its edges
/// are no open borders, which would be drawn as stray lines, e.g. across a flat face. Other
/// triangles without area are ignored.
pub fn contour_edges(mesh: &TriangleMesh, crease_angle_degrees: f64) -> Vec<[u32; 2]> {
    // The first vertex at each position, so that equal vertices of triangles are found.
    let mut first_at = HashMap::<[u32; 3], u32>::with_capacity(mesh.positions.len());
    let welded: Vec<u32> = mesh
        .positions
        .iter()
        .enumerate()
        .map(|(index, position)| {
            let bits = [position.x, position.y, position.z].map(f32::to_bits);
            *first_at.entry(bits).or_insert(index as u32)
        })
        .collect();
    let position = |vertex: u32| {
        let p = mesh.positions[vertex as usize];
        Vec3::new(p.x, p.y, p.z)
    };

    let mut edges = Edges::default();
    // The triangles without area with their longest edge.
    let mut without_area = Vec::new();
    for triangle in &mesh.triangle_indices {
        let vertices = [triangle.0, triangle.1, triangle.2].map(|i| welded[i as usize]);
        let [pa, pb, pc] = vertices.map(position);
        let (u, v) = (pb - pa, pc - pa);
        let normal = u.cross(v);
        // Without area, the normal of the triangle is undefined.
        if normal.length() <= f32::EPSILON * u.length_squared().max(v.length_squared()) {
            let [a, b, c] = vertices;
            let longest = [[a, b], [b, c], [c, a]]
                .into_iter()
                .max_by(|e1, e2| {
                    let length = |[v1, v2]: [u32; 2]| position(v1).distance_squared(position(v2));
                    length(*e1).total_cmp(&length(*e2))
                })
                .unwrap_or([a, b]);
            without_area.push((vertices, longest));
        } else {
            edges.add(vertices, normal.normalize());
        }
    }

    // The triangles without area, which bridge their neighbors, also in chains of them.
    loop {
        let count = without_area.len();
        without_area.retain(|&(vertices, longest)| match edges.only_normal(longest) {
            Some(normal) => {
                edges.add(vertices, normal);
                false
            }
            None => true,
        });
        if without_area.len() == count {
            break;
        }
    }

    let cos_threshold = crease_angle_degrees.to_radians().cos() as f32;
    let normals = &edges.normals;
    edges
        .edges
        .iter()
        .filter(|edge| {
            let [n1, n2] = edge.triangles.map(|t| normals[t as usize]);
            edge.count != 2 || n1.dot(n2) < cos_threshold
        })
        .map(|edge| edge.vertices)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mucad_core::Triangle;

    fn mesh(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> TriangleMesh {
        TriangleMesh {
            positions: positions
                .iter()
                .map(|p| cgmath::Vector3::new(p[0], p[1], p[2]))
                .collect(),
            normals: None,
            triangle_indices: triangles
                .iter()
                .map(|t| Triangle(t[0], t[1], t[2]))
                .collect(),
        }
    }

    #[test]
    fn square() {
        // A flat square of two triangles: the diagonal is no contour, the open border is.
        // The second triangle has its own vertices at the same positions.
        let square = mesh(
            &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            &[[0, 1, 2], [3, 4, 5]],
        );
        assert_eq!(contour_edges(&square, 20.0).len(), 4);
    }

    #[test]
    fn tetrahedron() {
        // All edges of a tetrahedron are sharp, a triangle without area on one of its edges
        // bridges nothing and is ignored.
        let tetrahedron = mesh(
            &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.5, 0.0, 0.0],
            ],
            &[[0, 2, 1], [0, 1, 3], [1, 2, 3], [2, 0, 3], [0, 4, 1]],
        );
        assert_eq!(contour_edges(&tetrahedron, 20.0).len(), 6);
    }

    /// A flat strip, whose two halves meet at a vertex in the middle of the long edge of the
    /// other half, with a triangle without area between them, e.g. like in the mesh of a
    /// manifold.
    fn strip_with_triangle_without_area(z_of_top_right: f32) -> TriangleMesh {
        // 0 ------ 1 ------ 2      The top half is one triangle 0 2 4 along the long edge 0-2,
        //   \      |      /        the bottom half are the triangles 0 1 3 and 1 2 3 with
        //     \    |    /          the vertex 1 in the middle of the long edge, and the
        //       3 -+- 4            triangle 0 1 2 without area bridges them.
        mesh(
            &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, -1.0, 0.0],
                [1.0, 1.0, z_of_top_right],
            ],
            &[[0, 2, 4], [0, 3, 1], [1, 3, 2], [0, 1, 2]],
        )
    }

    #[test]
    fn no_stray_lines_across_flat_faces() {
        let edges = contour_edges(&strip_with_triangle_without_area(0.0), 20.0);
        // Only the open border: 0-4, 4-2, 0-3, 3-2, and not 0-1, 1-2 or 0-2 across the strip.
        assert_eq!(edges.len(), 4, "{edges:?}");
        for inner in [[0, 1], [1, 2], [0, 2]] {
            assert!(!edges.contains(&inner), "{inner:?} in {edges:?}");
        }
    }

    #[test]
    fn creases_at_triangles_without_area() {
        // The top half is bent upwards, the edges along the bend are drawn at the vertices of
        // the bottom half, the long edge of the triangle without area is not.
        let edges = contour_edges(&strip_with_triangle_without_area(2.0), 20.0);
        assert!(
            edges.contains(&[0, 1]) && edges.contains(&[1, 2]),
            "{edges:?}"
        );
        assert!(!edges.contains(&[0, 2]), "{edges:?}");
    }

    #[test]
    fn crease_angle() {
        // Two faces at 25° are no contour at the crease angle of 30°, but at 20°.
        let angle = 25f32.to_radians();
        let folded = mesh(
            &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, -angle.cos(), angle.sin()],
            ],
            &[[0, 1, 2], [1, 0, 3]],
        );
        let border = 4;
        assert_eq!(contour_edges(&folded, CREASE_ANGLE_DEGREES).len(), border);
        assert_eq!(contour_edges(&folded, 20.0).len(), border + 1);
    }
}
