// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Mouse buttons, modifier keys and bindings of mouse and keys, e.g. `Ctrl+Shift+Right`.
//!
//! Bindings are written as their modifiers and their input joined by `+`, like in a
//! configuration file.

mod key;
mod modifiers;
mod mouse;

pub use key::KeyBinding;
pub use modifiers::Modifiers;
pub use mouse::{MouseBinding, MouseButton, MouseInput};

use std::fmt;

/// An error of parsing a binding, a mouse button, a modifier or a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

/// The names of `modifiers` and of `input` joined by `+`, e.g. `Ctrl+Right`.
fn join(modifiers: Modifiers, input: &[&str]) -> String {
    let mut names: Vec<&str> = modifiers.names().collect();
    names.extend(input);
    names.join("+")
}

/// The names of a binding joined by `+`, without spaces around them.
fn split(s: &str) -> impl Iterator<Item = &str> {
    s.split('+').map(str::trim).filter(|name| !name.is_empty())
}
