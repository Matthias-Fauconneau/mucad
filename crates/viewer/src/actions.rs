// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The actions of the viewer with their help, e.g. for tooltips and a help in Markdown, see
//! [`Actions`].

use std::sync::Arc;

use bevy::{app::App, ecs::resource::Resource, prelude::Deref};
use mu::viewer::{Action, ActionRegistry};
use mucad_driver::prelude as mu;

/// The actions of the viewer with their bindings in the order of its help: the ones of the
/// navigation, then the registered ones, see [`RegisterAction`].
///
/// The camera navigation shares them, e.g. with the bindings of the controls.
#[derive(Resource, Debug, Clone, Deref)]
pub struct Actions(pub Arc<ActionRegistry>);

impl Default for Actions {
    fn default() -> Self {
        Self(Arc::new(ActionRegistry::navigation()))
    }
}

/// Register actions of plugins in the [`Actions`] of an app.
pub trait RegisterAction {
    /// Register `action` after the others.
    ///
    /// # Panics
    ///
    /// If an action with its id is already registered.
    fn register_action(&mut self, action: impl Action + 'static) -> &mut Self;
}

impl RegisterAction for App {
    fn register_action(&mut self, action: impl Action + 'static) -> &mut Self {
        self.init_resource::<Actions>();
        let mut actions = self.world_mut().resource_mut::<Actions>();
        if let Err(error) = Arc::make_mut(&mut actions.0).register(action) {
            panic!("{error}");
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use mu::viewer::{Binding, BindingKind};

    use super::*;

    /// An action of a plugin.
    struct Screenshot;

    impl Action for Screenshot {
        fn id(&self) -> &'static str {
            "screenshot"
        }

        fn label(&self) -> &'static str {
            "Screenshot"
        }

        fn description(&self) -> &'static str {
            "Save a screenshot."
        }

        fn kind(&self) -> BindingKind {
            BindingKind::Key
        }

        fn default_bindings(&self) -> Vec<Binding> {
            Vec::new()
        }
    }

    #[test]
    fn register() {
        let mut app = App::new();
        app.register_action(Screenshot);
        let actions = app.world().resource::<Actions>();
        assert!(actions.get("zoom_fit").is_some());
        assert_eq!(
            actions.iter().last().map(|action| action.id()),
            Some("screenshot")
        );
    }

    #[test]
    #[should_panic(expected = "The action `screenshot` is already registered")]
    fn register_twice() {
        App::new()
            .register_action(Screenshot)
            .register_action(Screenshot);
    }
}
