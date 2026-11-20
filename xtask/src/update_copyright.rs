// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

use crate::generated::{CommentStyle, LICENSE_ID};

/// Executes `git log` once for the repository and maps file paths to their commit year ranges.
fn get_revision_years(repo_path: impl AsRef<Path>) -> HashMap<PathBuf, RangeInclusive<u32>> {
    let mut file_years = HashMap::new();
    let Ok(output) = Command::new("git")
        .current_dir(&repo_path)
        .args(["log", "--name-only", "--format=%ad", "--date=format:%Y"])
        .output()
    else {
        return file_years;
    };

    let mut current_year = None;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines().map(str::trim).filter(|s| !s.is_empty()) {
        if line.len() == 4 && line.chars().all(|c| c.is_ascii_digit()) {
            current_year = line.parse().ok();
        } else if let Some(year) = current_year {
            file_years
                .entry(PathBuf::from(line))
                .and_modify(|range: &mut RangeInclusive<u32>| {
                    let min = (*range.start()).min(year);
                    let max = (*range.end()).max(year);
                    *range = min..=max;
                })
                .or_insert(year..=year);
        }
    }

    file_years
}

/// The copyright notice of a file in `comment_style` for `years` like `2025-2026` and
/// `license_id`, followed by an empty line.
pub fn copyright_header(comment_style: CommentStyle, years: &str, license_id: &str) -> String {
    format!(
        "{}\n{}\n\n",
        comment_style.comment(&format!(
            "Copyright © {years} The µcad authors <info@mucad.org>"
        )),
        comment_style.comment(&format!("SPDX-License-Identifier: {license_id}")),
    )
}

/// The copyright notice in `comment_style` at the start of `content` up to the empty line
/// after it, if it has one.
pub fn copyright_notice(content: &str, comment_style: CommentStyle) -> Option<&str> {
    let first_line = content.lines().next()?;
    let text = first_line.strip_prefix(comment_style.prefix())?;
    if !text.trim_start().starts_with("Copyright ©") {
        return None;
    }
    content.find("\n\n").map(|end| &content[..end + 2])
}

/// Manages updating copyright notices across source files in a repository.
pub struct CopyrightUpdater {
    path: PathBuf,
    /// The comment style of each extension.
    extensions: HashMap<String, CommentStyle>,
    excluded_patterns: HashSet<PathBuf>,

    /// SPDX-License-Identifier
    license_id: String,
}

impl CopyrightUpdater {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            extensions: Default::default(),
            excluded_patterns: Default::default(),
            license_id: LICENSE_ID.to_string(),
        }
    }

    /// Change license id.
    pub fn license(mut self, license: impl AsRef<str>) -> Self {
        self.license_id = license.as_ref().to_string();
        self
    }

    /// Add an extension with its comment style, e.g. of an unknown file type.
    pub fn extension(mut self, extension: &str, comment_style: CommentStyle) -> Self {
        self.extensions
            .insert(extension.trim_start_matches('.').to_string(), comment_style);
        self
    }

    /// Add extensions of known file types with their comment styles, see
    /// [`CommentStyle::from_extension`].
    ///
    /// # Panics
    ///
    /// If the comment style of an extension is not known.
    pub fn extensions(self, extensions: &[&str]) -> Self {
        extensions.iter().fold(self, |updater, extension| {
            let comment_style = CommentStyle::from_extension(extension)
                .unwrap_or_else(|| panic!("Unknown comment style of `{extension}` files"));
            updater.extension(extension, comment_style)
        })
    }

    /// Add an explicit excluded path or glob pattern.
    pub fn exclude(mut self, path: impl Into<PathBuf>) -> Self {
        self.excluded_patterns.insert(path.into());
        self
    }

    pub fn with_git_exclusions(mut self) -> Self {
        if let Ok(output) = Command::new("git")
            .current_dir(&self.path)
            .args(["status", "--ignored", "--porcelain"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().filter_map(|l| l.strip_prefix("!! ")) {
                let trimmed = line.trim().trim_end_matches('/');
                self.excluded_patterns.insert(PathBuf::from(trimmed));
                self.excluded_patterns
                    .insert(PathBuf::from(format!("{trimmed}/*")));
            }
        }
        self
    }

    pub fn update(&self) -> std::io::Result<()> {
        let revision_years = get_revision_years(&self.path);
        let copyright_re = Regex::new(r"Copyright © (\d{4})(-\d{4})?").unwrap();
        let current_year = chrono::Datelike::year(&chrono::Utc::now()) as u32;

        let exclusion_globs: Vec<_> = self
            .excluded_patterns
            .iter()
            .filter_map(|p| glob::Pattern::new(&p.to_string_lossy()).ok())
            .collect();

        for entry in WalkDir::new(&self.path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let relative_path = path.strip_prefix(&self.path).unwrap_or(path);

            if exclusion_globs
                .iter()
                .any(|g| g.matches(&relative_path.to_string_lossy()))
            {
                continue;
            }

            let Some(ext) = path.extension().and_then(OsStr::to_str) else {
                continue;
            };
            let Some(comment_style) = self.extensions.get(ext) else {
                continue;
            };

            let range = revision_years
                .get(relative_path)
                .cloned()
                .unwrap_or(current_year..=current_year);

            let formatted_years = if range.start() == range.end() {
                format!("{}", range.start())
            } else {
                format!("{}-{}", range.start(), range.end())
            };

            let content = fs::read_to_string(path)?;

            let license_id = &self.license_id;
            let updated_content = if copyright_re.is_match(&content) {
                copyright_re
                    .replace(&content, format!("Copyright © {formatted_years}"))
                    .to_string()
            } else {
                copyright_header(*comment_style, &formatted_years, license_id) + &content
            };

            if updated_content != content {
                println!("updated: {path:?} -> {formatted_years}");
                fs::write(path, updated_content.as_bytes())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file without notice gets one in the comments of its extension, e.g. `/* */` in CSS.
    #[test]
    fn notices_in_comments_of_extensions() {
        let dir = tempfile::tempdir().expect("A temporary directory");
        fs::write(dir.path().join("theme.css"), ":root {}\n").expect("CSS");
        fs::write(dir.path().join("theme.ts"), "export {};\n").expect("TS");

        CopyrightUpdater::new(dir.path())
            .extensions(&["ts", "css"])
            .update()
            .expect("Updated notices");

        let year = chrono::Datelike::year(&chrono::Utc::now()).to_string();
        let css = fs::read_to_string(dir.path().join("theme.css")).expect("CSS");
        assert_eq!(
            css,
            format!(
                "/* Copyright © {year} The µcad authors <info@mucad.org> */\n\
                 /* SPDX-License-Identifier: AGPL-3.0-or-later */\n\n:root {{}}\n"
            )
        );
        let ts = fs::read_to_string(dir.path().join("theme.ts")).expect("TS");
        assert!(
            ts.starts_with(&format!("// Copyright © {year} The µcad")),
            "{ts}"
        );
    }
}
