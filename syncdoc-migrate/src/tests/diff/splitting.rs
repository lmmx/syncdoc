// syncdoc-migrate/src/tests/diff/split.rs

use super::helpers::*;
use crate::rewrite::reformat::diff::hunk::split_hunk_if_mixed;
use insta::assert_snapshot;

#[test]
fn splits_module_and_item_docs() {
    let case = mixed_module_and_item();
    let hunks = case.compute_hunks();

    assert_eq!(hunks.len(), 1, "Should start with one hunk");

    let split = split_hunk_if_mixed(&hunks[0], &case.transformed_lines());

    assert_snapshot!(format!(
        "Original hunks: {}\nSplit hunks: {}",
        hunks.len(),
        split.len()
    ));
}

#[test]
fn preserves_unsplittable_hunks() {
    let case = item_doc_to_omnidoc();
    let hunks = case.compute_hunks();

    if !hunks.is_empty() {
        let split = split_hunk_if_mixed(&hunks[0], &case.transformed_lines());
        assert_eq!(split.len(), 1, "Item-only hunk should not be split");
    }
}

#[test]
fn splits_complex_module_doc() {
    let case = full_module_doc();
    let hunks = case.compute_hunks();

    let all_splits: Vec<_> = hunks
        .iter()
        .flat_map(|h| split_hunk_if_mixed(h, &case.transformed_lines()))
        .collect();

    let summary: Vec<_> = all_splits
        .iter()
        .map(|h| {
            format!(
                "[{}..{}]->[{}..{}]",
                h.before_start,
                h.before_start + h.before_count,
                h.after_start,
                h.after_start + h.after_count
            )
        })
        .collect();

    assert_snapshot!(summary.join("\n"));
}

#[test]
fn handles_deletion_only_hunks() {
    let hunk = hunk(0, 5, 0, 0); // Deletes 5 lines, adds nothing
    let after_lines = vec![];

    let split = split_hunk_if_mixed(&hunk, &after_lines);
    assert_eq!(split.len(), 1, "Deletion-only hunks shouldn't split");
}

#[test]
fn handles_insertion_only_hunks() {
    let hunk = hunk(0, 0, 0, 3); // Adds 3 lines, deletes nothing
    let after_lines = vec!["line1", "line2", "line3"];

    let split = split_hunk_if_mixed(&hunk, &after_lines);
    assert_eq!(split.len(), 1, "Insertion-only hunks shouldn't split");
}

#[test]
fn splits_on_attribute_boundary() {
    let transformed = r#"#![doc = syncdoc::module_doc!()]
#[syncdoc::omnidoc]
pub struct Test;"#;

    let after_lines = lines(transformed);

    // Simulate a hunk that spans module doc and item attribute
    let hunk = hunk(0, 2, 0, 3);

    let split = split_hunk_if_mixed(&hunk, &after_lines);

    let summary: Vec<_> = split
        .iter()
        .map(|h| {
            format!(
                "before[{}..{}] after[{}..{}]",
                h.before_start,
                h.before_start + h.before_count,
                h.after_start,
                h.after_start + h.after_count
            )
        })
        .collect();

    assert_snapshot!(summary.join("\n"));
}
