// (C) 2025 - Enzo Lombardi

//! The documentation indexes stay complete: every public view module is in
//! `AGENTS.md`'s component index, and every view and core module has a
//! section in `docs/RUST-API-CATALOG.md`. A new module fails this test until
//! it is documented in both.

use std::fs;
use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(ROOT).join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The `pub mod name;` lines of a `mod.rs`.
fn public_modules(mod_rs: &str) -> Vec<String> {
    read(mod_rs)
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .filter_map(|rest| rest.strip_suffix(';'))
        .map(str::to_string)
        .collect()
}

/// Where a module's source lives, as the catalog names it: `src/x/name.rs`
/// for a file, `src/x/name/` for a directory module.
fn source_path(dir: &str, module: &str) -> String {
    let file = format!("{dir}/{module}.rs");
    if Path::new(ROOT).join(&file).exists() {
        file
    } else {
        format!("{dir}/{module}/")
    }
}

#[test]
fn every_view_module_is_in_the_agents_component_index() {
    let agents = read("AGENTS.md");
    let missing: Vec<String> = public_modules("src/views/mod.rs")
        .into_iter()
        .filter(|m| {
            !agents.contains(&format!("views::{m}`")) && !agents.contains(&format!("views::{m},"))
        })
        .collect();
    assert!(
        missing.is_empty(),
        "add these modules to AGENTS.md's component index (as `views::<name>`): {missing:?}"
    );
}

#[test]
fn every_view_and_core_module_has_a_catalog_section() {
    let catalog = read("docs/RUST-API-CATALOG.md");
    let mut missing = Vec::new();
    for (mod_rs, dir) in [
        ("src/views/mod.rs", "src/views"),
        ("src/core/mod.rs", "src/core"),
    ] {
        for module in public_modules(mod_rs) {
            let path = source_path(dir, &module);
            if !catalog.contains(&format!("(`{path}")) {
                missing.push(path);
            }
        }
    }
    assert!(
        missing.is_empty(),
        "add a section to docs/RUST-API-CATALOG.md for: {missing:?}"
    );
}
