// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The actions of the viewers with their bindings and help, e.g. for tooltips and a help in
//! Markdown, which the Bevy and the web viewer show the same.

use std::{error::Error, fmt, sync::Arc};

use strum::{EnumMessage, EnumProperty, IntoEnumIterator};

use crate::{Controls, KeyBinding, KeyCommand, MouseBinding, MouseCommand};

/// A binding of a key or of the mouse, written like in the controls, e.g. `Ctrl+Plus`.
#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    /// A key with modifiers.
    Key(KeyBinding),
    /// Mouse buttons or the wheel with modifiers.
    Mouse(MouseBinding),
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(binding) => binding.fmt(f),
            Self::Mouse(binding) => binding.fmt(f),
        }
    }
}

/// What the user can do in a viewer, e.g. zoom in, with its bindings and help.
///
/// The help of a viewer lists the actions of its [`ActionRegistry`].
pub trait Action: Send + Sync {
    /// The name, which is unique in a registry, e.g. `zoom_in` like in the controls.
    fn id(&self) -> &'static str;
    /// The short name, e.g. `Zoom in`.
    fn label(&self) -> &'static str;
    /// What the action does, e.g. `Zoom in at the center of the view.`
    fn description(&self) -> &'static str;
    /// The bindings in `controls`.
    fn bindings(&self, controls: &Controls) -> Vec<Binding>;

    /// The group in the help, e.g. `Views`, whose consecutive actions share one row.
    fn group(&self) -> Option<&'static str> {
        None
    }

    /// Whether the short help shows the action.
    fn basic(&self) -> bool {
        false
    }

    /// The label with the bindings in `controls`, e.g. `Zoom in (Ctrl+Plus, PageUp)`.
    fn tooltip(&self, controls: &Controls) -> String {
        let bindings = self.bindings(controls);
        if bindings.is_empty() {
            return self.label().to_string();
        }
        let bindings: Vec<String> = bindings.iter().map(ToString::to_string).collect();
        format!("{} ({})", self.label(), bindings.join(", "))
    }

    /// A row of the table of [`Help::markdown`]: the label, the bindings in
    /// `controls` as code and the description.
    fn help_markdown(&self, controls: &Controls) -> String {
        format!(
            "| {} | {} | {} |",
            self.label(),
            code(&self.bindings(controls)),
            self.description()
        )
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

    fn bindings(&self, controls: &Controls) -> Vec<Binding> {
        let bindings = controls.keyboard.bindings(*self);
        bindings.iter().cloned().map(Binding::Key).collect()
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

    fn bindings(&self, controls: &Controls) -> Vec<Binding> {
        let bindings = controls.mouse.bindings(*self);
        bindings.iter().cloned().map(Binding::Mouse).collect()
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
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate(id) => write!(f, "The action `{id}` is already registered"),
            Self::Unknown(id) => write!(f, "There is no action `{id}`"),
        }
    }
}

impl Error for ActionError {}

/// The actions of a viewer in the order of its help.
///
/// Cloning is cheap, the actions are shared.
#[derive(Clone, Default)]
pub struct ActionRegistry {
    actions: Vec<Arc<dyn Action>>,
}

impl ActionRegistry {
    /// The actions of the navigation: the mouse commands, then the key commands.
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
            actions: actions.collect(),
        }
    }

    /// Register `action` after the others, an error if its id is already registered.
    pub fn register(&mut self, action: impl Action + 'static) -> Result<(), ActionError> {
        if self.get(action.id()).is_some() {
            return Err(ActionError::Duplicate(action.id()));
        }
        self.actions.push(Arc::new(action));
        Ok(())
    }

    /// The action with `id`.
    pub fn get(&self, id: &str) -> Option<&dyn Action> {
        self.iter().find(|action| action.id() == id)
    }

    /// The actions in the order of the help.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Action> {
        self.actions.iter().map(AsRef::as_ref)
    }

    /// The help of the actions with the bindings in `controls`.
    pub fn help<'a>(&'a self, controls: &'a Controls) -> Help<'a> {
        Help {
            registry: self,
            controls,
        }
    }
}

