// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The bindings of keys, e.g. `Ctrl+Plus`.

use std::fmt;
use std::str::FromStr;

use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::{Modifiers, ParseError, join, split};

/// The characters, which are written by their names in bindings, because `+` joins the names
/// and spaces around them are ignored.
const KEY_NAMES: [(&str, &str); 3] = [("+", "Plus"), ("-", "Minus"), (" ", "Space")];

/// A binding of a key, e.g. `Ctrl+Plus`, `PageUp` or `Shift+ArrowLeft`.
///
/// The key is a logical key like `KeyboardEvent.key` of the web, which also Bevy provides:
/// a named key like `ArrowLeft`, `PageUp`, or a character like `0` or `a`. The characters `+`,
/// `-` and ` ` are written `Plus`, `Minus` and `Space`.
///
/// The modifiers must be pressed exactly, but Shift is ignored for characters, because
/// keyboard layouts need it for different characters, e.g. for `+`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, SerializeDisplay, DeserializeFromStr)]
pub struct KeyBinding {
    /// The modifiers, which are pressed.
    pub modifiers: Modifiers,
    /// The logical key, characters in lower case.
    pub key: String,
}

impl KeyBinding {
    /// A binding of `key`, e.g. `ArrowLeft` or `+`, with `modifiers`.
    pub fn new(modifiers: Modifiers, key: &str) -> Self {
        Self {
            modifiers,
            key: normalized_key(key),
        }
    }

    /// Whether `key` with `modifiers` is pressed for this binding.
    pub fn matches(&self, key: &str, modifiers: Modifiers) -> bool {
        let key = normalized_key(key);
        if key != self.key {
            return false;
        }
        if is_character(&key) {
            let without_shift = |m: Modifiers| Modifiers { shift: false, ..m };
            without_shift(modifiers) == without_shift(self.modifiers)
        } else {
            modifiers == self.modifiers
        }
    }
}

impl fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key = KEY_NAMES
            .iter()
            .find(|(key, _)| *key == self.key)
            .map_or(self.key.as_str(), |(_, name)| name);
        f.write_str(&join(self.modifiers, &[key]))
    }
}

impl FromStr for KeyBinding {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let names: Vec<&str> = split(s).collect();
        let Some((key, modifier_names)) = names.split_last() else {
            return Err(ParseError(format!("no key in `{s}`")));
        };
        let modifiers = Modifiers::from_names(modifier_names.iter().copied(), s)?;
        let key = KEY_NAMES
            .iter()
            .find(|(_, name)| name == key)
            .map_or(*key, |(key, _)| key);
        Ok(Self::new(modifiers, key))
    }
}

/// Whether `key` is a single character and not a named key.
fn is_character(key: &str) -> bool {
    key.chars().count() == 1
}

/// Characters in lower case, so that `A` with Shift matches `a`.
fn normalized_key(key: &str) -> String {
    if is_character(key) {
        key.to_lowercase()
    } else {
        key.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_binding() {
        let binding: KeyBinding = "Ctrl+Plus".parse().expect("binding");
        assert_eq!(binding, KeyBinding::new(Modifiers::CTRL, "+"));
        assert_eq!(binding.to_string(), "Ctrl+Plus");
        assert!("Shift+ArrowLeft".parse::<KeyBinding>().is_ok());
        assert!("Hyper+A".parse::<KeyBinding>().is_err());
        assert!("".parse::<KeyBinding>().is_err());
    }

    #[test]
    fn key_matches() {
        // Shift is ignored for characters, e.g. for `+` on a US keyboard.
        let plus = KeyBinding::new(Modifiers::CTRL, "+");
        assert!(plus.matches("+", Modifiers::CTRL));
        assert!(plus.matches("+", Modifiers::CTRL.union(Modifiers::SHIFT)));
        assert!(!plus.matches("+", Modifiers::NONE));
        assert!(KeyBinding::new(Modifiers::NONE, "a").matches("A", Modifiers::SHIFT));

        // But not for named keys.
        let left = KeyBinding::new(Modifiers::NONE, "ArrowLeft");
        assert!(left.matches("ArrowLeft", Modifiers::NONE));
        assert!(!left.matches("ArrowLeft", Modifiers::SHIFT));
    }
}
