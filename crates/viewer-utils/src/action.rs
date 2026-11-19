// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The actions of the viewers with their bindings and help, e.g. for tooltips and a help in
//! Markdown, which the Bevy and the web viewer show the same.

use std::{any::Any, collections::BTreeMap, error::Error, fmt, sync::Arc};

use serde::{Serialize, Serializer, ser::SerializeMap};
use strum::{EnumMessage, EnumProperty, IntoEnumIterator};

use crate::{
    KeyBinding, KeyCommand, Modifiers, MouseBinding, MouseButton, MouseCommand, ParseError,
};

/// A binding of a key or of the mouse: its modifiers and its input joined by `+`, e.g.
/// `Ctrl+Plus` or `Ctrl+Shift+Right`.
///
/// Modifiers: `Ctrl`, `Shift`, `Alt` (`Option`), `Meta` (`Cmd`, `Super`), which must be pressed
/// exactly.
///
/// Keys are logical keys, like `KeyboardEvent.key` of the web: named keys like `ArrowLeft`,
/// `PageUp`, or characters like `0` or `a`, with `Plus`, `Minus` and `Space` for `+`, `-` and
/// ` `. Shift is ignored for characters, because keyboard layouts need it for some of them.
///
/// Mouse buttons: `Left`, `Middle` (pressing the wheel) and `Right`, which are pressed in their
/// order, e.g. `Middle+Right` is first the middle and then the right button. `Wheel` is turning
/// the wheel, which only zooms. A mouse binding, which selects and drags, e.g. `Left`, selects
/// by clicking and drags, when the mouse moves more than a few pixels.
#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    /// A key with modifiers.
    Key(KeyBinding),
    /// Mouse buttons or the wheel with modifiers.
    Mouse(MouseBinding),
}

/// The input of the bindings of an action: keys or the mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    /// Keys, e.g. `Ctrl+Plus`.
    Key,
    /// Mouse buttons or the wheel, e.g. `Middle+Right`.
    Mouse,
}

impl Binding {
    /// Parse `binding` of `kind`, e.g. `Left` is the left mouse button and not a key.
    pub fn parse(kind: BindingKind, binding: &str) -> Result<Self, ParseError> {
        match kind {
            BindingKind::Key => binding.parse().map(Self::Key),
            BindingKind::Mouse => binding.parse().map(Self::Mouse),
        }
    }

    /// The key binding, if it is one.
    pub fn as_key(&self) -> Option<&KeyBinding> {
        match self {
            Self::Key(binding) => Some(binding),
            Self::Mouse(_) => None,
        }
    }

    /// The mouse binding, if it is one.
    pub fn as_mouse(&self) -> Option<&MouseBinding> {
        match self {
            Self::Mouse(binding) => Some(binding),
            Self::Key(_) => None,
        }
    }
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(binding) => binding.fmt(f),
            Self::Mouse(binding) => binding.fmt(f),
        }
    }
}

/// What the user can do in a viewer, e.g. zoom in, with its default bindings and help.
///
/// The [`ActionRegistry`] of a viewer holds its actions with their current bindings. A viewer
/// finds the action of an input, e.g. with [`ActionRegistry::key_action`], and downcasts it to
/// its type as a [`dyn Any`](Any), e.g. to a [`KeyCommand`].
pub trait Action: Any + Send + Sync {
    /// The name, which is unique in a registry, e.g. `zoom_in` in the `[bindings]` of the
    /// controls.
    fn id(&self) -> &'static str;
    /// The short name, e.g. `Zoom in`.
    fn label(&self) -> &'static str;
    /// What the action does, e.g. `Zoom in at the center of the view.`
    fn description(&self) -> &'static str;
    /// The input of the bindings: keys or the mouse.
    fn kind(&self) -> BindingKind;
    /// The bindings, which the action has without controls, of its [`kind`](Self::kind).
    fn default_bindings(&self) -> Vec<Binding>;

    /// The group in the help, e.g. `Views`, whose consecutive actions share one row.
    fn group(&self) -> Option<&'static str> {
        None
    }

    /// Whether the short help shows the action.
    fn basic(&self) -> bool {
        false
    }
}

