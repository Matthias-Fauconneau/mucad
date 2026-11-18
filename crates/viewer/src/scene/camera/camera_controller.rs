// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The camera controller plugin: navigates an orthographic 3D camera with the mouse, the keys
//! and touch by the [`Navigation`] of `mucad-viewer-utils`, configurable with [`Controls`].
//!
//! To use it, add the [`CameraControllerPlugin`] and attach a [`CameraNavigation`] to an entity
//! with a [`Camera3d`] and an orthographic [`Projection`]. The plugin sets the [`Transform`] and
//! the projection of the camera in each frame. A click or a tap sends a [`SelectAt`] message.

use bevy::{
    camera::{CameraProjection, ScalingMode},
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
        mouse::{AccumulatedMouseMotion, MouseButtonInput, MouseScrollUnit, MouseWheel},
        touch::{TouchInput, TouchPhase},
    },
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use mucad_viewer_utils::{
    CameraControl, CameraController, Command, Controls, Modifiers, Navigation,
    OrthoCameraController, TouchId,
};

/// The pixels of a line of the mouse wheel, e.g. of one notch, like one notch in browsers.
const LINE_PIXELS: f32 = 100.0;

/// The far plane of the projection in radii of the bounds, behind the camera at 10 radii.
const FAR_RADII: f32 = 20.0;

/// The camera controller plugin, see the [module](self).
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SelectAt>().add_systems(
            Update,
            (
                resize,
                mouse_buttons,
                mouse_motion,
                mouse_wheel,
                keys,
                touches,
                apply_camera,
                grab_cursor,
            )
                .chain()
                .in_set(CameraControllerSystems),
        );
    }
}

/// The systems of the [`CameraControllerPlugin`], e.g. to change the controller before them.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CameraControllerSystems;

/// The navigation of a camera, see the [module](self).
#[derive(Component, Debug, Clone)]
pub struct CameraNavigation(pub Navigation);

impl CameraNavigation {
    /// The navigation for a window of `window_size` in logical pixels with `controls`.
    pub fn new(window_size: Vec2, controls: Controls) -> Self {
        Self(Navigation::new(window_size.as_dvec2(), controls))
    }

    /// The camera controller.
    pub fn controller(&self) -> &CameraController {
        self.0.controller()
    }

    /// The camera controller, e.g. to set its bounds.
    pub fn controller_mut(&mut self) -> &mut CameraController {
        self.0.controller_mut()
    }
}

/// A click or a tap, which selects the object under it, e.g. with picking.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct SelectAt {
    /// The position in logical pixels from the top left of the window.
    pub position: Vec2,
    /// Whether the object is added to the selection, instead of replacing it.
    pub multi: bool,
}

impl From<Command> for SelectAt {
    fn from(command: Command) -> Self {
        match command {
            Command::Select { position, multi } => Self {
                position: position.as_vec2(),
                multi,
            },
        }
    }
}

/// The pressed modifier keys.
fn modifiers(keys: &ButtonInput<KeyCode>) -> Modifiers {
    Modifiers {
        ctrl: keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]),
        shift: keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        alt: keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]),
        meta: keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]),
    }
}

/// The button of the navigation.
fn mouse_button(button: MouseButton) -> Option<mucad_viewer_utils::MouseButton> {
    match button {
        MouseButton::Left => Some(mucad_viewer_utils::MouseButton::Left),
        MouseButton::Middle => Some(mucad_viewer_utils::MouseButton::Middle),
        MouseButton::Right => Some(mucad_viewer_utils::MouseButton::Right),
        _ => None,
    }
}

/// The name of a logical key like `KeyboardEvent.key` of the web, e.g. `ArrowLeft` or `+`.
fn key_name(key: &Key) -> Option<String> {
    match key {
        Key::Character(character) => Some(character.to_string()),
        Key::Space => Some(" ".into()),
        Key::Unidentified(_) | Key::Dead(_) => None,
        // The names of the other keys are the ones of the web, e.g. `ArrowLeft` or `PageUp`.
        named => Some(format!("{named:?}")).filter(|name| !name.contains('(')),
    }
}

/// The position of the pointer in the window, or the last one, e.g. while it is locked.
fn pointer_position(windows: &Query<&Window, With<PrimaryWindow>>, last: &mut Vec2) -> Vec2 {
    if let Ok(window) = windows.single()
        && let Some(position) = window.cursor_position()
    {
        *last = position;
    }
    *last
}

/// Set the size of the window, e.g. after resizing it.
fn resize(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut navigations: Query<&mut CameraNavigation>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let size = window.size().as_dvec2();
    for mut navigation in &mut navigations {
        if navigation.controller().window_size() != size {
            navigation.0.resize(size);
        }
    }
}

