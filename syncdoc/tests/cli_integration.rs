#![cfg(feature = "cli")]
use assert_cmd::cargo::cargo_bin_cmd;
use braces::{brace_paths, BraceConfig};
use insta::assert_snapshot;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[ctor::ctor]
fn init_debug() {
    syncdoc_core::debug::set_debug(true);
}

fn to_braces(paths: &[&str]) -> String {
    let braces_config = BraceConfig::default();
    brace_paths(paths, &braces_config).expect("Brace error")
}

fn collect_all_files(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_files_recursive(root, root, &mut files);
    files.sort();
    files
}

fn collect_files_recursive(root: &Path, current: &Path, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Ok(relative) = path.strip_prefix(root) {
                    files.push(relative.to_str().unwrap().to_string());
                }
            } else if path.is_dir() {
                collect_files_recursive(root, &path, files);
            }
        }
    }
}

fn setup_test_project() -> TempDir {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::create_dir(temp.path().join("src")).unwrap();
    fs::write(
        temp.path().join("src/lib.rs"),
        "//! Module docs\n\n/// Function docs\npub fn test() {}\n",
    )
    .unwrap();
    temp
}

#[test]
fn cli_dry_run_does_not_modify_filesystem() {
    let temp = setup_test_project();

    // Snapshot initial file structure
    let initial_files = collect_all_files(temp.path());
    let initial_refs: Vec<&str> = initial_files.iter().map(|s| s.as_str()).collect();
    assert_snapshot!(to_braces(&initial_refs), @"{Cargo.toml,src/lib.rs}");

    cargo_bin_cmd!("syncdoc")
        .current_dir(temp.path())
        .args(["--migrate", "--dry-run"])
        .assert()
        .success();

    let after_files = collect_all_files(temp.path());

    // Should be exactly the same files
    assert_eq!(
        initial_files, after_files,
        "Dry run should not create or modify any files!"
    );

    assert!(
        !temp.path().join("docs").exists(),
        "Dry run created docs directory!"
    );
}
