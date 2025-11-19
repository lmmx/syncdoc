// syncdoc-migrate/src/tests/diff/blank_lines.rs

use super::helpers::*;
use crate::rewrite::reformat::diff::apply::{apply_diff, apply_diff_restore};
use insta::assert_snapshot;

#[test]
fn preserves_blank_after_module_doc() {
    let original = r#"//! Module doc

use std::path::Path;"#;

    let transformed = r#"#![doc = syncdoc::module_doc!()]

use std::path::Path;"#;

    let hunks = compute_hunks(original, transformed);
    let result = apply_diff(original, &hunks, transformed);

    assert_snapshot!(result);
}

#[test]
fn preserves_blank_between_items() {
    let original = r#"pub struct Foo;

pub struct Bar;"#;

    let transformed = r#"#[syncdoc::omnidoc]
pub struct Foo;

#[syncdoc::omnidoc]
pub struct Bar;"#;

    let hunks = compute_hunks(original, transformed);
    let result = apply_diff(original, &hunks, transformed);

    // Count blank lines
    let blank_count = result.lines().filter(|l| l.trim().is_empty()).count();
    assert_eq!(blank_count, 1, "Should preserve blank line between items");
}

#[test]
fn removes_extra_blanks_during_restore() {
    let original = r#"#![doc = syncdoc::module_doc!()]


pub fn test() {}"#;

    let transformed = r#"//! Module doc

pub fn test() {}"#;

    let hunks = compute_hunks(original, transformed);
    let result = apply_diff_restore(original, &hunks, transformed);

    assert_snapshot!(result);
}

#[test]
fn full_module_blank_line_handling() {
    let case = full_module_doc();
    let hunks = case.compute_hunks();
    let result = apply_diff(case.original, &hunks, case.transformed);

    // Check that blank line appears after module doc
    let lines: Vec<&str> = result.lines().collect();
    let module_doc_idx = lines
        .iter()
        .position(|l| l.contains("module_doc"))
        .expect("Should have module_doc");

    assert_snapshot!(format!(
        "Line after module_doc (index {}): {:?}",
        module_doc_idx + 1,
        lines.get(module_doc_idx + 1)
    ));
}

#[test]
fn snapshot_blank_line_preservation() {
    let original = r#"//! Section representation for tree-sitter parsed documents.
//!
//! A section represents a hierarchical division of a document, typically
//! corresponding to a heading in markdown. Sections track their position
//! in the document tree through parent/child relationships and maintain
//! precise byte and line coordinates for content extraction and modification.

#[derive(Clone)]
/// Hierarchical document division with precise coordinates for extraction and modification.
pub struct Section {
    /// Section heading text without markup symbols.
    pub title: String,
}

/// What sort of hunk (syntactic diff atomic unit) it is.
#[derive(Clone)]
pub enum ChunkType {
    /// Only RHS exists
    Added,
    /// Only LHS exists
    Deleted,
}"#;

    let transformed = r#"#![doc = syncdoc::module_doc!()]
#[syncdoc::omnidoc]
#[derive(Clone)]
pub struct Section {
    pub title: String,
}
#[syncdoc::omnidoc]
#[derive(Clone)]
pub enum ChunkType {
    Added,
    Deleted,
}"#;

    let hunks = compute_hunks(original, transformed);
    let result = apply_diff(original, &hunks, transformed);

    assert_snapshot!(result, @r"
    #![doc = syncdoc::module_doc!()]

    #[syncdoc::omnidoc]
    #[derive(Clone)]
    pub struct Section {
        pub title: String,
    }

    #[syncdoc::omnidoc]
    #[derive(Clone)]
    pub enum ChunkType {
        Added,
        Deleted,
    }
    ");
}
