#![cfg(feature = "cli")]
use crate::helpers::{run_roundtrip, setup_test_project, ModuleConfig};
use insta::assert_snapshot;

#[test]
fn roundtrip_app_state_whitespace_issue() {
    let temp = setup_test_project(&[ModuleConfig::new("app_state")]);
    let result = run_roundtrip(temp.path());

    // Known issue: removes whitespace
    let diff = result.get_file_diff("app_state.rs").unwrap();

    assert_snapshot!("app_state_migrate_stderr", result.migrate_stderr);
    assert_snapshot!("app_state_restore_stderr", result.restore_stderr);

    // Document the whitespace difference
    if !diff.matches {
        let original_lines: Vec<&str> = diff.original.lines().collect();
        let restored_lines: Vec<&str> = diff.restored.lines().collect();

        assert_snapshot!(
            "app_state_line_count_diff",
            format!(
                "Original: {} lines\nRestored: {} lines\nDifference: {}",
                original_lines.len(),
                restored_lines.len(),
                original_lines.len() as i32 - restored_lines.len() as i32
            )
        );
    }
}

#[test]
fn roundtrip_highlight_omnidoc_added() {
    let temp = setup_test_project(&[ModuleConfig::new("highlight")]);
    let result = run_roundtrip(temp.path());

    // Known issue: adds omnidoc where previously undocumented
    let diff = result.get_file_diff("highlight.rs").unwrap();

    assert_snapshot!("highlight_migrate_stderr", result.migrate_stderr);
    assert_snapshot!("highlight_restore_stderr", result.restore_stderr);

    // Check if omnidoc was added
    let has_omnidoc_original = diff.original.contains("#[syncdoc::omnidoc]");
    let has_omnidoc_restored = diff.restored.contains("#[syncdoc::omnidoc]");

    assert_snapshot!(
        "highlight_omnidoc_status",
        format!(
            "Original has omnidoc: {}\nRestored has omnidoc: {}",
            has_omnidoc_original, has_omnidoc_restored
        )
    );
}

#[test]
fn roundtrip_section_completely_unrestored() {
    let temp = setup_test_project(&[ModuleConfig::new("section")]);
    let result = run_roundtrip(temp.path());

    // Known issue: completely fails to restore
    assert!(!result.is_perfectly_restored());

    let diff = result.get_file_diff("section.rs").unwrap();

    assert_snapshot!("section_migrate_stderr", result.migrate_stderr);
    assert_snapshot!("section_restore_stderr", result.restore_stderr);
    assert_snapshot!("section_original", diff.original);
    assert_snapshot!("section_restored", diff.restored);
}

#[test]
fn roundtrip_ui_completely_unrestored() {
    let temp = setup_test_project(&[ModuleConfig::new("ui")]);
    let result = run_roundtrip(temp.path());

    // Known issue: completely fails to restore
    assert!(!result.is_perfectly_restored());

    let diff = result.get_file_diff("ui.rs").unwrap();

    assert_snapshot!("ui_migrate_stderr", result.migrate_stderr);
    assert_snapshot!("ui_restore_stderr", result.restore_stderr);
    assert_snapshot!("ui_original", diff.original);
    assert_snapshot!("ui_restored", diff.restored);
}
