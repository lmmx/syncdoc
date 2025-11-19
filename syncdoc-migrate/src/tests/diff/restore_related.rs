// syncdoc-migrate/src/tests/diff/restore_related.rs

use super::helpers::*;
use crate::rewrite::reformat::diff::hunk::is_restore_related_hunk;
use insta::assert_snapshot;

#[test]
fn detects_omnidoc_removal() {
    let case = TestCase::new(
        r#"#[syncdoc::omnidoc]
pub fn test() {}"#,
        r#"/// Documentation
pub fn test() {}"#,
    );

    let hunks = case.compute_hunks();
    assert!(!hunks.is_empty());

    let is_restore =
        is_restore_related_hunk(&hunks[0], &case.original_lines(), &case.transformed_lines());

    assert!(is_restore, "Should detect omnidoc removal during restore");
}

#[test]
fn detects_module_doc_macro_removal() {
    let case = TestCase::new(
        r#"#![doc = syncdoc::module_doc!()]
pub fn test() {}"#,
        r#"//! Module documentation
pub fn test() {}"#,
    );

    let hunks = case.compute_hunks();
    let is_restore =
        is_restore_related_hunk(&hunks[0], &case.original_lines(), &case.transformed_lines());

    assert!(is_restore, "Should detect module_doc macro removal");
}

#[test]
fn detects_doc_comment_addition() {
    let case = TestCase::new(
        r#"#[syncdoc::omnidoc]
pub struct MyStruct;"#,
        r#"/// Struct docs
pub struct MyStruct;"#,
    );

    let hunks = case.compute_hunks();
    let is_restore =
        is_restore_related_hunk(&hunks[0], &case.original_lines(), &case.transformed_lines());

    assert!(is_restore, "Should detect doc comment addition");
}

#[test]
fn detects_doc_attr_addition() {
    let case = TestCase::new(
        r#"#[syncdoc::omnidoc]
pub fn test() {}"#,
        r#"#[doc = "/// Function docs"]
pub fn test() {}"#,
    );

    let hunks = case.compute_hunks();
    let is_restore =
        is_restore_related_hunk(&hunks[0], &case.original_lines(), &case.transformed_lines());

    assert!(is_restore, "Should detect doc attribute addition");
}

#[test]
fn ignores_non_restore_changes() {
    let case = TestCase::new(
        r#"#[derive(Debug)]
pub fn test() {}"#,
        r#"#[derive(Clone)]
pub fn test() {}"#,
    );

    let hunks = case.compute_hunks();
    if !hunks.is_empty() {
        let is_restore =
            is_restore_related_hunk(&hunks[0], &case.original_lines(), &case.transformed_lines());

        assert!(!is_restore, "Should ignore non-restore changes");
    }
}

#[test]
fn snapshot_restore_hunks() {
    let case = TestCase::new(
        r#"#![doc = syncdoc::module_doc!()]
#[syncdoc::omnidoc]
pub struct Section {
    pub title: String,
}
#[syncdoc::omnidoc]
pub enum ChunkType {
    Added,
    Deleted,
}"#,
        r#"//! Module doc

/// Struct doc
pub struct Section {
    /// Field doc
    pub title: String,
}

/// Enum doc
pub enum ChunkType {
    /// Variant A
    Added,
    /// Variant B
    Deleted,
}"#,
    );

    let hunks = case.compute_hunks();
    let results: Vec<_> = hunks
        .iter()
        .map(|h| {
            let is_restore =
                is_restore_related_hunk(h, &case.original_lines(), &case.transformed_lines());
            format!(
                "hunk[{}..{}]->[{}..{}]: {}",
                h.before_start,
                h.before_start + h.before_count,
                h.after_start,
                h.after_start + h.after_count,
                if is_restore { "RESTORE" } else { "OTHER" }
            )
        })
        .collect();

    assert_snapshot!(results.join("\n"));
}
