// syncdoc-migrate/src/tests/diff/apply.rs

use super::helpers::*;
use crate::rewrite::reformat::diff::apply::{apply_diff, apply_diff_restore};
use insta::assert_snapshot;

// Move existing tests from diff.rs here and add new ones

#[test]
fn applies_doc_only_changes() {
    let original = r#"/// Old doc
pub fn test() {}"#;

    let after = r#"/// New doc
pub fn test() {}"#;

    let hunks = compute_hunks(original, after);
    let result = apply_diff(original, &hunks, after);

    assert_snapshot!(result);
}

#[test]
fn applies_multiple_hunks() {
    let original = r#"/// Doc A
pub fn a() {}

/// Doc B
pub fn b() {}"#;

    let after = r#"#[syncdoc::omnidoc]
pub fn a() {}

#[syncdoc::omnidoc]
pub fn b() {}"#;

    let hunks = compute_hunks(original, after);
    let result = apply_diff(original, &hunks, after);

    assert!(result.contains("omnidoc"));
    assert_eq!(result.matches("omnidoc").count(), 2);
}

#[test]
fn applies_module_doc_change() {
    let original = r#"//! Module doc
//! More docs

pub fn test() {}"#;

    let after = r#"#![doc = syncdoc::module_doc!()]

pub fn test() {}"#;

    let hunks = compute_hunks(original, after);
    let result = apply_diff(original, &hunks, after);

    assert_snapshot!(result);
}

#[test]
fn restore_applies_doc_comments() {
    let original = r#"#[syncdoc::omnidoc]
pub fn test() {}"#;

    let after = r#"/// Documentation
pub fn test() {}"#;

    let hunks = compute_hunks(original, after);
    let result = apply_diff_restore(original, &hunks, after);

    assert_snapshot!(result);
}

#[test]
fn restore_strips_bookends() {
    let original = r#"#[syncdoc::omnidoc]
pub fn test() {}"#;

    let after = r#"#[doc = "/// Documentation"]
pub fn test() {}"#;

    let hunks = compute_hunks(original, after);
    let result = apply_diff_restore(original, &hunks, after);

    // Should convert #[doc = "/// ..."] to /// ...
    assert!(result.contains("/// Documentation"));
    assert!(!result.contains("#[doc"));
}

#[test]
fn applies_hunks_in_order() {
    let original = r#"//! Module

/// Item A
pub fn a() {}

/// Item B
pub fn b() {}"#;

    let after = r#"#![doc = syncdoc::module_doc!()]

#[syncdoc::omnidoc]
pub fn a() {}

#[syncdoc::omnidoc]
pub fn b() {}"#;

    let hunks = compute_hunks(original, after);
    let result = apply_diff(original, &hunks, after);

    // Verify order: module doc first, then items
    let lines: Vec<&str> = result.lines().collect();
    let module_doc_idx = lines
        .iter()
        .position(|l| l.contains("module_doc"))
        .expect("Should have module_doc");

    let first_omnidoc = lines
        .iter()
        .position(|l| l.contains("omnidoc"))
        .expect("Should have omnidoc");

    assert!(
        module_doc_idx < first_omnidoc,
        "Module doc should come before item attributes"
    );
}

#[test]
fn snapshot_full_apply() {
    let case = full_module_doc();
    let hunks = case.compute_hunks();
    let result = apply_diff(case.original, &hunks, case.transformed);

    assert_snapshot!(result);
}
