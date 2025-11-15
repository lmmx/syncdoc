// syncdoc-migrate/tests/touch_integration.rs

use std::fs;
use syncdoc_migrate::write::write_extractions;

mod helpers;

use helpers::*;

#[test]
fn test_extract_docs_and_touch_missing_files() {
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

    let (mut extractions, missing) = parse_and_extract(&file_path, docs_root);

    // Should have: lib.md (module), Foo.md (struct), Foo/a.md (field a)
    assert_eq!(extractions.len(), 3, "Should extract 3 docs with content");

    // Verify the docs have actual content
    let foo_doc = extractions
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap().ends_with("Foo.md"))
        .expect("Should find Foo.md");
    assert_eq!(foo_doc.content, "Docstring for Foo\n");

    let field_a_doc = extractions
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap().ends_with("Foo/a.md"))
        .expect("Should find Foo/a.md");
    assert_eq!(field_a_doc.content, "Docstring for field A\n");

    // Should only find Foo/b.md as missing
    assert_eq!(missing.len(), 1, "Should find 1 missing path");
    assert_missing_path(&missing, "Foo/b.md");
    assert_eq!(
        missing[0].content, "\n",
        "Missing file should have empty content"
    );

    // Add missing paths and write everything
    extractions.extend(missing);
    let report = write_extractions(&extractions, false).unwrap();

    assert_report(&report, 4);

    // Verify files exist and have correct content
    assert_file(docs_dir.join("Foo.md"), "Docstring for Foo\n");
    assert_file(docs_dir.join("Foo/a.md"), "Docstring for field A\n");
    assert_file(docs_dir.join("Foo/b.md"), "\n");
    assert!(docs_dir.join("lib.md").exists());
}

#[test]
fn test_touch_does_not_overwrite_existing_docs() {
    let source = r#"//! Module docstring

/// Documented function
pub fn documented() {}

pub fn undocumented() {}
"#;

    let (temp_dir, file_path) = setup_test_file(source, "test.rs");
    let docs_root = temp_dir.path().join("docs").to_str().unwrap().to_string();

    let (extractions, missing) = parse_and_extract(&file_path, &docs_root);

    // Should have: test.md (module), documented.md (function)
    assert_eq!(extractions.len(), 2);

    // Should only find undocumented.md
    assert_eq!(missing.len(), 1);
    assert_missing_path(&missing, "undocumented.md");

    // The documented.md should still have its content
    let doc_extraction = extractions
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap().ends_with("documented.md"))
        .unwrap();
    assert_eq!(doc_extraction.content, "Documented function\n");

    // The undocumented.md should have empty content
    assert_eq!(missing[0].content, "\n");
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

    let (mut extractions, missing) = parse_and_extract(&file_path, docs_root);

    // Should find undocumented_method
    assert_missing_path(&missing, "MarkdownFormat/undocumented_method.md");

    // All missing should have empty content
    for m in &missing {
        assert_eq!(m.content, "\n");
    }

    // Combine and write
    extractions.extend(missing);
    let report = write_extractions(&extractions, false).unwrap();

    assert_eq!(report.files_skipped, 0);
    assert!(report.errors.is_empty());

    // Verify documented and touched files
    assert_file(
        docs_dir.join("MarkdownFormat/new.md"),
        "Documented method\n",
    );
    assert_file(docs_dir.join("MarkdownFormat/undocumented_method.md"), "\n");
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

    let (mut extractions, missing) = parse_and_extract(&file_path, docs_root);

    assert_eq!(extractions.len(), 1); // Just test.md module file

    // Should NOT include my_function.md since it already exists
    assert!(!missing.iter().any(|e| e
        .markdown_path
        .to_str()
        .unwrap()
        .ends_with("my_function.md")));

    extractions.extend(missing);
    write_extractions(&extractions, false).unwrap();

    // Verify existing file was not overwritten
    assert_file(docs_dir.join("my_function.md"), "Existing documentation\n");
}
