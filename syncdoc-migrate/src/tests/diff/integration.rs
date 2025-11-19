// syncdoc-migrate/src/tests/diff/integration.rs

use crate::config::DocsPathMode;
use crate::discover::parse_file;
use crate::restore_file;
use crate::rewrite_file;
use insta::assert_snapshot;
use std::fs;
use tempfile::TempDir;

fn setup_test_env(source: &str) -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.rs");
    fs::write(&file_path, source).unwrap();
    (temp, file_path)
}

#[test]
fn integration_full_roundtrip() {
    let original_source = r#"//! Section representation for tree-sitter parsed documents.
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

    // Step 1: Parse and migrate
    let (_temp, file_path) = setup_test_env(original_source);
    let parsed = parse_file(&file_path).unwrap();

    let migrated = rewrite_file(&parsed, "docs", DocsPathMode::TomlConfig, true, true)
        .expect("Migration should succeed");

    assert_snapshot!("1_migrated", migrated);

    // Step 2: Write migrated code and create docs
    fs::write(&file_path, &migrated).unwrap();

    // Create mock documentation files
    let docs_dir = _temp.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap();

    fs::write(
        docs_dir.join("test.md"),
        "Section representation for tree-sitter parsed documents.\n\nA section represents a hierarchical division of a document, typically\ncorresponding to a heading in markdown. Sections track their position\nin the document tree through parent/child relationships and maintain\nprecise byte and line coordinates for content extraction and modification.\n"
    ).unwrap();

    fs::write(
        docs_dir.join("Section.md"),
        "Hierarchical document division with precise coordinates for extraction and modification.\n"
    ).unwrap();

    let section_dir = docs_dir.join("Section");
    fs::create_dir_all(&section_dir).unwrap();
    fs::write(
        section_dir.join("title.md"),
        "Section heading text without markup symbols.\n",
    )
    .unwrap();

    fs::write(
        docs_dir.join("ChunkType.md"),
        "What sort of hunk (syntactic diff atomic unit) it is.\n",
    )
    .unwrap();

    let chunk_type_dir = docs_dir.join("ChunkType");
    fs::create_dir(&chunk_type_dir).unwrap();
    fs::write(chunk_type_dir.join("Added.md"), "Only RHS exists\n").unwrap();
    fs::write(chunk_type_dir.join("Deleted.md"), "Only LHS exists\n").unwrap();

    // Step 3: Parse migrated and restore
    let parsed_migrated = parse_file(&file_path).unwrap();
    let restored =
        restore_file(&parsed_migrated, docs_dir.to_str().unwrap()).expect("Restore should succeed");

    assert_snapshot!("2_restored", restored);

    // Step 4: Compare structure
    let orig_lines: Vec<&str> = original_source.lines().collect();
    let rest_lines: Vec<&str> = restored.lines().collect();

    let comparison = format!(
        "Original lines: {}\nRestored lines: {}\nMatch: {}",
        orig_lines.len(),
        rest_lines.len(),
        orig_lines.len() == rest_lines.len()
    );

    assert_snapshot!("3_comparison", comparison);
}

#[test]
fn integration_preserves_attributes() {
    let source = r#"#[derive(Debug, Clone)]
#[cfg(test)]
/// Documentation
pub struct Test {
    /// Field doc
    pub field: i32,
}"#;

    let (_temp, file_path) = setup_test_env(source);
    let parsed = parse_file(&file_path).unwrap();

    let migrated = rewrite_file(&parsed, "docs", DocsPathMode::TomlConfig, true, true)
        .expect("Migration should succeed");

    assert!(migrated.contains("#[derive(Debug, Clone)]"));
    assert!(migrated.contains("#[cfg(test)]"));
    assert!(migrated.contains("#[syncdoc::omnidoc]"));

    assert_snapshot!(migrated);
}
