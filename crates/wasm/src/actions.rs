// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The actions of the viewer with their help, e.g. for tooltips and a help in Markdown.

use mucad_driver::prelude as mu;
use wasm_bindgen::prelude::*;

use crate::CameraNavigation;

/// The actions of the viewer in the order of its help, see `ActionRegistry` of
/// `mucad-viewer-utils`.
///
/// The help shows the bindings of the current controls of a navigation.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ActionRegistry(mu::viewer::ActionRegistry);

#[wasm_bindgen]
impl ActionRegistry {
    /// The actions of the navigation: the mouse commands, then the key commands.
    #[wasm_bindgen(constructor)]
    pub fn new() -> ActionRegistry {
        Self(mu::viewer::ActionRegistry::navigation())
    }

    /// The ids of the actions in the order of the help, e.g. `zoom_fit`.
    pub fn ids(&self) -> Vec<String> {
        self.0.iter().map(|action| action.id().into()).collect()
    }

    /// The tooltip of the action `id` with the bindings of `navigation`, e.g.
    /// `Zoom to fit (f)`.
    ///
    /// Throws an error for an unknown action.
    pub fn tooltip(&self, navigation: &CameraNavigation, id: &str) -> Result<String, JsError> {
        let help = self.0.help(navigation.controls());
        help.tooltip(id)
            .map_err(|error| JsError::new(&error.to_string()))
    }

    /// A Markdown table of the actions with the bindings of `navigation`, with `basic` only of
    /// the basic ones.
    #[wasm_bindgen(js_name = helpMarkdown)]
    pub fn help_markdown(&self, navigation: &CameraNavigation, basic: bool) -> String {
        self.0.help(navigation.controls()).markdown(basic)
    }
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
