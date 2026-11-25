// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration test runner for `.mu` standard library test files in `tests/data/`.

use std::path::PathBuf;

use mucad_driver::{Compiler, prelude as mu};

fn std_compiler() -> Compiler {
    let std_lib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib");
    Compiler::from_config(&mu::DriverConfig {
        search_paths: vec![std_lib],
        ..Default::default()
    })
}

#[test]
fn stdlib_mu_tests() {
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&data_dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", data_dir.display()))
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension().and_then(|ext| ext.to_str()) == Some("mu")).then_some(path)
        })
        .collect();
    files.sort();

    assert!(
        !files.is_empty(),
        "no .mu test files found in {}",
        data_dir.display()
    );

    for path in files {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let mut compiler = std_compiler();
        let diags = match compiler.compile(&*source) {
            Ok((_, diags)) => diags,
            Err(diags) => panic!("{} failed to compile:\n{diags:?}", path.display()),
        };
        assert!(
            !diags.has_errors(),
            "{} reported errors:\n{diags:?}",
            path.display()
        );
    }
}
