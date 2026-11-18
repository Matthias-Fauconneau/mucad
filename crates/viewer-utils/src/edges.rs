// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The edge lines of triangle meshes, which the viewers draw, and the crease angle.

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

/// The edges of `mesh` with the normals of their triangles, see [`edge_lines`]: vertices at
/// the same position are the same, triangles without area bridge their neighbors.
fn adjacency(mesh: &TriangleMesh) -> Edges {
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
    let position = |vertex: u32| position(mesh, vertex);

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
    edges
}

/// The position of `vertex` of `mesh`.
fn position(mesh: &TriangleMesh, vertex: u32) -> Vec3 {
    let p = mesh.positions[vertex as usize];
    Vec3::new(p.x, p.y, p.z)
}

/// The edge lines of a mesh, which the viewers draw, see [`edge_lines`]: the contour edges,
/// which are always drawn, and the smooth edges, which are drawn on the silhouette, if one of
/// their two triangles faces the viewer and the other one faces away.
///
/// Each line has two vertices, which both have the normals of the two triangles of its edge
/// and its midpoint, so that a vertex shader decides for both, whether the line is drawn, with
/// `is_silhouette` of [`crate::SILHOUETTE_WGSL`]. The normals of contour edges are zero, so
/// that they are always on the silhouette.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EdgeLines {
    /// The positions of the vertices, two of each line.
    pub positions: Vec<Vec3>,
    /// The normal of the first triangle of the edge of each vertex.
    pub normals_a: Vec<Vec3>,
    /// The normal of the second triangle of the edge of each vertex.
    pub normals_b: Vec<Vec3>,
    /// The midpoint of the edge of each vertex.
    pub midpoints: Vec<Vec3>,
}

impl EdgeLines {
    /// The number of lines.
    pub fn len(&self) -> usize {
        self.positions.len() / 2
    }

    /// Whether there are no lines.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Add the line from `a` to `b` between triangles with the normals `normal_a` and `normal_b`.
    fn push(&mut self, [a, b]: [Vec3; 2], normal_a: Vec3, normal_b: Vec3) {
        let midpoint = (a + b) / 2.0;
        self.positions.extend([a, b]);
        self.normals_a.extend([normal_a; 2]);
        self.normals_b.extend([normal_b; 2]);
        self.midpoints.extend([midpoint; 2]);
    }
}

/// The cosine of the angle between two normals, above which their edge is flat and never on the
/// silhouette, except seen exactly from the side.
const FLAT_COS: f32 = 0.9999;

