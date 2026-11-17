// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests of the camera navigation of the WebAssembly interface, which run natively.

use glam::{Mat4, Vec3};
use mucad_wasm::{CTRL, CameraNavigation, SHIFT};

const WIDTH: f64 = 400.0;
const HEIGHT: f64 = 200.0;
const FRAME_MS: f64 = 16.0;

/// A navigation with the default controls.
fn navigation() -> CameraNavigation {
    let Ok(navigation) = CameraNavigation::new(WIDTH, HEIGHT, None) else {
        panic!("The default controls");
    };
    navigation
}

fn matrix(columns: Vec<f32>) -> Mat4 {
    Mat4::from_cols_slice(&columns)
}

#[test]
fn matrices() {
    let navigation = navigation();
    let camera = matrix(navigation.camera_matrix());
    let view = matrix(navigation.view_matrix());
    assert!((camera * view).abs_diff_eq(Mat4::IDENTITY, 1e-5));
    let position = Vec3::from_slice(&navigation.position());
    assert!(
        camera
            .transform_point3(Vec3::ZERO)
            .abs_diff_eq(position, 1e-3)
    );
    // The scene radius 100 fits into the height of the orthographic projection.
    let projection = matrix(navigation.projection_matrix());
    let top = projection * view * Vec3::new(0.0, 100.0, 0.0).extend(1.0);
    assert!((top.y / top.w - 1.0).abs() < 1e-5);
}

#[test]
fn click_selects() {
    let mut navigation = navigation();
    navigation.mouse_down(0, 10.0, 20.0, CTRL);
    assert_eq!(navigation.mouse_action().as_deref(), Some("select"));
    let selection = navigation.mouse_up(0, 10.0, 20.0, CTRL).expect("A click");
    assert_eq!(
        (selection.x(), selection.y(), selection.multi()),
        (10.0, 20.0, true)
    );
    assert_eq!(navigation.mouse_action(), None);
}

#[test]
fn drag_orbits() {
    let mut navigation = navigation();
    let view = navigation.view_matrix();
    navigation.mouse_down(2, 200.0, 100.0, SHIFT);
    assert_eq!(navigation.mouse_action().as_deref(), Some("orbit"));
    assert!(navigation.pointer_pinned());
    navigation.mouse_move(30.0, 10.0, FRAME_MS);
    assert!(navigation.mouse_up(2, 200.0, 100.0, SHIFT).is_none());
    assert!(!navigation.pointer_wrapped());
    assert_ne!(navigation.view_matrix(), view);
}

#[test]
fn wheel_zooms() {
    let mut navigation = navigation();
    navigation.take_events();
    // Scrolling away from the user, i.e. a negative deltaY, zooms in.
    assert!(navigation.wheel(-3.0, 1, 200.0, 100.0, 0, FRAME_MS));
    assert!(navigation.zoom_level() > 1.0);
    assert_eq!(navigation.take_events(), ["zoomLevelChanged"]);
    // Shift with the wheel is not bound, e.g. to scroll.
    assert!(!navigation.wheel(-3.0, 1, 200.0, 100.0, SHIFT, FRAME_MS));
}

#[test]
fn keys() {
    let mut navigation = navigation();
    assert!(navigation.key_down("PageUp", 0));
    assert!((navigation.zoom_level() - 1.25).abs() < 1e-9);
    assert!(navigation.key_down("2", 0));
    let position = Vec3::from_slice(&navigation.position());
    assert!(position.abs_diff_eq(Vec3::new(0.0, 0.0, 1000.0), 1e-2));
    assert!(!navigation.key_down("x", 0));
}

#[test]
fn views_and_scene() {
    let mut navigation = navigation();
    assert!(navigation.set_view("front").is_ok());
    assert_eq!(navigation.up(), [0.0, 0.0, 1.0]);
    navigation.set_scene(10.0, 0.0, 0.0, 50.0);
    assert_eq!(navigation.view_center(), [10.0, 0.0, 0.0]);
    navigation.key_down("PageUp", 0);
    navigation.zoom_fit();
    assert!((navigation.zoom_level() - 1.0).abs() < 1e-9);
}

#[test]
fn controls() {
    let toml = "[mouse]\npan = [\"Right\"]\n";
    let Ok(mut navigation) = CameraNavigation::new(WIDTH, HEIGHT, Some(toml.into())) else {
        panic!("Valid controls");
    };
    navigation.mouse_down(2, 0.0, 0.0, 0);
    assert_eq!(navigation.mouse_action().as_deref(), Some("pan"));
    assert!(CameraNavigation::default_controls().contains("[keyboard]"));
}
