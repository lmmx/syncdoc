// syncdoc-migrate/src/tests/diff/doc_related.rs

use super::helpers::*;
use crate::rewrite::reformat::diff::hunk::is_doc_related_hunk;
use insta::assert_snapshot;

#[test]
fn detects_doc_comment_removal() {
    let case = TestCase::new(
        r#"/// Documentation
pub fn test() {}"#,
        r#"pub fn test() {}"#,
    );

    let hunks = case.compute_hunks();
    assert_eq!(hunks.len(), 1);

    let is_doc = is_doc_related_hunk(&hunks[0], &case.original_lines(), &case.transformed_lines());

    assert!(is_doc, "Should detect doc comment removal");
}

#[test]
fn detects_module_doc_removal() {
    let case = TestCase::new(
        r#"//! Module doc
pub fn test() {}"#,
        r#"pub fn test() {}"#,
    );

    let hunks = case.compute_hunks();
    let is_doc = is_doc_related_hunk(&hunks[0], &case.original_lines(), &case.transformed_lines());

    assert!(is_doc, "Should detect module doc removal");
}

#[test]
fn detects_omnidoc_addition() {
    let case = item_doc_to_omnidoc();

    let hunks = case.compute_hunks();
    let is_doc = is_doc_related_hunk(&hunks[0], &case.original_lines(), &case.transformed_lines());

    assert!(is_doc, "Should detect omnidoc addition");
}

#[test]
fn detects_module_doc_macro_addition() {
    let case = module_doc_to_macro();

    let hunks = case.compute_hunks();
    let is_doc = is_doc_related_hunk(&hunks[0], &case.original_lines(), &case.transformed_lines());

    assert!(is_doc, "Should detect module_doc macro addition");
}

#[test]
fn ignores_non_doc_changes() {
    let case = TestCase::new(
        r#"#[derive(Debug)]
pub fn test() {}"#,
        r#"#[derive(Clone)]
pub fn test() {}"#,
    );

    let hunks = case.compute_hunks();
    if !hunks.is_empty() {
        let is_doc =
            is_doc_related_hunk(&hunks[0], &case.original_lines(), &case.transformed_lines());

        assert!(!is_doc, "Should ignore non-doc attribute changes");
    }
}

#[test]
fn detects_field_doc_removal() {
    let case = struct_fields_doc();

    let hunks = case.compute_hunks();

    // Find the hunk that removes field documentation
    let doc_hunks: Vec<_> = hunks
        .iter()
        .filter(|h| is_doc_related_hunk(h, &case.original_lines(), &case.transformed_lines()))
        .collect();

    assert!(
        !doc_hunks.is_empty(),
        "Should detect field documentation changes"
    );
}

#[test]
fn detects_variant_doc_removal() {
    let case = enum_variants_doc();

    let hunks = case.compute_hunks();

    let doc_hunks: Vec<_> = hunks
        .iter()
        .filter(|h| is_doc_related_hunk(h, &case.original_lines(), &case.transformed_lines()))
        .collect();

    assert!(
        !doc_hunks.is_empty(),
        "Should detect variant documentation changes"
    );
}

#[test]
fn snapshot_doc_related_hunks() {
    let case = full_module_doc();
    let hunks = case.compute_hunks();

    let results: Vec<_> = hunks
        .iter()
        .map(|h| {
            let is_doc = is_doc_related_hunk(h, &case.original_lines(), &case.transformed_lines());
            format!(
                "hunk[{}..{}]->[{}..{}]: {}",
                h.before_start,
                h.before_start + h.before_count,
                h.after_start,
                h.after_start + h.after_count,
                if is_doc { "DOC" } else { "NON-DOC" }
            )
        })
        .collect();

    assert_snapshot!(results.join("\n"));
}
