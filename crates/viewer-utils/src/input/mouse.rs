// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The mouse buttons and the bindings of the mouse, e.g. `Ctrl+Shift+Right`.

use std::fmt;
use std::str::FromStr;

use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::{Modifiers, ParseError, join, split};

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, SerializeDisplay, DeserializeFromStr)]
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
        let input: Vec<&str> = match &self.input {
            MouseInput::Wheel => vec!["Wheel"],
            MouseInput::Buttons(buttons) => buttons.iter().map(|&b| b.into()).collect(),
        };
        f.write_str(&join(self.modifiers, &input))
    }
}

impl FromStr for MouseBinding {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let names: Vec<&str> = split(s).collect();
        // The modifiers come first, then the wheel or the buttons.
        let is_input = |name: &&str| *name == "Wheel" || name.parse::<MouseButton>().is_ok();
        let start = names
            .iter()
            .position(is_input)
            .ok_or_else(|| ParseError(format!("no button or wheel in `{s}`")))?;
        let (modifier_names, inputs) = names.split_at(start);
        let modifiers = Modifiers::from_names(modifier_names.iter().copied(), s)?;
        if inputs == ["Wheel"] {
            return Ok(Self::wheel(modifiers));
        }
        let buttons: Vec<MouseButton> = inputs
            .iter()
            .map(|name| {
                name.parse().map_err(|_| {
                    ParseError(format!(
                        "misplaced `{name}` in mouse binding `{s}`, expected modifiers and \
                         then `Left`, `Middle`, `Right` or `Wheel`"
                    ))
                })
            })
            .collect::<Result<_, _>>()?;
        if (1..buttons.len()).any(|i| buttons[..i].contains(&buttons[i])) {
            return Err(ParseError(format!("a button twice in `{s}`")));
        }
        Ok(Self::buttons(modifiers, &buttons))
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
