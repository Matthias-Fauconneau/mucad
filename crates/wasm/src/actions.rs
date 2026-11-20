// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The actions of the viewer with their help, e.g. for tooltips and a help in Markdown.

use mucad_driver::prelude as mu;
use wasm_bindgen::prelude::*;

/// The actions of the viewer with their bindings in the order of its help, see `ActionRegistry`
/// of `mucad-viewer-utils`.
///
/// `mucad.actions` are the actions with their default bindings, e.g. for a `CameraNavigation`.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ActionRegistry(mu::viewer::ActionRegistry);

#[wasm_bindgen]
impl ActionRegistry {
    /// The actions of the navigation with their default bindings: the mouse commands, then the
    /// key commands.
    #[wasm_bindgen(constructor)]
    pub fn new() -> ActionRegistry {
        Self(mu::viewer::ActionRegistry::navigation())
    }

    /// The ids of the actions in the order of the help, e.g. `zoom_fit`.
    pub fn ids(&self) -> Vec<String> {
        self.0.iter().map(|action| action.id().into()).collect()
    }

    /// The label of the action `id` with its bindings, e.g. `Zoom to fit (f)`.
    ///
    /// Throws an error for an unknown action.
    pub fn tooltip(&self, id: &str) -> Result<String, JsError> {
        (self.0.tooltip(id)).map_err(|error| JsError::new(&error.to_string()))
    }

    /// A Markdown table of the actions with their bindings, with `basic` only of the basic ones.
    #[wasm_bindgen(js_name = helpMarkdown)]
    pub fn help_markdown(&self, basic: bool) -> String {
        self.0.help_markdown(basic)
    }
}

impl ActionRegistry {
    /// The actions of `mucad-viewer-utils`.
    pub(crate) fn registry(&self) -> &mu::viewer::ActionRegistry {
        &self.0
    }
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