impl Action for KeyCommand {
    fn id(&self) -> &'static str {
        self.into()
    }

    fn label(&self) -> &'static str {
        self.get_message().unwrap_or_default()
    }

    fn description(&self) -> &'static str {
        self.get_documentation().unwrap_or_default()
    }

    fn kind(&self) -> BindingKind {
        BindingKind::Key
    }

    fn default_bindings(&self) -> Vec<Binding> {
        use KeyCommand::*;
        let (ctrl, shift, none) = (Modifiers::CTRL, Modifiers::SHIFT, Modifiers::NONE);
        let bindings: &[(Modifiers, &str)] = match self {
            ZoomIn => &[(ctrl, "+"), (none, "PageUp")],
            ZoomOut => &[(ctrl, "-"), (none, "PageDown")],
            ZoomFit => &[(none, "f")],
            ToggleProjection => &[(none, "p")],
            PanLeft => &[(none, "ArrowLeft")],
            PanRight => &[(none, "ArrowRight")],
            PanUp => &[(none, "ArrowUp")],
            PanDown => &[(none, "ArrowDown")],
            RotateLeft => &[(shift, "ArrowLeft")],
            RotateRight => &[(shift, "ArrowRight")],
            ViewIsometric => &[(none, "0")],
            ViewFront => &[(none, "1")],
            ViewTop => &[(none, "2")],
            ViewRight => &[(none, "3")],
            ViewRear => &[(none, "4")],
            ViewBottom => &[(none, "5")],
            ViewLeft => &[(none, "6")],
        };
        bindings
            .iter()
            .map(|(modifiers, key)| Binding::Key(KeyBinding::new(*modifiers, key)))
            .collect()
    }

    fn group(&self) -> Option<&'static str> {
        self.get_str("group")
    }

    fn basic(&self) -> bool {
        use KeyCommand::*;
        matches!(
            self,
            ZoomFit
                | ToggleProjection
                | ViewIsometric
                | ViewFront
                | ViewTop
                | ViewRight
                | ViewRear
                | ViewBottom
                | ViewLeft
        )
    }
}

impl Action for MouseCommand {
    fn id(&self) -> &'static str {
        self.into()
    }

    fn label(&self) -> &'static str {
        self.get_message().unwrap_or_default()
    }

    fn description(&self) -> &'static str {
        self.get_documentation().unwrap_or_default()
    }

    fn kind(&self) -> BindingKind {
        BindingKind::Mouse
    }

    fn default_bindings(&self) -> Vec<Binding> {
        use MouseButton::*;
        let (none, ctrl, shift) = (Modifiers::NONE, Modifiers::CTRL, Modifiers::SHIFT);
        let bindings = match self {
            MouseCommand::Select { .. } => vec![MouseBinding::buttons(none, &[Left])],
            MouseCommand::Orbit => vec![
                MouseBinding::buttons(none, &[Middle, Right]),
                MouseBinding::buttons(none, &[Middle, Left]),
                MouseBinding::buttons(none, &[Right]),
            ],
            MouseCommand::Pan => vec![
                MouseBinding::buttons(none, &[Left]),
                MouseBinding::buttons(none, &[Middle]),
                MouseBinding::buttons(ctrl, &[Right]),
            ],
            MouseCommand::Zoom => vec![
                MouseBinding::wheel(none),
                MouseBinding::buttons(ctrl.union(shift), &[Right]),
            ],
        };
        bindings.into_iter().map(Binding::Mouse).collect()
    }

    fn basic(&self) -> bool {
        true
    }
}

/// An error of the actions of a registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionError {
    /// An action with the id is already registered.
    Duplicate(&'static str),
    /// No action with the id is registered.
    Unknown(String),
    /// A binding of the action with the id is invalid.
    Binding {
        /// The id of the action.
        id: String,
        /// Why the binding is invalid.
        error: ParseError,
    },
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate(id) => write!(f, "The action `{id}` is already registered"),
            Self::Unknown(id) => write!(f, "There is no action `{id}`"),
            Self::Binding { id, error } => write!(f, "Invalid binding of `{id}`: {error}"),
        }
    }
}

impl Error for ActionError {}

/// An action with its current bindings.
#[derive(Clone)]
struct Entry {
    action: Arc<dyn Action>,
    bindings: Vec<Binding>,
}

