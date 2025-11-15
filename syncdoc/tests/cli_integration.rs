#![cfg(feature = "cli")]
use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

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
    let original = fs::read_to_string(temp.path().join("src/lib.rs")).unwrap();
    let original_toml = fs::read_to_string(temp.path().join("Cargo.toml")).unwrap();

    cargo_bin_cmd!("syncdoc")
        .current_dir(temp.path())
        .args(["--migrate", "--dry-run"])
        .assert()
        .success();

    let after = fs::read_to_string(temp.path().join("src/lib.rs")).unwrap();
    assert_eq!(original, after, "Dry run modified source files!");

    let after_toml = fs::read_to_string(temp.path().join("Cargo.toml")).unwrap();
    assert_eq!(original_toml, after_toml, "Dry run modified Cargo.toml!");

    assert!(
        !temp.path().join("docs").exists(),
        "Dry run created docs directory!"
    );
}
