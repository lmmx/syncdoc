#![cfg(feature = "cli")]
use crate::helpers::{run_roundtrip, setup_test_project, ModuleConfig};
use insta::assert_snapshot;

#[test]
fn roundtrip_config() {
    let temp = setup_test_project(&[ModuleConfig::new("config")]);
    let result = run_roundtrip(temp.path());

    // Snapshot file lists
    assert_snapshot!(result.get_source_files_brace(), @"{config.rs,lib.rs}");
    assert_snapshot!(result.get_docs_files_brace());

    // Snapshot file contents in subdirectory
    result.snapshot_source_files("config");
    result.snapshot_docs_files("config");

    // Document round-trip status
    assert_snapshot!(
        "config_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences"
        }
    );
}

#[test]
fn roundtrip_edit_plan() {
    let temp = setup_test_project(&[ModuleConfig::new("edit_plan")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{edit_plan.rs,lib.rs}");
    assert_snapshot!(result.get_docs_files_brace());

    result.snapshot_source_files("edit_plan");
    result.snapshot_docs_files("edit_plan");

    assert_snapshot!(
        "edit_plan_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences"
        }
    );
}

#[test]
fn roundtrip_input() {
    let temp = setup_test_project(&[ModuleConfig::new("input")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{input.rs,lib.rs}");
    assert_snapshot!(result.get_docs_files_brace());

    result.snapshot_source_files("input");
    result.snapshot_docs_files("input");

    assert_snapshot!(
        "input_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences"
        }
    );
}

#[test]
fn roundtrip_formats_parent() {
    let temp = setup_test_project(&[ModuleConfig::new("formats")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{formats.rs,lib.rs}");
    assert_snapshot!(result.get_docs_files_brace());

    result.snapshot_source_files("formats_parent");
    result.snapshot_docs_files("formats_parent");

    assert_snapshot!(
        "formats_parent_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences"
        }
    );
}

#[test]
fn roundtrip_formats_markdown() {
    let temp = setup_test_project(&[
        ModuleConfig::new("formats"),
        ModuleConfig::with_submodules("formats", &["markdown"]),
    ]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{formats.rs,formats/markdown.rs,lib.rs}");
    assert_snapshot!(result.get_docs_files_brace());

    result.snapshot_source_files("formats_markdown");
    result.snapshot_docs_files("formats_markdown");

    assert_snapshot!(
        "formats_markdown_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences"
        }
    );
}

#[test]
fn roundtrip_formats_difftastic() {
    let temp = setup_test_project(&[
        ModuleConfig::new("formats"),
        ModuleConfig::with_submodules("formats", &["difftastic"]),
    ]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{formats.rs,formats/difftastic.rs,lib.rs}");
    assert_snapshot!(result.get_docs_files_brace());

    result.snapshot_source_files("formats_difftastic");
    result.snapshot_docs_files("formats_difftastic");

    assert_snapshot!(
        "formats_difftastic_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences"
        }
    );
}
