#![cfg(feature = "cli")]
use crate::helpers::{run_roundtrip, setup_test_project, ModuleConfig};
use insta::assert_snapshot;

#[test]
fn roundtrip_app_state() {
    let temp = setup_test_project(&[ModuleConfig::new("app_state")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{app_state.rs,lib.rs}");
    assert_snapshot!(result.get_docs_files_brace());

    result.snapshot_source_files("app_state");
    result.snapshot_docs_files("app_state");

    assert_snapshot!(
        "app_state_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences (whitespace changes)"
        }
    );

    // Extra diagnostic: line count difference
    if let Some(diff) = result.get_file_diff("app_state.rs") {
        let original_lines = diff.original.lines().count();
        let restored_lines = diff.restored.lines().count();
        assert_snapshot!(
            "app_state_line_diff",
            format!(
                "Original: {} lines\nRestored: {} lines\nDiff: {}",
                original_lines,
                restored_lines,
                original_lines as i32 - restored_lines as i32
            )
        );
    }
}

#[test]
fn roundtrip_highlight() {
    let temp = setup_test_project(&[ModuleConfig::new("highlight")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{highlight.rs,lib.rs}");
    assert_snapshot!(result.get_docs_files_brace());

    result.snapshot_source_files("highlight");
    result.snapshot_docs_files("highlight");

    assert_snapshot!(
        "highlight_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences (omnidoc added)"
        }
    );

    // Extra diagnostic: omnidoc presence
    if let Some(diff) = result.get_file_diff("highlight.rs") {
        let original_omnidoc = diff.original.matches("#[syncdoc::omnidoc]").count();
        let restored_omnidoc = diff.restored.matches("#[syncdoc::omnidoc]").count();
        assert_snapshot!(
            "highlight_omnidoc_diff",
            format!(
                "Original omnidoc attrs: {}\nRestored omnidoc attrs: {}\nDiff: {}",
                original_omnidoc,
                restored_omnidoc,
                restored_omnidoc as i32 - original_omnidoc as i32
            )
        );
    }
}

#[test]
fn roundtrip_section() {
    let temp = setup_test_project(&[ModuleConfig::new("section")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{lib.rs,section.rs}");
    assert_snapshot!(result.get_docs_files_brace());

    result.snapshot_source_files("section");
    result.snapshot_docs_files("section");

    assert_snapshot!(
        "section_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences (major restoration issues)"
        }
    );
}

#[test]
fn roundtrip_ui() {
    let temp = setup_test_project(&[ModuleConfig::new("ui")]);
    let result = run_roundtrip(temp.path());

    assert_snapshot!(result.get_source_files_brace(), @"{lib.rs,ui.rs}");
    assert_snapshot!(result.get_docs_files_brace());

    result.snapshot_source_files("ui");
    result.snapshot_docs_files("ui");

    assert_snapshot!(
        "ui_roundtrip_status",
        if result.is_perfectly_restored() {
            "PERFECT: Round-trip successful"
        } else {
            "IMPERFECT: Round-trip has differences (major restoration issues)"
        }
    );
}