/// The help of the actions of a registry with the bindings of controls, e.g. for tooltips.
#[derive(Debug, Clone, Copy)]
pub struct Help<'a> {
    registry: &'a ActionRegistry,
    controls: &'a Controls,
}

impl Help<'_> {
    /// The tooltip of the action with `id`, see [`Action::tooltip`].
    pub fn tooltip(&self, id: &str) -> Result<String, ActionError> {
        let action = self.registry.get(id);
        let action = action.ok_or_else(|| ActionError::Unknown(id.to_string()))?;
        Ok(action.tooltip(self.controls))
    }

    /// A Markdown table of the actions, with `basic` only of the basic ones.
    ///
    /// The consecutive actions of a group share one row with the group as label, e.g. `Views`,
    /// and each binding followed by the label of its action.
    pub fn markdown(&self, basic: bool) -> String {
        let controls = self.controls;
        let mut help = String::from("| Action | Bindings | Description |\n|---|---|---|\n");
        let actions: Vec<&dyn Action> = self
            .registry
            .iter()
            .filter(|action| !basic || action.basic())
            .collect();
        for chunk in actions.chunk_by(|a, b| a.group().is_some() && a.group() == b.group()) {
            match chunk {
                [first, _, ..] => {
                    let bindings: Vec<String> = chunk
                        .iter()
                        .map(|action| {
                            format!("{} {}", code(&action.bindings(controls)), action.label())
                        })
                        .collect();
                    let group = first.group().unwrap_or_default();
                    help.push_str(&format!("| {group} | {} | |", bindings.join(", ")));
                }
                [action] => help.push_str(&action.help_markdown(controls)),
                [] => {}
            }
            help.push('\n');
        }
        help
    }
}

impl fmt::Debug for ActionRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.iter().map(|action| action.id()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An action of a viewer, which is not in the controls.
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

        fn bindings(&self, _: &Controls) -> Vec<Binding> {
            vec![Binding::Key(KeyBinding::new(crate::Modifiers::NONE, "g"))]
        }
    }

    #[test]
    fn actions() {
        let controls = Controls::default();
        let mut registry = ActionRegistry::navigation();
        assert_eq!(
            registry.help(&controls).tooltip("zoom_sideways"),
            Err(ActionError::Unknown("zoom_sideways".into()))
        );
        let zoom_in = registry.get("zoom_in").expect("zoom_in");
        assert_eq!(zoom_in.label(), "Zoom in");
        assert_eq!(zoom_in.description(), "Zoom in at the center of the view.");
        assert_eq!(zoom_in.group(), Some("Zoom"));
        assert_eq!(
            registry.help(&controls).tooltip("zoom_fit").as_deref(),
            Ok("Zoom to fit (f)")
        );
        assert_eq!(
            registry.help(&controls).tooltip("orbit").as_deref(),
            Ok("Orbit (Middle+Right, Middle+Left, Right)")
        );

        registry.register(ToggleGrid).expect("new action");
        assert_eq!(
            registry.register(ToggleGrid),
            Err(ActionError::Duplicate("toggle_grid"))
        );
        assert_eq!(
            registry.help(&controls).tooltip("toggle_grid").as_deref(),
            Ok("Toggle grid (g)")
        );
        assert!(
            registry
                .iter()
                .all(|action| !action.label().is_empty() && !action.description().is_empty())
        );
    }

    #[test]
    fn help() {
        let controls = Controls::from_toml("[keyboard]\nzoom_fit = [\"Home\"]\n").expect("TOML");
        let registry = ActionRegistry::navigation();
        let help = registry.help(&controls).markdown(true);
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
        let help = registry.help(&controls).markdown(false);
        assert!(help.contains("| Zoom | `Ctrl+Plus`, `PageUp` Zoom in, "));
    }
}
