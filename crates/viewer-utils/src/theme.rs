// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The color theme of the µcad viewers and editors, e.g. from TOML, and its CSS.

use mucad_core::Color;
use serde::{Deserialize, Serialize};

/// The color theme of the µcad viewers and editors.
///
/// Its default is the only definition of the theme colors in the workspace. It is read from
/// TOML, missing entries are the defaults, colors are written in hex like `#RRGGBB`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    /// The name of the theme.
    pub name: String,
    /// The background.
    #[serde(with = "hex")]
    pub primary: Color,
    /// A bright color, e.g. for text and models.
    #[serde(with = "hex")]
    pub bright: Color,
    /// The brightest color, e.g. for highlighted text and lights.
    #[serde(with = "hex")]
    pub brighter: Color,
    /// A dark color, e.g. for the grid and panels.
    #[serde(with = "hex")]
    pub darker: Color,
    /// The complementary color, e.g. for errors, operators and intersections.
    #[serde(with = "hex")]
    pub complementary: Color,
    /// The signal color, e.g. for selections and buttons.
    #[serde(with = "hex")]
    pub signal: Color,
    /// The color of guides, e.g. inference lines.
    #[serde(with = "hex")]
    pub guide: Color,
}

impl Default for Theme {
    fn default() -> Self {
        let hex = |s| Color::from_hex_str(s).expect("A valid hex color");
        Self {
            name: "default".into(),
            primary: hex("#020D1A"),
            bright: hex("#82A8C4"),
            brighter: hex("#E6F4FF"),
            darker: hex("#33556D"),
            complementary: hex("#C5221F"),
            signal: hex("#FF9F0A"),
            guide: hex("#00B4D8"),
        }
    }
}

impl Theme {
    /// The theme in TOML, missing entries are the defaults.
    pub fn from_toml(toml: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml)
    }

    /// The theme as TOML.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    /// The colors of the TOML of the theme as CSS custom properties of `:root`, e.g.
    /// `--primary: #020D1A;`, in alphabetical order.
    pub fn to_css(&self) -> Result<String, toml::ser::Error> {
        let table = toml::Table::try_from(self)?;
        let mut css = String::from(":root {\n");
        for (name, value) in &table {
            if name == "name" {
                continue;
            }
            if let Some(color) = value.as_str() {
                css.push_str(&format!("  --{}: {color};\n", name.replace('_', "-")));
            }
        }
        css.push_str("}\n");
        Ok(css)
    }
}

/// Colors in TOML as hex, e.g. `#RRGGBB`, or `#RRGGBBAA` with alpha, also by name, e.g. `red`.
mod hex {
    use mucad_core::Color;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(color: &Color, serializer: S) -> Result<S::Ok, S::Error> {
        let component = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
        let mut hex = format!(
            "#{:02X}{:02X}{:02X}",
            component(color.r),
            component(color.g),
            component(color.b)
        );
        if component(color.a) != 255 {
            hex.push_str(&format!("{:02X}", component(color.a)));
        }
        serializer.serialize_str(&hex)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Color, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse()
            .map_err(|error| serde::de::Error::custom(format!("invalid color `{s}`: {error}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css() {
        let css = Theme::default().to_css().expect("CSS");
        assert!(css.starts_with(":root {\n"), "{css}");
        assert!(css.contains("  --primary: #020D1A;\n"), "{css}");
        assert!(css.contains("  --brighter: #E6F4FF;\n"), "{css}");
        assert!(!css.contains("name"), "{css}");
        assert!(css.ends_with("}\n"));
    }
}
