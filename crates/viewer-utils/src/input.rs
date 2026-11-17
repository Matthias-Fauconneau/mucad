// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Mouse buttons, modifier keys and bindings of mouse and keys, e.g. `Ctrl+Shift+Right`.
//!
//! Bindings are written as their modifiers and their input joined by `+`, like in a
//! configuration file.

use std::fmt;
use std::str::FromStr;

/// An error of parsing a binding, a mouse button, a modifier or a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

/// A button of the mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// The left button.
    Left,
    /// The middle button, i.e. pressing the wheel.
    Middle,
    /// The right button.
    Right,
}

impl MouseButton {
    /// The name in a binding.
    pub fn name(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Middle => "Middle",
            Self::Right => "Right",
        }
    }

    fn parse(name: &str) -> Option<Self> {
        match name {
            "Left" => Some(Self::Left),
            "Middle" => Some(Self::Middle),
            "Right" => Some(Self::Right),
            _ => None,
        }
    }
}

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
    fn set(&mut self, name: &str) -> bool {
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
    fn names(self) -> impl Iterator<Item = &'static str> {
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

/// The input of a [`MouseBinding`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MouseInput {
    /// Turning the wheel.
    Wheel,
    /// Dragging or clicking with the buttons, which are pressed in this order, e.g. first
    /// `Middle` and then `Right`.
    Buttons(Vec<MouseButton>),
}

/// A binding of the mouse, e.g. `Ctrl+Shift+Right`, `Middle+Right` or `Wheel`.
///
/// The modifiers must be pressed exactly, and the buttons in their order.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MouseBinding {
    /// The modifiers, which are pressed.
    pub modifiers: Modifiers,
    /// The wheel or the buttons.
    pub input: MouseInput,
}

impl MouseBinding {
    /// A binding of `buttons` pressed in their order with `modifiers`.
    pub fn buttons(modifiers: Modifiers, buttons: &[MouseButton]) -> Self {
        Self {
            modifiers,
            input: MouseInput::Buttons(buttons.to_vec()),
        }
    }

    /// A binding of the wheel with `modifiers`.
    pub fn wheel(modifiers: Modifiers) -> Self {
        Self {
            modifiers,
            input: MouseInput::Wheel,
        }
    }
}

impl fmt::Display for MouseBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut names: Vec<&str> = self.modifiers.names().collect();
        match &self.input {
            MouseInput::Wheel => names.push("Wheel"),
            MouseInput::Buttons(buttons) => names.extend(buttons.iter().map(|b| b.name())),
        }
        f.write_str(&names.join("+"))
    }
}

impl FromStr for MouseBinding {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut modifiers = Modifiers::NONE;
        let mut buttons = Vec::new();
        let mut wheel = false;
        for name in split(s) {
            if let Some(button) = MouseButton::parse(name) {
                if buttons.contains(&button) {
                    return Err(ParseError(format!("button `{name}` twice in `{s}`")));
                }
                buttons.push(button);
            } else if name == "Wheel" {
                wheel = true;
            } else if !buttons.is_empty() || wheel || !modifiers.set(name) {
                return Err(ParseError(format!(
                    "unknown or misplaced `{name}` in mouse binding `{s}`, \
                     expected modifiers and then `Left`, `Middle`, `Right` or `Wheel`"
                )));
            }
        }
        let input = match (wheel, buttons.is_empty()) {
            (true, true) => MouseInput::Wheel,
            (false, false) => MouseInput::Buttons(buttons),
            (true, false) => {
                return Err(ParseError(format!("wheel and buttons in `{s}`")));
            }
            (false, true) => {
                return Err(ParseError(format!("no button or wheel in `{s}`")));
            }
        };
        Ok(Self { modifiers, input })
    }
}

impl TryFrom<String> for MouseBinding {
    type Error = ParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<MouseBinding> for String {
    fn from(binding: MouseBinding) -> Self {
        binding.to_string()
    }
}

/// A binding of a key, e.g. `Ctrl+Plus`, `PageUp` or `Shift+ArrowLeft`.
///
/// The key is a logical key like `KeyboardEvent.key` of the web, which also Bevy provides:
/// a named key like `ArrowLeft`, `PageUp`, or a character like `0` or `a`. The characters `+`,
/// `-` and ` ` are written `Plus`, `Minus` and `Space`.
///
/// The modifiers must be pressed exactly, but Shift is ignored for characters, because
/// keyboard layouts need it for different characters, e.g. for `+`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
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
        let key = match self.key.as_str() {
            "+" => "Plus",
            "-" => "Minus",
            " " => "Space",
            key => key,
        };
        let mut names: Vec<&str> = self.modifiers.names().collect();
        names.push(key);
        f.write_str(&names.join("+"))
    }
}

impl FromStr for KeyBinding {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let names: Vec<&str> = split(s).collect();
        let Some((key, modifier_names)) = names.split_last() else {
            return Err(ParseError(format!("no key in `{s}`")));
        };
        let mut modifiers = Modifiers::NONE;
        for name in modifier_names {
            if !modifiers.set(name) {
                return Err(ParseError(format!("unknown modifier `{name}` in `{s}`")));
            }
        }
        let key = match *key {
            "Plus" => "+",
            "Minus" => "-",
            "Space" => " ",
            key => key,
        };
        Ok(Self::new(modifiers, key))
    }
}

impl TryFrom<String> for KeyBinding {
    type Error = ParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<KeyBinding> for String {
    fn from(binding: KeyBinding) -> Self {
        binding.to_string()
    }
}

/// The names of a binding joined by `+`, without spaces around them.
fn split(s: &str) -> impl Iterator<Item = &str> {
    s.split('+').map(str::trim).filter(|name| !name.is_empty())
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
    use MouseButton::*;

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

    #[test]
    fn mouse_binding() {
        let binding: MouseBinding = "Ctrl+Shift+Right".parse().expect("binding");
        assert_eq!(
            binding,
            MouseBinding::buttons(Modifiers::CTRL.union(Modifiers::SHIFT), &[Right])
        );
        assert_eq!(binding.to_string(), "Ctrl+Shift+Right");

        let binding: MouseBinding = "Middle+Right".parse().expect("binding");
        assert_eq!(binding.input, MouseInput::Buttons(vec![Middle, Right]));
        assert_eq!("Wheel".parse(), Ok(MouseBinding::wheel(Modifiers::NONE)));

        for wrong in ["", "Ctrl", "Right+Ctrl", "Wheel+Left", "Left+Left", "Up"] {
            assert!(wrong.parse::<MouseBinding>().is_err(), "{wrong}");
        }
    }

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
