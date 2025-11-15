// syncdoc-migrate/tests/touch_integration.rs

use std::fs;
use syncdoc_migrate::{
    discover::parse_file,
    write::{extract_all_docs, find_expected_doc_paths, write_extractions},
};
use tempfile::TempDir;

fn setup_test_file(source: &str, filename: &str) -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, source).unwrap();
    (temp_dir, file_path)
}

#[test]
fn test_extract_docs_and_touch_missing_files() {
    // This is the regression test: a struct with one documented field and one undocumented field
    let source = r#"//! Module docstring

/// Docstring for Foo
pub struct Foo {
    /// Docstring for field A
    pub a: i32,
    pub b: i32,  // No docstring - this should be touched
}
"#;

    let (temp_dir, file_path) = setup_test_file(source, "lib.rs");
    let docs_dir = temp_dir.path().join("docs");
    let docs_root = docs_dir.to_str().unwrap();

    let parsed = parse_file(&file_path).unwrap();

    // Extract actual docs (those with content)
    let mut all_extractions = extract_all_docs(&parsed, docs_root);

    // Should have: lib.md (module), Foo.md (struct), Foo/a.md (field a)
    assert_eq!(
        all_extractions.len(),
        3,
        "Should extract 3 docs with content"
    );

    // Verify the docs have actual content
    let foo_doc = all_extractions
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap().ends_with("Foo.md"))
        .expect("Should find Foo.md");
    assert_eq!(foo_doc.content, "Docstring for Foo\n");

    let field_a_doc = all_extractions
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap().ends_with("Foo/a.md"))
        .expect("Should find Foo/a.md");
    assert_eq!(field_a_doc.content, "Docstring for field A\n");

    // Now find expected paths (all items that should have docs)
    let expected_paths = find_expected_doc_paths(&parsed, docs_root);

    // Should find: lib.md, Foo.md, Foo/a.md, Foo/b.md
    assert_eq!(expected_paths.len(), 4, "Should find 4 expected paths");

    // Simulate the touch logic: filter to only those we don't have content for
    let existing_paths: std::collections::HashSet<_> =
        all_extractions.iter().map(|e| &e.markdown_path).collect();

    let missing_paths: Vec<_> = expected_paths
        .into_iter()
        .filter(|extraction| {
            !existing_paths.contains(&extraction.markdown_path)
                && !extraction.markdown_path.exists()
        })
        .collect();

    // Should only find Foo/b.md as missing
    assert_eq!(missing_paths.len(), 1, "Should find 1 missing path");

    let missing = &missing_paths[0];
    assert!(missing
        .markdown_path
        .to_str()
        .unwrap()
        .ends_with("Foo/b.md"));
    assert_eq!(
        missing.content, "\n",
        "Missing file should have empty content (just newline)"
    );

    // Add missing paths to all extractions
    all_extractions.extend(missing_paths);

    // Now write everything
    let report = write_extractions(&all_extractions, false).unwrap();

    assert_eq!(report.files_written, 4, "Should write 4 files total");
    assert_eq!(report.files_skipped, 0);
    assert!(report.errors.is_empty());

    // Verify the files exist and have correct content
    assert!(docs_dir.join("lib.md").exists());
    assert!(docs_dir.join("Foo.md").exists());
    assert!(docs_dir.join("Foo/a.md").exists());
    assert!(docs_dir.join("Foo/b.md").exists());

    // Verify content of documented files
    let foo_content = fs::read_to_string(docs_dir.join("Foo.md")).unwrap();
    assert_eq!(foo_content, "Docstring for Foo\n");

    let field_a_content = fs::read_to_string(docs_dir.join("Foo/a.md")).unwrap();
    assert_eq!(field_a_content, "Docstring for field A\n");

    // Verify touched file is empty (just newline)
    let field_b_content = fs::read_to_string(docs_dir.join("Foo/b.md")).unwrap();
    assert_eq!(
        field_b_content, "\n",
        "Touched file should only contain newline"
    );
}

#[test]
fn test_touch_does_not_overwrite_existing_docs() {
    let source = r#"//! Module docstring

/// Documented function
pub fn documented() {}

pub fn undocumented() {}
"#;

    let (temp_dir, file_path) = setup_test_file(source, "test.rs");
    let docs_dir = temp_dir.path().join("docs");
    let docs_root = docs_dir.to_str().unwrap();

    let parsed = parse_file(&file_path).unwrap();

    // Extract actual docs
    let all_extractions = extract_all_docs(&parsed, docs_root);

    // Should have: test.md (module), documented.md (function)
    assert_eq!(all_extractions.len(), 2);

    // Find expected paths
    let expected_paths = find_expected_doc_paths(&parsed, docs_root);

    // Should find: test.md, documented.md, undocumented.md
    assert_eq!(expected_paths.len(), 3);

    // Filter missing
    let existing_paths: std::collections::HashSet<_> =
        all_extractions.iter().map(|e| &e.markdown_path).collect();

    let missing_paths: Vec<_> = expected_paths
        .into_iter()
        .filter(|extraction| !existing_paths.contains(&extraction.markdown_path))
        .collect();

    // Should only find undocumented.md
    assert_eq!(missing_paths.len(), 1);
    assert!(missing_paths[0]
        .markdown_path
        .to_str()
        .unwrap()
        .ends_with("undocumented.md"));

    // The documented.md should still have its content
    let doc_extraction = all_extractions
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap().ends_with("documented.md"))
        .unwrap();
    assert_eq!(doc_extraction.content, "Documented function\n");

    // The undocumented.md should have empty content
    assert_eq!(missing_paths[0].content, "\n");
}

