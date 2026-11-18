// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The modifier keys, e.g. `Ctrl+Shift`.

use std::fmt;
use std::str::FromStr;

use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::{ParseError, join, split};

/// The pressed modifier keys, written like `Ctrl+Shift`.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SerializeDisplay, DeserializeFromStr,
)]
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

    /// The modifiers of `names`, e.g. of the binding `s` for errors.
    pub(super) fn from_names<'a>(
        names: impl IntoIterator<Item = &'a str>,
        s: &str,
    ) -> Result<Self, ParseError> {
        names.into_iter().try_fold(Self::NONE, |modifiers, name| {
            let modifier = match name {
                "Ctrl" | "Control" => Self::CTRL,
                "Shift" => Self::SHIFT,
                "Alt" | "Option" => Self {
                    alt: true,
                    ..Self::NONE
                },
                "Meta" | "Cmd" | "Command" | "Super" => Self {
                    meta: true,
                    ..Self::NONE
                },
                _ => return Err(ParseError(format!("unknown modifier `{name}` in `{s}`"))),
            };
            Ok(modifiers.union(modifier))
        })
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
        f.write_str(&join(*self, &[]))
    }
}

impl FromStr for Modifiers {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_names(split(s), s)
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