/// The actions of a viewer with their current bindings in the order of its help.
///
/// It is serialized as the bindings of each action, e.g. `zoom_in = ["Ctrl+Plus", "PageUp"]`
/// for the `[bindings]` of the controls. Cloning is cheap, the actions are shared.
#[derive(Clone, Default)]
pub struct ActionRegistry {
    entries: Vec<Entry>,
}

impl ActionRegistry {
    /// The actions of the navigation with their default bindings: the mouse commands, then the
    /// key commands.
    pub fn navigation() -> Self {
        let mouse = [
            MouseCommand::Select { multi: false },
            MouseCommand::Orbit,
            MouseCommand::Pan,
            MouseCommand::Zoom,
        ];
        let actions = mouse
            .into_iter()
            .map(|command| Arc::new(command) as Arc<dyn Action>)
            .chain(KeyCommand::iter().map(|command| Arc::new(command) as Arc<dyn Action>));
        Self {
            entries: actions
                .map(|action| Entry {
                    bindings: action.default_bindings(),
                    action,
                })
                .collect(),
        }
    }

    /// Register `action` with its default bindings after the others, an error if its id is
    /// already registered.
    pub fn register(&mut self, action: impl Action) -> Result<(), ActionError> {
        if self.get(action.id()).is_some() {
            return Err(ActionError::Duplicate(action.id()));
        }
        self.entries.push(Entry {
            bindings: action.default_bindings(),
            action: Arc::new(action),
        });
        Ok(())
    }

    /// The action with `id`.
    pub fn get(&self, id: &str) -> Option<&dyn Action> {
        self.entry(id).map(|entry| entry.action.as_ref())
    }

    /// The actions in the order of the help.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Action> {
        self.entries.iter().map(|entry| entry.action.as_ref())
    }

    /// The current bindings of the action with `id`, none for an unknown action.
    pub fn bindings(&self, id: &str) -> &[Binding] {
        self.entry(id).map_or(&[], |entry| &entry.bindings)
    }

    /// Replace the bindings of the actions in `bindings` by their ids, e.g. the `bindings` of
    /// the controls, the other actions keep theirs.
    ///
    /// An unknown action or an invalid binding is an error, which keeps all bindings.
    pub fn set_bindings(
        &mut self,
        bindings: &BTreeMap<String, Vec<String>>,
    ) -> Result<(), ActionError> {
        let mut parsed = Vec::new();
        for (id, strings) in bindings {
            let index = self
                .entries
                .iter()
                .position(|entry| entry.action.id() == id)
                .ok_or_else(|| ActionError::Unknown(id.clone()))?;
            let kind = self.entries[index].action.kind();
            let values = strings
                .iter()
                .map(|binding| Binding::parse(kind, binding))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| ActionError::Binding {
                    id: id.clone(),
                    error,
                })?;
            parsed.push((index, values));
        }
        for (index, values) in parsed {
            self.entries[index].bindings = values;
        }
        Ok(())
    }

    /// The first action in the order of the help, which `key` pressed with `modifiers` is
    /// bound to.
    pub fn key_action(&self, key: &str, modifiers: Modifiers) -> Option<&dyn Action> {
        self.entries
            .iter()
            .find(|entry| {
                entry
                    .bindings
                    .iter()
                    .filter_map(Binding::as_key)
                    .any(|binding| binding.matches(key, modifiers))
            })
            .map(|entry| entry.action.as_ref())
    }

    /// The mouse bindings of `command`.
    pub(crate) fn mouse_bindings(
        &self,
        command: MouseCommand,
    ) -> impl Iterator<Item = &MouseBinding> {
        self.bindings(command.id())
            .iter()
            .filter_map(Binding::as_mouse)
    }

    /// The label of the action with `id` with its bindings, e.g. `Zoom to fit (f)`.
    pub fn tooltip(&self, id: &str) -> Result<String, ActionError> {
        let entry = self
            .entry(id)
            .ok_or_else(|| ActionError::Unknown(id.to_string()))?;
        let label = entry.action.label();
        if entry.bindings.is_empty() {
            return Ok(label.to_string());
        }
        let bindings: Vec<String> = entry.bindings.iter().map(ToString::to_string).collect();
        Ok(format!("{label} ({})", bindings.join(", ")))
    }

    /// A Markdown table of the actions with their labels, bindings and descriptions, with
    /// `basic` only of the basic ones.
    ///
    /// The consecutive actions of a group share one row with the group as label, e.g. `Views`,
    /// and each binding followed by the label of its action.
    pub fn help_markdown(&self, basic: bool) -> String {
        let mut help = String::from("| Action | Bindings | Description |\n|---|---|---|\n");
        let entries: Vec<&Entry> = self
            .entries
            .iter()
            .filter(|entry| !basic || entry.action.basic())
            .collect();
        let same_group = |a: &&Entry, b: &&Entry| {
            let group = a.action.group();
            group.is_some() && group == b.action.group()
        };
        for chunk in entries.chunk_by(same_group) {
            let row = match chunk {
                [first, _, ..] => {
                    let bindings: Vec<String> = chunk
                        .iter()
                        .map(|entry| format!("{} {}", code(&entry.bindings), entry.action.label()))
                        .collect();
                    let group = first.action.group().unwrap_or_default();
                    format!("| {group} | {} | |", bindings.join(", "))
                }
                [entry] => format!(
                    "| {} | {} | {} |",
                    entry.action.label(),
                    code(&entry.bindings),
                    entry.action.description()
                ),
                [] => continue,
            };
            help.push_str(&row);
            help.push('\n');
        }
        help
    }

    /// The entry of the action with `id`.
    fn entry(&self, id: &str) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.action.id() == id)
    }
}

