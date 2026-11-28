// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_builtin::mu::{self, ops::Placement};
use mucad_core::{self as core, CalcBounds2D, DMat4, DVec3, Extrude, Geometry, GeometryType, Transformed2D, Transformed3D, UnaryBooleanOp};
use mucad_lang_types::{List, math::IntoFloat, model::ModelType};

use crate::{
    GeometryOutput, GeometryOutputs, GeometryTree, Render, RenderContext, RenderError, RenderResult,
};

fn boolean_op(
    tree: &GeometryTree,
    op: core::BooleanOp,
    context: &mut RenderContext,
    preserve_colors: bool,
) -> RenderResult {
    let outputs = context.collect_outputs(tree).primary();
    use GeometryType::*;

    // A union normally produces a single geometry output. Keep independently
    // colored parts separate by unioning each color group on its own.
    if preserve_colors {
        let mut groups: Vec<(Option<core::Color>, Vec<core::Geometry>)> = Vec::new();
        for output in outputs {
            let color = output.attr.fill_color;
            let index = groups
                .iter()
                .position(|(group_color, _)| *group_color == color)
                .unwrap_or_else(|| {
                    groups.push((color, Vec::new()));
                    groups.len() - 1
                });
            groups[index].1.push(output.geometry.clone());
        }

        return Ok(groups
            .into_iter()
            .flat_map(|(color, geometries)| {
                let geometry = match geometries.first().map(core::Geometry::ty) {
                    Some(Geometry2D) => {
                        let geometries = geometries
                            .into_iter()
                            .filter_map(|geometry| match geometry {
                                Geometry::Geometry2D(geometry) => Some(geometry),
                                Geometry::Geometry3D(_) => None,
                            })
                            .collect();
                        Geometry::geo2d(core::Geometries2D::new(geometries).union())
                    }
                    Some(Geometry3D) => {
                        let geometries = geometries
                            .into_iter()
                            .filter_map(|geometry| match geometry {
                                Geometry::Geometry2D(_) => None,
                                Geometry::Geometry3D(geometry) => Some(geometry),
                            })
                            .collect();
                        Geometry::geo3d(core::Geometries3D::new(geometries).union())
                    }
                    _ => return None,
                };
                let mut output = GeometryOutput::from(geometry);
                if let Some(color) = color {
                    output.attr = output.attr.with_color(color);
                }
                Some(output.into())
            })
            .collect());
    }

    Ok(match outputs.ty() {
        Geometry2D => Geometry::geo2d(outputs.to_2d().boolean_op(op)).into(),
        Geometry3D => Geometry::geo3d(outputs.to_3d().boolean_op(op)).into(),
        Empty => GeometryOutputs::default(),
        Mixed => todo!("Error handling: Mixed geometry"),
    })
}

impl Render for mu::ops::Union {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        boolean_op(tree, core::BooleanOp::Union, context, true)
    }
}

impl Render for mu::ops::Subtract {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        boolean_op(tree, core::BooleanOp::Subtract, context, false)
    }
}

impl Render for mu::ops::Intersect {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        boolean_op(tree, core::BooleanOp::Intersect, context, false)
    }
}

impl Render for mu::ops::Hull {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        let outputs = context.collect_outputs(tree).primary();
        use GeometryType::*;
        use core::geo2d::ConvexHull;

        Ok(match outputs.ty() {
            Geometry2D => Geometry::geo2d(outputs.to_2d().union().convex_hull()).into(),
            Geometry3D => Geometry::geo3d(outputs.to_3d().union().convex_hull()).into(),
            Empty => GeometryOutputs::default(),
            Mixed => todo!("Error handling: Mixed geometry"),
        })
    }
}

impl Render for mu::ops::Extrude {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        let multi_polygon = context.collect_outputs(tree).primary().to_2d().union();

