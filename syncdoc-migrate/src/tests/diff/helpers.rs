// syncdoc-migrate/src/tests/diff/helpers.rs

use crate::rewrite::reformat::diff::{compute_line_diff, DiffHunk};

pub fn hunk(
    before_start: usize,
    before_count: usize,
    after_start: usize,
    after_count: usize,
) -> DiffHunk {
    DiffHunk {
        before_start,
        before_count,
        after_start,
        after_count,
    }
}

pub fn lines(text: &str) -> Vec<&str> {
    text.lines().collect()
}

pub fn compute_hunks(before: &str, after: &str) -> Vec<DiffHunk> {
    compute_line_diff(before, after)
}

// Helper to create test scenarios
pub struct TestCase {
    pub original: &'static str,
    pub transformed: &'static str,
}

impl TestCase {
    pub fn new(original: &'static str, transformed: &'static str) -> Self {
        Self {
            original,
            transformed,
        }
    }

    pub fn original_lines(&self) -> Vec<&str> {
        lines(self.original)
    }

    pub fn transformed_lines(&self) -> Vec<&str> {
        lines(self.transformed)
    }

    pub fn compute_hunks(&self) -> Vec<DiffHunk> {
        compute_hunks(self.original, self.transformed)
    }
}

// Common test cases
pub fn module_doc_to_macro() -> TestCase {
    TestCase::new(
        r#"//! Module documentation
//!
//! More details here

pub fn foo() {}"#,
        r#"#![doc = syncdoc::module_doc!()]

pub fn foo() {}"#,
    )
}

pub fn item_doc_to_omnidoc() -> TestCase {
    TestCase::new(
        r#"/// Item documentation
pub struct MyStruct;"#,
        r#"#[syncdoc::omnidoc]
pub struct MyStruct;"#,
    )
}

pub fn struct_fields_doc() -> TestCase {
    TestCase::new(
        r#"pub struct Section {
    /// Section heading text without markup symbols.
    pub title: String,
}"#,
        r#"#[syncdoc::omnidoc]
pub struct Section {
    pub title: String,
}"#,
    )
}

pub fn enum_variants_doc() -> TestCase {
    TestCase::new(
        r#"pub enum ChunkType {
    /// Only RHS exists
    Added,
    /// Only LHS exists
    Deleted,
}"#,
        r#"#[syncdoc::omnidoc]
pub enum ChunkType {
    Added,
    Deleted,
}"#,
    )
}

pub fn full_module_doc() -> TestCase {
    TestCase::new(
        r#"//! Section representation for tree-sitter parsed documents.
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
}"#,
        r#"#![doc = syncdoc::module_doc!()]
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
}"#,
    )
}
