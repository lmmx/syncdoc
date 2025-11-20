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

#[test]
fn preserves_blank_line_between_module_and_item_docs() {
    let original = r#"//! Module doc

/// Item doc
pub fn test() {}"#;

    let after = r#"#![doc = syncdoc::module_doc!()]

#[syncdoc::omnidoc]
pub fn test() {}"#;

    let hunks = compute_hunks(original, after);

    // Debug: print what hunks we got
    eprintln!("\n=== HUNKS ===");
    for (i, h) in hunks.iter().enumerate() {
        eprintln!(
            "Hunk {}: before[{}..{}] after[{}..{}]",
            i,
            h.before_start,
            h.before_start + h.before_count,
            h.after_start,
            h.after_start + h.after_count
        );
    }
    eprintln!("=============\n");

    let result = apply_diff(original, &hunks, after);

    // Debug: print the result
    eprintln!("\n=== RESULT ===");
    for (i, line) in result.lines().enumerate() {
        eprintln!("{}: {:?}", i, line);
    }
    eprintln!("==============\n");

    // The critical assertion: blank line must be preserved
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 4, "Should have 4 lines");
    assert!(
        lines[0].contains("module_doc"),
        "Line 0 should be module doc"
    );
    assert_eq!(lines[1].trim(), "", "Line 1 should be blank");
    assert!(lines[2].contains("omnidoc"), "Line 2 should be omnidoc");
    assert!(
        lines[3].contains("pub fn test"),
        "Line 3 should be function"
    );

    assert_snapshot!(result);
}

#[test]
fn restore_preserves_blank_line_between_module_and_item_docs() {
    let original = r#"#![doc = syncdoc::module_doc!()]

#[syncdoc::omnidoc]
pub fn test() {}"#;

    let after = r#"#[doc = "//! Module doc"]

#[doc = "/// Item doc"]
pub fn test() {}"#;

    let hunks = compute_hunks(original, after);

    eprintln!("\n=== RESTORE HUNKS ===");
    for (i, h) in hunks.iter().enumerate() {
        eprintln!(
            "Hunk {}: before[{}..{}] after[{}..{}]",
            i,
            h.before_start,
            h.before_start + h.before_count,
            h.after_start,
            h.after_start + h.after_count
        );
    }
    eprintln!("=====================\n");

    let result = apply_diff_restore(original, &hunks, after);

    eprintln!("\n=== RESTORE RESULT ===");
    for (i, line) in result.lines().enumerate() {
        eprintln!("{}: {:?}", i, line);
    }
    eprintln!("======================\n");

    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 4, "Should have 4 lines");
    assert!(
        lines[0].starts_with("//!"),
        "Line 0 should be module doc comment"
    );
    assert_eq!(lines[1].trim(), "", "Line 1 should be blank");
    assert!(
        lines[2].starts_with("///"),
        "Line 2 should be item doc comment"
    );
    assert!(
        lines[3].contains("pub fn test"),
        "Line 3 should be function"
    );

    assert_snapshot!(result);
}

#[test]
fn restore_preserves_blank_line_and_correct_doc_order() {
    // This is the ACTUAL bug case from your report
    let original = r#"#![doc = syncdoc::module_doc!()]

#[syncdoc::omnidoc]
#[derive(Clone)]
pub struct Section {}"#;

    let after = r#"#[doc = "//! Section representation for tree-sitter parsed documents."]
#[doc = "//!"]
#[doc = "//! A section represents a hierarchical division of a document, typically"]
#[doc = "//! corresponding to a heading in markdown. Sections track their position"]
#[doc = "//! in the document tree through parent/child relationships and maintain"]
#[doc = "//! precise byte and line coordinates for content extraction and modification."]

#[doc = "/// Hierarchical document division with precise coordinates for extraction and modification."]
#[derive(Clone)]
pub struct Section {}"#;

    let hunks = compute_hunks(original, after);

    eprintln!("\n=== RESTORE HUNKS (ACTUAL BUG) ===");
    for (i, h) in hunks.iter().enumerate() {
        eprintln!(
            "Hunk {}: before[{}..{}] after[{}..{}]",
            i,
            h.before_start,
            h.before_start + h.before_count,
            h.after_start,
            h.after_start + h.after_count
        );

        eprintln!("  Before lines:");
        let orig_lines: Vec<&str> = original.lines().collect();
        for j in h.before_start..(h.before_start + h.before_count).min(orig_lines.len()) {
            eprintln!("    [{}]: {:?}", j, orig_lines[j]);
        }

        eprintln!("  After lines:");
        let after_lines: Vec<&str> = after.lines().collect();
        for j in h.after_start..(h.after_start + h.after_count).min(after_lines.len()) {
            eprintln!("    [{}]: {:?}", j, after_lines[j]);
        }
    }
    eprintln!("===================================\n");

    let result = apply_diff_restore(original, &hunks, after);

    eprintln!("\n=== RESTORE RESULT (ACTUAL BUG) ===");
    for (i, line) in result.lines().enumerate() {
        eprintln!("{}: {:?}", i, line);
    }
    eprintln!("====================================\n");

    let lines: Vec<&str> = result.lines().collect();

    // CRITICAL: The blank line should separate module docs from item docs
    // Find the last module doc line
    let last_module_doc = lines
        .iter()
        .rposition(|l| l.starts_with("//!"))
        .expect("Should have module docs");

    // Find the first item doc line
    let first_item_doc = lines
        .iter()
        .position(|l| l.starts_with("///"))
        .expect("Should have item docs");

    // There should be exactly one blank line between them
    assert!(
        first_item_doc > last_module_doc + 1,
        "Item doc should come after module doc with space"
    );
    assert_eq!(
        lines[last_module_doc + 1].trim(),
        "",
        "Should have blank line after module docs"
    );

    // CRITICAL: Item doc should come BEFORE #[derive(Clone)]
    let derive_line = lines
        .iter()
        .position(|l| l.contains("derive(Clone)"))
        .expect("Should have derive");

    assert!(
        first_item_doc < derive_line,
        "Item doc should come BEFORE #[derive], not after! Got item doc at {} and derive at {}",
        first_item_doc,
        derive_line
    );

    assert_snapshot!(result);
}

