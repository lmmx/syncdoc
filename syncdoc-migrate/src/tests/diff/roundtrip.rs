// syncdoc-migrate/src/tests/diff/roundtrip.rs

use super::helpers::*;
use crate::rewrite::reformat::diff::apply::{apply_diff, apply_diff_restore};
use insta::assert_snapshot;

#[test]
fn roundtrip_full_module_doc() {
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

    // Step 1: Migrate (original -> transformed)
    let migrated = r#"#![doc = syncdoc::module_doc!()]
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

    let migrate_hunks = compute_hunks(original, migrated);
    let migrate_result = apply_diff(original, &migrate_hunks, migrated);

    assert_snapshot!("after_migrate", migrate_result);

    // Step 2: Restore (transformed -> back to original docs)
    let restored_docs = r#"//! Section representation for tree-sitter parsed documents.
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

    let restore_hunks = compute_hunks(&migrate_result, restored_docs);
    let restore_result = apply_diff_restore(&migrate_result, &restore_hunks, restored_docs);

    assert_snapshot!("after_restore", restore_result);

    // Step 3: Verify round-trip preserves structure
    let original_lines: Vec<&str> = original.lines().collect();
    let restored_lines: Vec<&str> = restore_result.lines().collect();

    assert_eq!(
        original_lines.len(),
        restored_lines.len(),
        "Line count should match after round-trip"
    );

    // Check blank line preservation
    let original_blanks: Vec<_> = original_lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.trim().is_empty())
        .map(|(i, _)| i)
        .collect();

    let restored_blanks: Vec<_> = restored_lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.trim().is_empty())
        .map(|(i, _)| i)
        .collect();

    assert_snapshot!(
        "blank_line_comparison",
        format!(
            "Original blank lines at: {:?}\nRestored blank lines at: {:?}",
            original_blanks, restored_blanks
        )
    );
}

#[test]
fn roundtrip_preserves_blank_after_module_doc() {
    let original = r#"//! Module doc

pub fn test() {}"#;

    let migrated = r#"#![doc = syncdoc::module_doc!()]

pub fn test() {}"#;

    // Migrate
    let migrate_hunks = compute_hunks(original, migrated);
    let migrate_result = apply_diff(original, &migrate_hunks, migrated);

    // Restore
    let restored_docs = r#"//! Module doc

pub fn test() {}"#;

    let restore_hunks = compute_hunks(&migrate_result, restored_docs);
    let restore_result = apply_diff_restore(&migrate_result, &restore_hunks, restored_docs);

    assert_snapshot!(restore_result);

    // Verify blank line after module doc
    let lines: Vec<&str> = restore_result.lines().collect();
    assert!(
        lines.len() >= 2 && lines[1].trim().is_empty(),
        "Should have blank line after module doc"
    );
}

#[test]
fn roundtrip_multiple_blank_lines() {
    let original = r#"//! Module doc


pub struct A;

pub struct B;"#;

    let migrated = r#"#![doc = syncdoc::module_doc!()]

#[syncdoc::omnidoc]
pub struct A;

#[syncdoc::omnidoc]
pub struct B;"#;

    let migrate_hunks = compute_hunks(original, migrated);
    let migrate_result = apply_diff(original, &migrate_hunks, migrated);

    let restored_docs = r#"//! Module doc

pub struct A;

pub struct B;"#;

    let restore_hunks = compute_hunks(&migrate_result, restored_docs);
    let restore_result = apply_diff_restore(&migrate_result, &restore_hunks, restored_docs);

    assert_snapshot!(restore_result);
}

#[test]
fn roundtrip_complex_whitespace() {
    let original = r#"//! Module with complex spacing.
//!
//! Details here.

#[derive(Clone)]
/// Struct doc
pub struct Foo {
    /// Field
    pub x: i32,
}

/// Enum doc
#[derive(Debug)]
pub enum Bar {
    /// Variant
    A,
}

pub fn func() {}"#;

    let migrated = r#"#![doc = syncdoc::module_doc!()]
#[syncdoc::omnidoc]
#[derive(Clone)]
pub struct Foo {
    pub x: i32,
}
#[syncdoc::omnidoc]
#[derive(Debug)]
pub enum Bar {
    A,
}
#[syncdoc::omnidoc]
pub fn func() {}"#;

    let migrate_hunks = compute_hunks(original, migrated);
    let migrate_result = apply_diff(original, &migrate_hunks, migrated);

    assert_snapshot!("complex_migrate", migrate_result);

    let restored_docs = r#"//! Module with complex spacing.
//!
//! Details here.

#[derive(Clone)]
/// Struct doc
pub struct Foo {
    /// Field
    pub x: i32,
}

/// Enum doc
#[derive(Debug)]
pub enum Bar {
    /// Variant
    A,
}

pub fn func() {}"#;

    let restore_hunks = compute_hunks(&migrate_result, restored_docs);
    let restore_result = apply_diff_restore(&migrate_result, &restore_hunks, restored_docs);

    assert_snapshot!("complex_restore", restore_result);
}

#[test]
fn identifies_whitespace_discrepancies() {
    let original = full_module_doc().original;
    let transformed = full_module_doc().transformed;

    // Simulate full round-trip
    let migrate_hunks = compute_hunks(original, transformed);
    let migrate_result = apply_diff(original, &migrate_hunks, transformed);

    // Now restore back
    let restore_hunks = compute_hunks(&migrate_result, original);
    let restore_result = apply_diff_restore(&migrate_result, &restore_hunks, original);

    // Compare line by line
    let orig_lines: Vec<&str> = original.lines().collect();
    let rest_lines: Vec<&str> = restore_result.lines().collect();

    let mut differences = Vec::new();
    let max_len = orig_lines.len().max(rest_lines.len());

    for i in 0..max_len {
        let orig = orig_lines.get(i).unwrap_or(&"<missing>");
        let rest = rest_lines.get(i).unwrap_or(&"<missing>");

        if orig != rest {
            differences.push(format!(
                "Line {}: {:?} -> {:?}",
                i, orig, rest
            ));
        }
    }

    if !differences.is_empty() {
        assert_snapshot!("whitespace_diffs", differences.join("\n"));
    } else {
        assert_snapshot!("whitespace_diffs", "Perfect round-trip!");
    }
}
