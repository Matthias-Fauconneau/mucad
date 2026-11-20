// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::Source;

fn _formatted_test_case(name: &str) {
    use mucad_lang_format::{FormatConfig, format_ast};
    let source = Source::load(format!("tests/test_cases/formatted/{name}.mu")).expect("No errors");
    let ast = mucad_lang_parse::parse(&source).expect("No errors").0;

    pretty_assertions::assert_eq!(
        source.code(),
        format_ast(&ast, &FormatConfig::default()),
        "Format error:\n{ast:#?}",
    );
}

fn _unformatted_test_case(name: &str) {
    use mucad_lang_format::{FormatConfig, format_ast};
    let source =
        Source::load(format!("tests/test_cases/unformatted/{name}.mu")).expect("no errors");
    let ast = mucad_lang_parse::parse(&source).expect("No errors").0;
    insta::assert_snapshot!(name, format_ast(&ast, &FormatConfig::default()))
}

// A test case whose format output does not change.
macro_rules! formatted_test_case {
    ($name:ident) => {
        #[test]
        fn $name() {
            _formatted_test_case(stringify!($name));
        }
    };
}

// A test case whose format output does not change.
macro_rules! unformatted_test_case {
    ($name:ident) => {
        #[test]
        fn $name() {
            _unformatted_test_case(stringify!($name));
        }
    };
}

formatted_test_case!(statement_list_whitespace);
formatted_test_case!(list);
formatted_test_case!(tuple);
formatted_test_case!(tuple_type);
formatted_test_case!(body);
formatted_test_case!(expression);
formatted_test_case!(expression_if);
formatted_test_case!(statements);
formatted_test_case!(workbench);
formatted_test_case!(extras);
formatted_test_case!(extras_module);
formatted_test_case!(extras_multiline_comment);
formatted_test_case!(extras_comment_separator);
formatted_test_case!(extras_workbench);
formatted_test_case!(extras_attribute);
formatted_test_case!(definition);
formatted_test_case!(string);
formatted_test_case!(alignment);
formatted_test_case!(width);

unformatted_test_case!(init);
unformatted_test_case!(line_breaks);

/// The `.mu` files in a directory and its subdirectories, without the build directories and the
/// checkouts of the grammars of the editors.
fn mu_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("readable directory") {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            if !["target", "node_modules", "grammars"]
                .iter()
                .any(|dir| path.ends_with(dir))
            {
                mu_files(&path, files);
            }
        } else if path.extension().is_some_and(|ext| ext == "mu") {
            files.push(path);
        }
    }
}

/// Every `.mu` file of the repository is formatted into code, which can be parsed, and whose
/// format does not change anymore.
///
/// Files, which cannot be parsed, e.g. of tests of parse errors, are not formatted.
#[test]
fn round_trip() {
    use mucad_lang_format::{FormatConfig, FormatError, format};

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for dir in ["crates", "examples", "editors"] {
        mu_files(&root.join(dir), &mut files);
    }
    assert!(!files.is_empty());

    let config = FormatConfig::default();
    let mut failures = Vec::new();
    for path in files {
        let name = path
            .strip_prefix(&root)
            .expect("in root")
            .to_string_lossy()
            .to_string();
        let source = Source::load(&path).expect("readable file");
        let formatted = match format(source, &config) {
            Ok(formatted) => formatted.source,
            Err(FormatError::Parse(_)) => continue,
            Err(err) => {
                failures.push(format!("{name}: {err}"));
                continue;
            }
        };
        match format(formatted.clone(), &config) {
            Ok(again) if again.source.code() == formatted.code() => {}
            Ok(_) => failures.push(format!("{name}: the format changes when formatted again")),
            Err(err) => failures.push(format!("{name}: {err}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