#[test]
fn restore_real_bug_case_section_struct() {
    // This is the EXACT bug case from the real file
    let original = r#"#![doc = syncdoc::module_doc!()]

#[syncdoc::omnidoc]
#[derive(Clone)]
pub struct Section {
    pub title: String,
    pub level: usize,
    pub line_start: i64,
    pub line_end: i64,
    pub column_start: i64,
    pub column_end: i64,
    pub byte_start: usize,
    pub byte_end: usize,
    pub file_path: String,
    pub parent_index: Option<usize>,
    pub children_indices: Vec<usize>,
    pub section_content: Option<Vec<String>>,
    pub chunk_type: Option<ChunkType>,
    pub lhs_content: Option<String>,
    pub rhs_content: Option<String>,
}

#[syncdoc::omnidoc]
#[derive(Clone)]
pub enum ChunkType {
    Added,
    Deleted,
    Modified,
    Unchanged,
}

#[syncdoc::omnidoc]
#[derive(Clone)]
pub enum NodeType {
    Directory { name: String, path: String },
    File { name: String, path: String },
    Section(Section),
}

#[syncdoc::omnidoc]
#[derive(Clone)]
pub struct TreeNode {
    pub node_type: NodeType,
    pub tree_level: usize,
    pub navigable: bool,
    pub section_index: Option<usize>,
}

impl TreeNode {
    #[syncdoc::omnidoc]
    #[must_use]
    pub fn directory(name: String, path: String, tree_level: usize) -> Self {
        Self {
            node_type: NodeType::Directory { name, path },
            tree_level,
            navigable: false,
            section_index: None,
        }
    }

    #[syncdoc::omnidoc]
    #[must_use]
    pub fn file(name: String, path: String, tree_level: usize) -> Self {
        Self {
            node_type: NodeType::File { name, path },
            tree_level,
            navigable: false,
            section_index: None,
        }
    }

    #[syncdoc::omnidoc]
    #[must_use]
    pub fn section(section: Section, tree_level: usize, section_index: usize) -> Self {
        Self {
            node_type: NodeType::Section(section),
            tree_level,
            navigable: true,
            section_index: Some(section_index),
        }
    }
}"#;

    // After restore, this is what the generated code looks like (with doc attributes)
    let after = r#"#[doc = "//! Section representation for tree-sitter parsed documents."]
#[doc = "//!"]
#[doc = "//! A section represents a hierarchical division of a document, typically"]
#[doc = "//! corresponding to a heading in markdown. Sections track their position"]
#[doc = "//! in the document tree through parent/child relationships and maintain"]
#[doc = "//! precise byte and line coordinates for content extraction and modification."]

#[doc = "/// Hierarchical document division with precise coordinates for extraction and modification."]
#[derive(Clone)]
pub struct Section {
    #[doc = "/// Section heading text without markup symbols."]
    pub title: String,
    #[doc = "/// Nesting depth in the document hierarchy (1 for top-level)."]
    pub level: usize,
    #[doc = "/// First line of section content (after the heading)."]
    pub line_start: i64,
    #[doc = "/// Line where the next section begins or file ends."]
    pub line_end: i64,
    #[doc = "/// Starting column of the section heading."]
    pub column_start: i64,
    #[doc = "/// Ending column of the section heading."]
    pub column_end: i64,
    #[doc = "/// Byte offset where section content begins."]
    pub byte_start: usize,
    #[doc = "/// Byte offset where section content ends."]
    pub byte_end: usize,
    #[doc = "/// Source file containing this section."]
    pub file_path: String,
    #[doc = "/// Index of the containing section in the hierarchy."]
    pub parent_index: Option<usize>,
    #[doc = "/// Indices of directly nested subsections."]
    pub children_indices: Vec<usize>,
    #[doc = "/// Edited content for this section (if modified)"]
    pub section_content: Option<Vec<String>>,
    #[doc = "/// The chunk type (for diffs)"]
    pub chunk_type: Option<ChunkType>,
    #[doc = "/// The LHS (for diffs)"]
    pub lhs_content: Option<String>,
    #[doc = "/// The RHS (for diffs)"]
    pub rhs_content: Option<String>,
}

#[doc = "/// What sort of hunk (syntactic diff atomic unit) it is."]
#[derive(Clone)]
pub enum ChunkType {
    #[doc = "/// Only RHS exists"]
    Added,
    #[doc = "/// Only LHS exists"]
    Deleted,
    #[doc = "/// Both LHS and RHS exist (and differ)"]
    Modified,
    #[doc = "/// Both LHS and RHS exist (and are the same, at least syntactically)"]
    Unchanged,
}

#[doc = "/// Types of nodes that can appear in the file tree view."]
#[derive(Clone)]
pub enum NodeType {
    #[doc = "/// Directory node showing a path component"]
    Directory {
        #[doc = "/// Directory name (not full path)"]
        name: String,
        #[doc = "/// Full path for reference"]
        path: String,
    },
    #[doc = "/// File node (non-navigable, just shows filename)"]
    File {
        #[doc = "/// File name"]
        name: String,
        #[doc = "/// Full path for reference"]
        path: String,
    },
    #[doc = "/// Actual document section (navigable)"]
    Section(Section),
}

#[doc = "/// A node in the unified file + section tree."]
#[derive(Clone)]
pub struct TreeNode {
    #[doc = "/// The type of node (directory, file, or section)"]
    pub node_type: NodeType,
    #[doc = "/// Nesting level in the tree (for indentation/box-drawing)"]
    pub tree_level: usize,
    #[doc = "/// Whether this node can be selected and edited"]
    pub navigable: bool,
    #[doc = "/// Index of the actual section if this is a Section node"]
    pub section_index: Option<usize>,
}

impl TreeNode {
    #[doc = "/// Create a directory node"]
    #[must_use]
    pub fn directory(name: String, path: String, tree_level: usize) -> Self {
        Self {
            node_type: NodeType::Directory { name, path },
            tree_level,
            navigable: false,
            section_index: None,
        }
    }

    #[doc = "/// Create a file node"]
    #[must_use]
    pub fn file(name: String, path: String, tree_level: usize) -> Self {
        Self {
            node_type: NodeType::File { name, path },
            tree_level,
            navigable: false,
            section_index: None,
        }
    }

    #[doc = "/// Create a section node"]
    #[must_use]
    pub fn section(section: Section, tree_level: usize, section_index: usize) -> Self {
        Self {
            node_type: NodeType::Section(section),
            tree_level,
            navigable: true,
            section_index: Some(section_index),
        }
    }
}"#;

    let hunks = compute_hunks(original, after);

    eprintln!("\n=== REAL BUG HUNKS ===");
    eprintln!("Total hunks: {}", hunks.len());
    for (i, h) in hunks.iter().enumerate() {
        eprintln!(
            "Hunk {}: before[{}..{}] after[{}..{}]",
            i,
            h.before_start,
            h.before_start + h.before_count,
            h.after_start,
            h.after_start + h.after_count
        );
    }
    eprintln!("======================\n");

    let result = apply_diff_restore(original, &hunks, after);

    eprintln!("\n=== REAL BUG RESULT (first 20 lines) ===");
    for (i, line) in result.lines().take(20).enumerate() {
        eprintln!("{}: {:?}", i, line);
    }
    eprintln!("=========================================\n");

    let lines: Vec<&str> = result.lines().collect();

    // Find critical positions
    let last_module_doc = lines
        .iter()
        .rposition(|l| l.starts_with("//!"))
        .expect("Should have module docs");

    let first_item_doc = lines
        .iter()
        .position(|l| l.starts_with("///"))
        .expect("Should have item docs");

    let derive_line = lines
        .iter()
        .position(|l| l.contains("#[derive(Clone)]"))
        .expect("Should have derive");

    eprintln!("Last module doc line: {}", last_module_doc);
    eprintln!("First item doc line: {}", first_item_doc);
    eprintln!("Derive line: {}", derive_line);

    // CRITICAL ASSERTIONS
    assert!(
        lines[last_module_doc + 1].trim().is_empty(),
        "Should have blank line after module docs at line {}",
        last_module_doc + 1
    );

    assert!(
        first_item_doc < derive_line,
        "Item doc (line {}) should come BEFORE #[derive] (line {})",
        first_item_doc,
        derive_line
    );

    assert_snapshot!(result);
}