/// Press and release the buttons of the mouse, a click selects.
fn mouse_buttons(
    mut inputs: MessageReader<MouseButtonInput>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut last_position: Local<Vec2>,
    mut navigations: Query<&mut CameraNavigation>,
    mut selections: MessageWriter<SelectAt>,
) {
    let modifiers = modifiers(&keys);
    let position = pointer_position(&windows, &mut last_position).as_dvec2();
    for input in inputs.read() {
        let Some(button) = mouse_button(input.button) else {
            continue;
        };
        for mut navigation in &mut navigations {
            match input.state {
                ButtonState::Pressed => navigation.0.mouse_pressed(button, modifiers, position),
                ButtonState::Released => {
                    if let Some(command) = navigation.0.mouse_released(button, modifiers, position)
                    {
                        selections.write(command.into());
                    }
                }
            }
        }
    }
}

/// Move the mouse by the movement of the pointer in this frame, so that panning follows it
/// exactly, or by the motion of the mouse, while the pointer is locked.
///
/// The motion of the mouse is not used otherwise, because it is without the acceleration of
/// the pointer.
fn mouse_motion(
    motion: Res<AccumulatedMouseMotion>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut last_position: Local<Option<Vec2>>,
    time: Res<Time>,
    mut navigations: Query<&mut CameraNavigation>,
) {
    let position = windows
        .single()
        .ok()
        .and_then(|window| window.cursor_position());
    let previous = std::mem::replace(&mut *last_position, position);
    let pointer_delta = match (previous, position) {
        (Some(previous), Some(position)) => position - previous,
        _ => Vec2::ZERO,
    };
    for mut navigation in &mut navigations {
        let delta = if navigation.controller().pointer_pinned() {
            motion.delta
        } else {
            pointer_delta
        };
        if delta != Vec2::ZERO {
            navigation.0.mouse_moved(delta, time.delta());
        }
    }
}

/// Turn the wheel, a positive delta of Bevy is away from the user.
fn mouse_wheel(
    mut inputs: MessageReader<MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut last_position: Local<Vec2>,
    time: Res<Time>,
    mut navigations: Query<&mut CameraNavigation>,
) {
    let modifiers = modifiers(&keys);
    let position = pointer_position(&windows, &mut last_position).as_dvec2();
    for input in inputs.read() {
        let delta = match input.unit {
            MouseScrollUnit::Line => input.y * LINE_PIXELS,
            MouseScrollUnit::Pixel => input.y,
        };
        for mut navigation in &mut navigations {
            navigation.0.wheel(delta, modifiers, position, time.delta());
        }
    }
}

/// Press the keys, also repeated ones, e.g. to pan with an arrow key.
fn keys(
    mut inputs: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut navigations: Query<&mut CameraNavigation>,
) {
    let modifiers = modifiers(&keys);
    for input in inputs.read() {
        if input.state != ButtonState::Pressed {
            continue;
        }
        let Some(name) = key_name(&input.logical_key) else {
            continue;
        };
        for mut navigation in &mut navigations {
            navigation.0.key_pressed(&name, modifiers);
        }
    }
}

/// Start, move and end the touches, the moves of a frame together, so that moving two fingers
/// in parallel pans without zooming. A tap selects.
fn touches(
    mut inputs: MessageReader<TouchInput>,
    time: Res<Time>,
    mut navigations: Query<&mut CameraNavigation>,
    mut selections: MessageWriter<SelectAt>,
) {
    let mut moved: Vec<(TouchId, bevy::math::DVec2)> = Vec::new();
    let move_touches = |moved: &mut Vec<_>, navigations: &mut Query<&mut CameraNavigation>| {
        if moved.is_empty() {
            return;
        }
        for mut navigation in navigations.iter_mut() {
            navigation.0.touches_moved(moved, time.delta());
        }
        moved.clear();
    };
    for input in inputs.read() {
        let id = input.id as TouchId;
        let position = input.position.as_dvec2();
        if input.phase == TouchPhase::Moved {
            moved.retain(|(touch, _)| *touch != id);
            moved.push((id, position));
            continue;
        }
        move_touches(&mut moved, &mut navigations);
        for mut navigation in &mut navigations {
            match input.phase {
                TouchPhase::Started => navigation.0.touch_started(id, position),
                TouchPhase::Ended => {
                    if let Some(command) = navigation.0.touch_ended(id, position) {
                        selections.write(command.into());
                    }
                }
                TouchPhase::Canceled => navigation.0.touch_cancelled(id),
                TouchPhase::Moved => {}
            }
        }
    }
    move_touches(&mut moved, &mut navigations);
}