/// The edge lines of `mesh`: the contour edges and the smooth edges, which are on the
/// silhouette depending on the view, e.g. the left and right side of a cylinder.
///
/// An edge is a contour, if the normals of its two triangles differ more than
/// `crease_angle_degrees`, or if it has not exactly two triangles, e.g. at an open border. An
/// edge is smooth, if its two triangles differ less, but are not flat. Vertices at the same
/// position are the same.
///
/// A triangle without area, e.g. with three points on a line, has no normal of its own. If it
/// bridges its neighbors, i.e. its longest edge has one triangle and its other edges are next
/// to other triangles, it gets the normal of the triangle at its longest edge. Then its edges
/// are no open borders, which would be drawn as stray lines, e.g. across a flat face. Other
/// triangles without area are ignored.
pub fn edge_lines(mesh: &TriangleMesh, crease_angle_degrees: f64) -> EdgeLines {
    let edges = adjacency(mesh);
    let cos_threshold = crease_angle_degrees.to_radians().cos() as f32;
    let mut lines = EdgeLines::default();
    for edge in &edges.edges {
        let points = edge.vertices.map(|v| position(mesh, v));
        let [n1, n2] = edge.triangles.map(|t| edges.normals[t as usize]);
        let cos = n1.dot(n2);
        if edge.count != 2 || cos < cos_threshold {
            lines.push(points, Vec3::ZERO, Vec3::ZERO);
        } else if cos <= FLAT_COS {
            lines.push(points, n1, n2);
        }
    }
    lines
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

    /// The contour edges of `mesh` as pairs of positions, the edge lines with zero normals.
    fn contour_edges(mesh: &TriangleMesh, crease_angle_degrees: f64) -> Vec<[Vec3; 2]> {
        let lines = edge_lines(mesh, crease_angle_degrees);
        lines
            .positions
            .chunks(2)
            .zip(lines.normals_a.chunks(2))
            .filter(|(_, normals)| normals[0] == Vec3::ZERO)
            .map(|(points, _)| [points[0], points[1]])
            .collect()
    }

    /// Whether `edges` contain the edge between the vertices `a` and `b` of `mesh`.
    fn contains(mesh: &TriangleMesh, edges: &[[Vec3; 2]], [a, b]: [u32; 2]) -> bool {
        let [a, b] = [a, b].map(|v| position(mesh, v));
        edges.contains(&[a, b]) || edges.contains(&[b, a])
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
        let strip = strip_with_triangle_without_area(0.0);
        let edges = contour_edges(&strip, 20.0);
        // Only the open border: 0-4, 4-2, 0-3, 3-2, and not 0-1, 1-2 or 0-2 across the strip.
        assert_eq!(edges.len(), 4, "{edges:?}");
        for inner in [[0, 1], [1, 2], [0, 2]] {
            assert!(!contains(&strip, &edges, inner), "{inner:?} in {edges:?}");
        }
    }

    #[test]
    fn creases_at_triangles_without_area() {
        // The top half is bent upwards, the edges along the bend are drawn at the vertices of
        // the bottom half, the long edge of the triangle without area is not.
        let strip = strip_with_triangle_without_area(2.0);
        let edges = contour_edges(&strip, 20.0);
        assert!(
            contains(&strip, &edges, [0, 1]) && contains(&strip, &edges, [1, 2]),
            "{edges:?}"
        );
        assert!(!contains(&strip, &edges, [0, 2]), "{edges:?}");
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

    #[test]
    fn edge_lines_of_a_curved_strip() {
        // Four triangles, the second pair bent by 10° like a part of a cylinder.
        let (c, s) = (10f32.to_radians().cos(), 10f32.to_radians().sin());
        let strip = mesh(
            &[
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0 + c, 0.0, s],
                [1.0 + c, 1.0, s],
            ],
            &[[0, 2, 1], [1, 2, 3], [2, 4, 3], [3, 4, 5]],
        );
        let lines = edge_lines(&strip, CREASE_ANGLE_DEGREES);
        // The open border with zero normals, the bent edge 2-3, but not the flat diagonals.
        let border = contour_edges(&strip, CREASE_ANGLE_DEGREES).len();
        assert_eq!(border, 6);
        assert_eq!(lines.len(), border + 1);
        let smooth: Vec<_> = (0..lines.positions.len())
            .filter(|&i| lines.normals_a[i] != Vec3::ZERO)
            .collect();
        let smooth: [usize; 2] = smooth.try_into().expect("one smooth line");
        let positions = smooth.map(|i| lines.positions[i]);
        assert_eq!(positions, [Vec3::X, Vec3::new(1.0, 1.0, 0.0)]);
        let [m1, m2] = smooth.map(|i| lines.midpoints[i]);
        assert_eq!(m1, m2);
        let i = smooth[0];
        let angle = lines.normals_a[i].angle_between(lines.normals_b[i]);
        assert!((angle.to_degrees() - 10.0).abs() < 1e-3);
    }

    #[test]
    fn no_silhouette_edges_at_creases() {
        // The edges of a tetrahedron are sharp, they are always drawn contours.
        let tetrahedron = mesh(
            &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            &[[0, 2, 1], [0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let lines = edge_lines(&tetrahedron, CREASE_ANGLE_DEGREES);
        assert_eq!(lines.len(), 6);
        assert!(
            lines
                .normals_a
                .iter()
                .chain(&lines.normals_b)
                .all(|n| *n == Vec3::ZERO)
        );
    }
}
