// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The mouse buttons and the bindings of the mouse, e.g. `Ctrl+Shift+Right`.

use std::fmt;
use std::str::FromStr;

use super::{Modifiers, ParseError, split};

/// A button of the mouse, written by its name in a binding, e.g. `Left`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, strum::EnumString, strum::IntoStaticStr,
)]
pub enum MouseButton {
    /// The left button.
    Left,
    /// The middle button, i.e. pressing the wheel.
    Middle,
    /// The right button.
    Right,
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
            MouseInput::Buttons(buttons) => names.extend(buttons.iter().map(|&b| <&str>::from(b))),
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
            if let Ok(button) = name.parse::<MouseButton>() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use MouseButton::*;

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
}