#[test]
fn test_touch_with_impl_trait_methods() {
    let source = r#"//! Module docstring

pub trait Format {
    fn file_extension(&self) -> &str;
}

pub struct MarkdownFormat;

impl Format for MarkdownFormat {
    /// Documented implementation
    fn file_extension(&self) -> &str {
        "md"
    }
}

impl MarkdownFormat {
    /// Documented method
    pub fn new() -> Self {
        Self
    }
    
    pub fn undocumented_method(&self) {
        // No docs
    }
}
"#;

    let (temp_dir, file_path) = setup_test_file(source, "test.rs");
    let docs_dir = temp_dir.path().join("docs");
    let docs_root = docs_dir.to_str().unwrap();

    let parsed = parse_file(&file_path).unwrap();

    // Extract actual docs
    let mut all_extractions = extract_all_docs(&parsed, docs_root);

    // Should have docs for: test.md, Format.md, MarkdownFormat.md,
    // MarkdownFormat/Format/file_extension.md, MarkdownFormat/new.md
    let _documented_count = all_extractions.len();

    // Find expected paths
    let expected_paths = find_expected_doc_paths(&parsed, docs_root);

    // Filter missing
    let existing_paths: std::collections::HashSet<_> =
        all_extractions.iter().map(|e| &e.markdown_path).collect();

    let missing_paths: Vec<_> = expected_paths
        .into_iter()
        .filter(|extraction| !existing_paths.contains(&extraction.markdown_path))
        .collect();

    // Should find undocumented_method
    assert!(missing_paths.iter().any(|e| e
        .markdown_path
        .to_str()
        .unwrap()
        .ends_with("MarkdownFormat/undocumented_method.md")));

    // All missing should have empty content
    for missing in &missing_paths {
        assert_eq!(missing.content, "\n");
    }

    // Combine and write
    all_extractions.extend(missing_paths);
    let report = write_extractions(&all_extractions, false).unwrap();

    assert_eq!(report.files_skipped, 0);
    assert!(report.errors.is_empty());

    // Verify documented files have content
    let new_method = fs::read_to_string(docs_dir.join("MarkdownFormat/new.md")).unwrap();
    assert_eq!(new_method, "Documented method\n");

    // Verify touched file is empty
    let undoc_method =
        fs::read_to_string(docs_dir.join("MarkdownFormat/undocumented_method.md")).unwrap();
    assert_eq!(undoc_method, "\n");
}

#[test]
fn test_touch_respects_existing_files_on_disk() {
    let source = r#"//! Module docstring

pub fn my_function() {}
"#;

    let (temp_dir, file_path) = setup_test_file(source, "test.rs");
    let docs_dir = temp_dir.path().join("docs");
    let docs_root = docs_dir.to_str().unwrap();

    // Create the docs directory and pre-populate my_function.md
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(docs_dir.join("my_function.md"), "Existing documentation\n").unwrap();

    let parsed = parse_file(&file_path).unwrap();

    // Extract actual docs (none, since no docstrings)
    let mut all_extractions = extract_all_docs(&parsed, docs_root);
    assert_eq!(all_extractions.len(), 1); // Just test.md module file

    // Find expected paths
    let expected_paths = find_expected_doc_paths(&parsed, docs_root);

    // Filter missing - should respect files that already exist on disk
    let existing_paths: std::collections::HashSet<_> =
        all_extractions.iter().map(|e| &e.markdown_path).collect();

    let missing_paths: Vec<_> = expected_paths
        .into_iter()
        .filter(|extraction| {
            !existing_paths.contains(&extraction.markdown_path)
                && !extraction.markdown_path.exists() // This should filter out my_function.md
        })
        .collect();

    // Should NOT include my_function.md since it already exists
    assert!(!missing_paths.iter().any(|e| e
        .markdown_path
        .to_str()
        .unwrap()
        .ends_with("my_function.md")));

    all_extractions.extend(missing_paths);
    write_extractions(&all_extractions, false).unwrap();

    // Verify existing file was not overwritten
    let content = fs::read_to_string(docs_dir.join("my_function.md")).unwrap();
    assert_eq!(content, "Existing documentation\n");
}
