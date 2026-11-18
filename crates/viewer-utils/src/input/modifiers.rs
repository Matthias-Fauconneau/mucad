// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The modifier keys, e.g. `Ctrl+Shift`.

use std::fmt;
use std::str::FromStr;

use super::{ParseError, split};

/// The pressed modifier keys, written like `Ctrl+Shift`.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct Modifiers {
    /// Control.
    pub ctrl: bool,
    /// Shift.
    pub shift: bool,
    /// Alt, Option on macOS.
    pub alt: bool,
    /// Meta, Command on macOS, Windows on Windows.
    pub meta: bool,
}

impl Modifiers {
    /// No modifiers.
    pub const NONE: Self = Self {
        ctrl: false,
        shift: false,
        alt: false,
        meta: false,
    };

    /// Control only.
    pub const CTRL: Self = Self {
        ctrl: true,
        ..Self::NONE
    };

    /// Shift only.
    pub const SHIFT: Self = Self {
        shift: true,
        ..Self::NONE
    };

    /// Whether no modifier is pressed.
    pub fn is_empty(self) -> bool {
        self == Self::NONE
    }

    /// Whether all modifiers of `other` are pressed.
    pub fn contains(self, other: Self) -> bool {
        self.union(other) == self
    }

    /// The modifiers of both.
    pub fn union(self, other: Self) -> Self {
        Self {
            ctrl: self.ctrl || other.ctrl,
            shift: self.shift || other.shift,
            alt: self.alt || other.alt,
            meta: self.meta || other.meta,
        }
    }

    /// Set the modifier `name`, returns `false` if it is no modifier.
    pub(super) fn set(&mut self, name: &str) -> bool {
        match name {
            "Ctrl" | "Control" => self.ctrl = true,
            "Shift" => self.shift = true,
            "Alt" | "Option" => self.alt = true,
            "Meta" | "Cmd" | "Command" | "Super" => self.meta = true,
            _ => return false,
        }
        true
    }

    /// The names of the pressed modifiers.
    pub(super) fn names(self) -> impl Iterator<Item = &'static str> {
        [
            (self.ctrl, "Ctrl"),
            (self.shift, "Shift"),
            (self.alt, "Alt"),
            (self.meta, "Meta"),
        ]
        .into_iter()
        .filter_map(|(pressed, name)| pressed.then_some(name))
    }
}

/// Modifiers are written like `Ctrl+Shift`, no modifiers as an empty string.
impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.names().collect::<Vec<_>>().join("+"))
    }
}

impl FromStr for Modifiers {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut modifiers = Self::NONE;
        for name in split(s) {
            if !modifiers.set(name) {
                return Err(ParseError(format!("unknown modifier `{name}` in `{s}`")));
            }
        }
        Ok(modifiers)
    }
}

impl TryFrom<String> for Modifiers {
    type Error = ParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<Modifiers> for String {
    fn from(modifiers: Modifiers) -> Self {
        modifiers.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers() {
        let modifiers: Modifiers = "Ctrl+Shift".parse().expect("modifiers");
        assert!(modifiers.ctrl && modifiers.shift && !modifiers.alt);
        assert_eq!(modifiers.to_string(), "Ctrl+Shift");
        assert_eq!("".parse(), Ok(Modifiers::NONE));
        assert!("Hyper".parse::<Modifiers>().is_err());
        assert!(modifiers.contains(Modifiers::CTRL));
        assert!(!Modifiers::CTRL.contains(modifiers));
    }
}
