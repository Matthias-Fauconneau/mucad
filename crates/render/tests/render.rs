// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for rendering model trees.

use mucad_builtin::{BuiltinEvalContext, BuiltinPrimitiveCall, mu};
use mucad_lang_types::{
    Model, ModelTree, Value, arguments,
    model::{Attribute, Attributes, Element},
};
use mucad_render::Renderer;

fn circle(r: f64) -> Model {
    let mut ctx = BuiltinEvalContext::default();
    mu::geo2d::Circle::call(arguments!(radius = Value::mm(r)), &mut ctx).expect("No error")
}

/// Circle(42mm);
#[test]
fn render_circle() {
    let circle = circle(42.0);
    let mut model_tree = ModelTree::new(circle);
    model_tree.compute_hashes();

    let geometry = Renderer::new()
        .render(&model_tree)
        .expect("No render errors")
        .0;

    insta::assert_snapshot!("render_circle", geometry);
}

/// Circle(42mm) - Circle(23mm);
#[test]
fn render_subtract() {
    let mut ctx = BuiltinEvalContext::default();

    // {
    //     Circle(42mm);
    //     Circle(23mm);
    // }
    let mut group = ModelTree::new(Element::Group);
    group.append(circle(42.0));
    group.append(circle(23.0));
    let mut diff = mu::ops::subtract(arguments!(self = group), &mut ctx).expect("No error");
    diff.compute_hashes();

    let geometry = Renderer::new().render(&diff).expect("No render errors").0;

    insta::assert_snapshot!("render_difference", geometry);
}

/// Circle(42mm).extrude(23mm);
#[test]
fn render_extrude() {
    let mut ctx = BuiltinEvalContext::default();

    let mut extrude = mu::ops::extrude(
        arguments!(
            self = circle(42.0),
            height = Value::mm(23.0),
            scale_x = 1.0,
            scale_y = 1.0,
            twist = Value::deg(0.0)
        ),
        &mut ctx,
    )
    .expect("No error");
    extrude.compute_hashes();

    let geometry = Renderer::new()
        .render(&extrude)
        .expect("No render errors")
        .0;

    insta::assert_snapshot!("render_extrude", geometry);
}

/// Union keeps outputs from differently colored parts in separate groups.
#[test]
fn render_union_preserves_part_colors() {
    let mut group = ModelTree::new(Element::Group);
    for (radius, color) in [
        (3.0, mucad_lang_types::Color::rgb(1.0, 0.0, 0.0)),
        (2.0, mucad_lang_types::Color::rgb(0.0, 0.0, 1.0)),
    ] {
        let model = circle(radius).with_attr(Attributes(vec![Attribute::Color(color)]));
        group.append(model);
    }

    let mut ctx = BuiltinEvalContext::default();
    let mut union = mu::ops::union(arguments!(self = group), &mut ctx).expect("No error");
    union.compute_hashes();
    let geometry = Renderer::new().render(&union).expect("No render errors").0;
    let root = geometry.root();
    let outputs: Vec<_> = root.outputs.iter().collect();

    assert_eq!(outputs.len(), 2);
    assert!(
        outputs
            .iter()
            .any(|output| output.attr.fill_color == Some(mucad_core::Color::RED))
    );
    assert!(
        outputs
            .iter()
            .any(|output| output.attr.fill_color == Some(mucad_core::Color::BLUE))
    );
}

/// Parts with the same color still produce one union output.
#[test]
fn render_union_merges_same_color_parts() {
    let mut group = ModelTree::new(Element::Group);
    for radius in [3.0, 2.0] {
        let model = circle(radius).with_attr(Attributes(vec![Attribute::Color(
            mucad_lang_types::Color::rgb(1.0, 0.0, 0.0),
        )]));
        group.append(model);
    }

    let mut ctx = BuiltinEvalContext::default();
    let mut union = mu::ops::union(arguments!(self = group), &mut ctx).expect("No error");
    union.compute_hashes();
    let geometry = Renderer::new().render(&union).expect("No render errors").0;

    assert_eq!(geometry.root().outputs.len(), 1);
    assert_eq!(
        geometry
            .root()
            .outputs
            .iter()
            .next()
            .unwrap()
            .attr
            .fill_color,
        Some(mucad_core::Color::RED)
    );
}