        Ok(GeometryOutput::from(multi_polygon.linear_extrude(
            mucad_core::Length::mm(self.height.as_mm()),
            self.scale_x.into_float(),
            self.scale_y.into_float(),
            core::Angle(self.twist.into_float()),
        ))
        .into())
    }
}

impl Render for mu::ops::Revolve {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        let multi_polygon = context.collect_outputs(tree).primary().to_2d().union();
        let radius = multi_polygon.calc_bounds_2d().radius();

        Ok(GeometryOutput::from(multi_polygon.revolve_extrude(
            core::Angle(self.angle.into_float()),
            context.current_resolution(tree).circular_segments(radius) as usize,
        ))
        .into())
    }
}

impl Render for mu::ops::Buffer {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::geo2d::Buffer;
        let multi_polygon = context.collect_outputs(tree).primary().to_2d().union();

        Ok(GeometryOutput::geo2d(multi_polygon.buffer(self.distance.as_mm())).into())
    }
}

impl Render for mu::ops::Center {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::traits::Center;
        let outputs = context.collect_outputs(tree).primary();
        use GeometryType::*;

        Ok(match outputs.ty() {
            Geometry2D => Geometry::geo2d(outputs.to_2d()).center().into(),
            Geometry3D => Geometry::geo3d(outputs.to_3d()).center().into(),
            Empty => GeometryOutputs::default(),
            Mixed => todo!("Error handling: Mixed geometry"),
        })
    }
}

/// Apply `geo2d` or `geo3d` to the geometries of the children, depending on their type.
fn map_geometries(
    tree: &GeometryTree,
    context: &RenderContext,
    geo2d: impl FnOnce(core::Geometries2D) -> core::Geometries2D,
    geo3d: impl FnOnce(core::Geometries3D) -> core::Geometries3D,
) -> RenderResult<GeometryOutputs> {
    let outputs = context.collect_outputs(tree).primary();
    use GeometryType::*;

    match outputs.ty() {
        Geometry2D => Ok(Geometry::geo2d(geo2d(outputs.to_2d())).into()),
        Geometry3D => Ok(Geometry::geo3d(geo3d(outputs.to_3d())).into()),
        Empty => Ok(GeometryOutputs::default()),
        Mixed => Err(RenderError::InvalidOutputType(ModelType::Any)),
    }
}

impl Render for mu::ops::Align {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::{Align2D, Align3D};
        let (x, y, z) = (
            self.x.into_float(),
            self.y.into_float(),
            self.z.into_float(),
        );
        let spacing = core::Length::mm(self.spacing.as_mm());

        map_geometries(
            tree,
            context,
            |geometries| geometries.align_2d(core::DVec2::new(x, y), spacing),
            |geometries| geometries.align_3d(core::DVec3::new(x, y, z), spacing),
        )
    }
}

impl Render for mu::ops::DistributeGrid {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::traits::DistributeGrid;
        let (x, y) = (self.x.as_mm(), self.y.as_mm());
        let rect = core::Rect::new((x, y), (x + self.width.as_mm(), y + self.height.as_mm()));
        let (rows, columns) = (self.rows.to_num(), self.columns.to_num());

        map_geometries(
            tree,
            context,
            |geometries| geometries.distribute_grid(rect, rows, columns),
            |geometries| geometries.distribute_grid(rect, rows, columns),
        )
    }
}

/// The line (2D) or plane (3D) of a mirror or reflection through a point with a normal.
fn mirror_plane(
    (x, y, z): (core::Scalar, core::Scalar, core::Scalar),
    (nx, ny, nz): (core::Scalar, core::Scalar, core::Scalar),
) -> (core::geo2d::Line, core::geo3d::Plane) {
    use core::geo2d::Point;
    (
        core::geo2d::Line(Point::new(x, y), Point::new(x - ny, y + nx)),
        core::geo3d::Plane {
            p: core::DVec3::new(x, y, z),
            n: core::DVec3::new(nx, ny, nz),
        },
    )
}