/// The bindings as Markdown code, e.g. `` `Ctrl+Plus`, `PageUp` ``.
fn code(bindings: &[Binding]) -> String {
    let code: Vec<String> = bindings
        .iter()
        .map(|binding| format!("`{binding}`"))
        .collect();
    code.join(", ")
}

impl Serialize for ActionRegistry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.entries.len()))?;
        for entry in &self.entries {
            let bindings: Vec<String> = entry.bindings.iter().map(ToString::to_string).collect();
            map.serialize_entry(entry.action.id(), &bindings)?;
        }
        map.end()
    }
}

impl fmt::Debug for ActionRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        for entry in &self.entries {
            let bindings: Vec<String> = entry.bindings.iter().map(ToString::to_string).collect();
            map.entry(&entry.action.id(), &bindings);
        }
        map.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An action of a viewer, which is not one of the navigation.
    struct ToggleGrid;

    impl Action for ToggleGrid {
        fn id(&self) -> &'static str {
            "toggle_grid"
        }

        fn label(&self) -> &'static str {
            "Toggle grid"
        }

        fn description(&self) -> &'static str {
            "Show or hide the grid."
        }

        fn kind(&self) -> BindingKind {
            BindingKind::Key
        }

        fn default_bindings(&self) -> Vec<Binding> {
            vec![Binding::Key(KeyBinding::new(Modifiers::NONE, "g"))]
        }
    }

    /// The bindings of `entries` like in the controls.
    fn bindings(entries: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        entries
            .iter()
            .map(|(id, bindings)| {
                let bindings = bindings.iter().map(|binding| binding.to_string()).collect();
                (id.to_string(), bindings)
            })
            .collect()
    }

    #[test]
    fn actions() {
        let mut registry = ActionRegistry::navigation();
        let zoom_in = registry.get("zoom_in").expect("zoom_in");
        assert_eq!(zoom_in.label(), "Zoom in");
        assert_eq!(zoom_in.description(), "Zoom in at the center of the view.");
        assert_eq!(zoom_in.group(), Some("Zoom"));
        assert_eq!(
            registry.tooltip("zoom_fit").as_deref(),
            Ok("Zoom to fit (f)")
        );
        assert_eq!(
            registry.tooltip("orbit").as_deref(),
            Ok("Orbit (Middle+Right, Middle+Left, Right)")
        );
        assert_eq!(
            registry.tooltip("zoom_sideways"),
            Err(ActionError::Unknown("zoom_sideways".into()))
        );

        registry.register(ToggleGrid).expect("new action");
        assert_eq!(
            registry.register(ToggleGrid),
            Err(ActionError::Duplicate("toggle_grid"))
        );
        assert_eq!(
            registry.tooltip("toggle_grid").as_deref(),
            Ok("Toggle grid (g)")
        );
        assert!(
            registry
                .iter()
                .all(|action| !action.label().is_empty() && !action.description().is_empty())
        );
    }

    #[test]
    fn key_actions() {
        let mut registry = ActionRegistry::navigation();
        registry.register(ToggleGrid).expect("new action");
        let action = registry.key_action("PageDown", Modifiers::NONE);
        let any: Option<&dyn Any> = action.map(|action| action as &dyn Any);
        assert_eq!(
            any.and_then(|any| any.downcast_ref::<KeyCommand>()),
            Some(&KeyCommand::ZoomOut)
        );
        let action = registry.key_action("g", Modifiers::NONE);
        assert_eq!(action.map(Action::id), Some("toggle_grid"));
        assert!(registry.key_action("g", Modifiers::CTRL).is_none());
    }

    #[test]
    fn set_bindings() {
        let mut registry = ActionRegistry::navigation();
        registry.register(ToggleGrid).expect("new action");
        let changed = bindings(&[("toggle_grid", &["Shift+G"]), ("pan", &["Middle"])]);
        registry.set_bindings(&changed).expect("valid bindings");
        assert_eq!(
            registry.tooltip("toggle_grid").as_deref(),
            Ok("Toggle grid (Shift+g)")
        );
        assert_eq!(registry.tooltip("pan").as_deref(), Ok("Pan (Middle)"));
        // The others keep their bindings.
        assert_eq!(
            registry.tooltip("zoom_fit").as_deref(),
            Ok("Zoom to fit (f)")
        );
        // `Left` is the mouse button for the mouse commands.
        assert_eq!(
            registry.bindings("select"),
            [Binding::Mouse(MouseBinding::buttons(
                Modifiers::NONE,
                &[MouseButton::Left]
            ))]
        );

        // Errors keep all bindings.
        let before = format!("{registry:?}");
        let error = registry.set_bindings(&bindings(&[("pan", &["Left"]), ("spin", &["x"])]));
        assert_eq!(error, Err(ActionError::Unknown("spin".into())));
        let error = registry.set_bindings(&bindings(&[("orbit", &["Middle+Up"])]));
        assert!(matches!(error, Err(ActionError::Binding { id, .. }) if id == "orbit"));
        assert_eq!(format!("{registry:?}"), before);
    }

    #[test]
    fn serialize() {
        let registry = ActionRegistry::navigation();
        let toml = toml::to_string(&registry).expect("TOML");
        let lines: Vec<&str> = toml.lines().collect();
        assert_eq!(lines[0], "select = [\"Left\"]");
        assert_eq!(lines[4], "zoom_in = [\"Ctrl+Plus\", \"PageUp\"]");

        // The serialized bindings are the bindings again.
        let parsed: BTreeMap<String, Vec<String>> = toml::from_str(&toml).expect("TOML");
        let mut changed = ActionRegistry::navigation();
        let home = bindings(&[("zoom_fit", &["Home"])]);
        changed.set_bindings(&home).expect("valid");
        changed.set_bindings(&parsed).expect("valid");
        assert_eq!(format!("{changed:?}"), format!("{registry:?}"));
    }

    #[test]
    fn help() {
        let mut registry = ActionRegistry::navigation();
        let home = bindings(&[("zoom_fit", &["Home"])]);
        registry.set_bindings(&home).expect("valid");
        let help = registry.help_markdown(true);
        let rows: Vec<&str> = help.lines().collect();
        assert_eq!(rows[0], "| Action | Bindings | Description |");
        assert_eq!(
            rows[2],
            "| Select | `Left` | Select an object by clicking, with the modifiers of `multi_select` \
             add it to the selection. |"
        );
        assert_eq!(
            rows[6],
            "| Zoom to fit | `Home` | Show the whole scene in the middle of the view. |"
        );
        assert_eq!(
            rows[8],
            "| Views | `0` Isometric view, `1` Front view, `2` Top view, `3` Right view, \
             `4` Rear view, `5` Bottom view, `6` Left view | |"
        );
        assert_eq!(rows.len(), 9);

        // The full help has the groups of the keys, e.g. zooming.
        let help = registry.help_markdown(false);
        assert!(help.contains("| Zoom | `Ctrl+Plus`, `PageUp` Zoom in, "));
    }
}
