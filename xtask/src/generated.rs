// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Files, which xtasks generate, e.g. `cargo xtask web`, with the copyright notices of
//! `cargo xtask update-copyright` in the comments of their file types.
//!
//! A generated file keeps its notice, a new one gets the notice of the current year.

use std::error::Error;
use std::path::{Path, PathBuf};

use mucad_driver::prelude::MUCAD_EXTENSIONS;

use crate::change_version::workspace_root;
use crate::update_copyright::{copyright_header, copyright_notice};

/// The license of the generated files and the default of `cargo xtask update-copyright`.
pub const LICENSE_ID: &str = "AGPL-3.0-or-later";

/// The comments of a file type, e.g. for its copyright notice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentStyle {
    /// `#` comments like in TOML or Python.
    Hash,
    /// `//` one line comments like in µcad or Rust.
    Slash,
    /// `/* */` inline comments like in CSS.
    Inline,
}

impl CommentStyle {
    /// The comment style of the files with `extension`, e.g. `rs`, if it is known.
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.trim_start_matches('.') {
            "rs" | "slint" | "ts" | "js" | "wgsl" => Some(Self::Slash),
            extension if MUCAD_EXTENSIONS.contains(&extension) => Some(Self::Slash),
            "css" => Some(Self::Inline),
            "toml" | "py" | "sh" | "yml" | "yaml" => Some(Self::Hash),
            _ => None,
        }
    }

    /// `text` as a comment of one line, without line break.
    pub fn comment(self, text: &str) -> String {
        match self {
            Self::Hash => format!("# {text}"),
            Self::Slash => format!("// {text}"),
            Self::Inline => format!("/* {text} */"),
        }
    }

    /// The start of a comment of one line, e.g. `//`.
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Hash => "#",
            Self::Slash => "//",
            Self::Inline => "/*",
        }
    }
}

/// A generated file: its path relative to the workspace root, its content without copyright
/// notice and the comment style of its notice.
pub struct Generated {
    path: PathBuf,
    content: String,
    comment_style: CommentStyle,
}

impl Generated {
    /// A generated file at `path` with `content` and a notice in `comment_style`.
    pub fn new(path: impl AsRef<Path>, content: String, comment_style: CommentStyle) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            content,
            comment_style,
        }
    }

    /// A generated TypeScript file.
    pub fn ts(path: impl AsRef<Path>, content: String) -> Self {
        Self::new(path, content, CommentStyle::Slash)
    }

    /// A generated CSS file.
    pub fn css(path: impl AsRef<Path>, content: String) -> Self {
        Self::new(path, content, CommentStyle::Inline)
    }

    /// The content with the notice of `current`, the current content of the file, or with a
    /// new notice of the current year.
    fn with_notice(&self, current: &str) -> String {
        let notice = copyright_notice(current, self.comment_style).map_or_else(
            || {
                let year = chrono::Datelike::year(&chrono::Utc::now()).to_string();
                copyright_header(self.comment_style, &year, LICENSE_ID)
            },
            str::to_string,
        );
        notice + &self.content
    }
}

/// Write `files` into the workspace, or with `check` only report the ones which are outdated,
/// with `command`, which generates them, e.g. `cargo xtask web`.
pub fn write(
    files: impl IntoIterator<Item = Generated>,
    check: bool,
    command: &str,
) -> Result<(), Box<dyn Error>> {
    let root = workspace_root();
    let mut outdated = Vec::new();
    for generated in files {
        let file = root.join(&generated.path);
        let current = std::fs::read_to_string(&file).unwrap_or_default();
        let content = generated.with_notice(&current);
        if current == content {
            continue;
        }
        if check {
            outdated.push(generated.path.display().to_string());
        } else {
            std::fs::create_dir_all(file.parent().unwrap_or(Path::new(".")))?;
            std::fs::write(&file, content)?;
            println!("Generated {}", generated.path.display());
        }
    }
    if outdated.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Outdated generated files, run `{command}`: {}",
            outdated.join(", ")
        )
        .into())
    }
}