impl Render for mu::ops::Mirror {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::{Mirror2D, Mirror3D};
        let (line, plane) = mirror_plane(
            (self.x.as_mm(), self.y.as_mm(), self.z.as_mm()),
            (
                self.nx.into_float(),
                self.ny.into_float(),
                self.nz.into_float(),
            ),
        );

        map_geometries(
            tree,
            context,
            |geometries| geometries.mirror_2d(&line),
            |geometries| geometries.mirror_3d(&plane),
        )
    }
}

impl Render for mu::ops::Reflect {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        use core::{Reflect2D, Reflect3D};
        let (line, plane) = mirror_plane(
            (self.x.as_mm(), self.y.as_mm(), self.z.as_mm()),
            (
                self.nx.into_float(),
                self.ny.into_float(),
                self.nz.into_float(),
            ),
        );

        map_geometries(
            tree,
            context,
            |geometries| geometries.reflect_2d(&line),
            |geometries| geometries.reflect_3d(&plane),
        )
    }
}

impl Render for mu::ops::Spiralize {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        let multi_polygon = context.collect_outputs(tree).primary().to_2d().union();
        let mut bounds = multi_polygon.calc_bounds_2d();
        bounds.extend_by_point(core::DVec2::new(0.0, 0.0));
        // The profile is moved away from the axis by up to the larger radius.
        let radius = bounds.max_extent() + self.inner_radius.as_mm().max(self.outer_radius.as_mm());
        let angle = core::Angle(self.angle.into_float());

        Ok(GeometryOutput::from(multi_polygon.spiralize(
            self.height.as_mm(),
            self.inner_radius.as_mm(),
            self.outer_radius.as_mm(),
            angle.0 / std::f64::consts::TAU,
            context.current_resolution(tree).circular_segments(radius) as usize,
        ))
        .into())
    }
}

impl Render for mu::ops::Multiply {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        let outputs = context.collect_outputs(tree).primary();
        let n: usize = self.n.to_num::<i64>().max(0) as usize;

        Ok(GeometryOutputs::from_iter(
            (0..n).flat_map(|_| outputs.iter().cloned()),
        ))
    }
}

//impl From<&Placement> for DMat4 {
    /// Converts this Placement into a right-handed column-major 4x4 transform matrix:
    ///   local +X -> tangent
    ///   local +Y -> cross(normal, tangent)  (right-handed binormal)
    ///   local +Z -> normal
    fn from(&Placement{position, tangent, normal, scale, ..}: &Placement) -> DMat4 {
    	use mucad_lang_types::math::FromFixed;
        let x = DVec3::from_fixed(tangent).normalize_or(DVec3::X);
        // Orthogonalize `normal` against `x`, with safe fallback if colinear/zero
        let z = DVec3::from_fixed(normal)
            .reject_from(x)
            .try_normalize()
            .unwrap_or_else(|| x.any_orthonormal_vector());
        let y = z.cross(x);
        let scale = f64::from_fixed(scale);
        DMat4::from_cols(
            (x * scale).extend(0.),
            (y * scale).extend(0.),
            (z * scale).extend(0.),
            DVec3::from_fixed(position).extend(1.),
        )
    }
//}

impl Render for mu::ops::Instantiate {
    fn render(
        &self,
        tree: &GeometryTree,
        context: &mut RenderContext,
    ) -> RenderResult<GeometryOutputs> {
        Ok(
            self.inner.iter()
            	.map(|value| Placement::try_from(value).unwrap())
             	.flat_map(|ref p| map_geometries(tree, context,
	                |g| core::Geometries2D::from_iter(g.iter().map(|geometry| geometry.transformed_2d(todo!()))),
	                |g| core::Geometries3D::from_iter(g.iter().map(|geometry| geometry.transformed_3d(&from(p)))),
	            ).unwrap())
              	.collect()
        )
    }
}