/// Apply the camera of the controller to the transform and the projection.
fn apply_camera(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<(&CameraNavigation, &mut Transform, &mut Projection)>,
) {
    let window_size = windows.single().map_or(Vec2::ONE, |window| window.size());
    for (navigation, mut transform, mut projection) in &mut cameras {
        let controller = navigation.controller();
        *transform = Transform::from_matrix(controller.camera().transform_matrix());
        match controller {
            CameraController::Ortho(controller) => {
                if !matches!(*projection, Projection::Orthographic(_)) {
                    *projection = Projection::from(OrthographicProjection::default_3d());
                }
                if let Projection::Orthographic(ortho) = projection.as_mut() {
                    set_orthographic(ortho, controller);
                    ortho.update(window_size.x, window_size.y);
                }
            }
            CameraController::Perspective(controller) => {
                let (near, far) = controller.near_far();
                *projection = Projection::Perspective(PerspectiveProjection {
                    fov: controller.fov(),
                    aspect_ratio: window_size.x / window_size.y,
                    near,
                    far,
                    ..Default::default()
                });
            }
        }
    }
}

/// Set an orthographic projection of Bevy to the detail of the zoom of `controller`.
///
/// The area of the projection is the detail: its scale is the inverse zoom level, like
/// before, its size the overview and its origin the offset of the detail.
fn set_orthographic(ortho: &mut OrthographicProjection, controller: &OrthoCameraController) {
    let zoom_level = controller.zoom_level() as f32;
    let detail = controller.zoom_model().detail();
    let (width, height) = (detail.width as f32, detail.height as f32);
    ortho.scale = 1.0 / zoom_level;
    ortho.scaling_mode = ScalingMode::Fixed {
        width: width * zoom_level,
        height: height * zoom_level,
    };
    ortho.viewport_origin = Vec2::new(-detail.left() as f32 / width, -detail.top() as f32 / height);
    ortho.near = 0.0;
    ortho.far = FAR_RADII * controller.bounds().radius as f32;
}

/// Lock and hide the pointer while orbiting or zooming with the mouse.
fn grab_cursor(
    navigations: Query<&CameraNavigation>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let pinned = navigations
        .iter()
        .any(|navigation| navigation.controller().pointer_pinned());
    let grab_mode = if pinned {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    for mut cursor in &mut cursors {
        if cursor.grab_mode != grab_mode {
            cursor.grab_mode = grab_mode;
            cursor.visible = !pinned;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_names() {
        assert_eq!(key_name(&Key::ArrowLeft).as_deref(), Some("ArrowLeft"));
        assert_eq!(key_name(&Key::PageUp).as_deref(), Some("PageUp"));
        assert_eq!(key_name(&Key::Character("+".into())).as_deref(), Some("+"));
        assert_eq!(key_name(&Key::Space).as_deref(), Some(" "));
        assert_eq!(key_name(&Key::Dead(None)), None);
    }

    /// The area of the projection is the detail of the zoom.
    #[test]
    fn orthographic() {
        let mut navigation = CameraNavigation::new(Vec2::new(400.0, 200.0), Controls::default());
        navigation.0.key_pressed("PageUp", Modifiers::NONE);
        navigation.0.key_pressed("ArrowLeft", Modifiers::NONE);
        let mut ortho = OrthographicProjection::default_3d();
        let controller = navigation.controller().ortho().expect("Orthographic");
        set_orthographic(&mut ortho, controller);
        ortho.update(400.0, 200.0);
        let detail = controller.zoom_model().detail();
        let area = ortho.area;
        assert!((area.min.x as f64 - detail.left()).abs() < 1e-3);
        assert!((area.max.x as f64 - detail.right()).abs() < 1e-3);
        assert!((area.min.y as f64 - detail.top()).abs() < 1e-3);
        assert!((area.max.y as f64 - detail.bottom()).abs() < 1e-3);
        assert!((ortho.scale - 1.0 / 1.25).abs() < 1e-6);
    }

    /// The transform of the camera is the one of the controller.
    #[test]
    fn transform() {
        let navigation = CameraNavigation::new(Vec2::new(400.0, 200.0), Controls::default());
        let camera = navigation.controller().camera();
        let transform = Transform::from_matrix(camera.transform_matrix());
        assert!(transform.translation.abs_diff_eq(camera.position(), 1e-3));
        let forward = *transform.forward();
        assert!(forward.abs_diff_eq(camera.view_vector().normalize(), 1e-5));
    }
}
