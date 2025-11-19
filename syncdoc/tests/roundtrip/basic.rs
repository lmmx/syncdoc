#![cfg(feature = "cli")]
use crate::helpers::{run_roundtrip, setup_test_project, ModuleConfig};
use insta::assert_snapshot;

#[test]
fn roundtrip_config() {
    let temp = setup_test_project(&[ModuleConfig::new("config")]);
    let result = run_roundtrip(temp.path());

    assert!(
        result.is_perfectly_restored(),
        "config.rs should round-trip perfectly"
    );

    assert_snapshot!(result.migrate_stderr);
}

#[test]
fn roundtrip_edit_plan() {
    let temp = setup_test_project(&[ModuleConfig::new("edit_plan")]);
    let result = run_roundtrip(temp.path());

    assert!(
        result.is_perfectly_restored(),
        "edit_plan.rs should round-trip perfectly"
    );

    assert_snapshot!(result.migrate_stderr);
}

#[test]
fn roundtrip_input() {
    let temp = setup_test_project(&[ModuleConfig::new("input")]);
    let result = run_roundtrip(temp.path());

    assert!(
        result.is_perfectly_restored(),
        "input.rs should round-trip perfectly"
    );

    assert_snapshot!(result.migrate_stderr);
}

#[test]
fn roundtrip_formats_parent() {
    let temp = setup_test_project(&[ModuleConfig::new("formats")]);
    let result = run_roundtrip(temp.path());

    assert!(
        result.is_perfectly_restored(),
        "formats.rs should round-trip perfectly"
    );

    assert_snapshot!(result.migrate_stderr);
}

#[test]
fn roundtrip_formats_markdown() {
    let temp = setup_test_project(&[
        ModuleConfig::new("formats"),
        ModuleConfig::with_submodules("formats", &["markdown"]),
    ]);
    let result = run_roundtrip(temp.path());

    assert!(
        result.is_perfectly_restored(),
        "formats/markdown.rs should round-trip perfectly"
    );

    assert_snapshot!(result.migrate_stderr);
}

#[test]
fn roundtrip_formats_difftastic() {
    let temp = setup_test_project(&[
        ModuleConfig::new("formats"),
        ModuleConfig::with_submodules("formats", &["difftastic"]),
    ]);
    let result = run_roundtrip(temp.path());

    assert!(
        result.is_perfectly_restored(),
        "formats/difftastic.rs should round-trip perfectly"
    );

    assert_snapshot!(result.migrate_stderr);
}
