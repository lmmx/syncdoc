use braces::{brace_paths, BraceConfig};
use insta::assert_snapshot;
use std::fs;
use syncdoc_migrate::write::write_extracts;

mod helpers;
use helpers::*;

fn to_braces(paths: &[&str]) -> String {
    let braces_config = BraceConfig::default();
    brace_paths(paths, &braces_config).expect("Brace error")
}

fn get_paths_as_strings(extracts: &[syncdoc_migrate::write::DocExtract]) -> Vec<String> {
    extracts
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect()
}

fn strip_prefix_from_paths(paths: &[String], prefix: &str) -> Vec<String> {
    paths
        .iter()
        .map(|p| {
            p.strip_prefix(prefix)
                .unwrap_or(p)
                .trim_start_matches('/')
                .to_string()
        })
        .collect()
}

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

    let (mut extracts, missing) = parse_and_extract(&file_path, docs_root);

    // Snapshot extracted paths (with content)
    let extracted_paths = get_paths_as_strings(&extracts);
    let relative = strip_prefix_from_paths(&extracted_paths, docs_root);
    let refs: Vec<&str> = relative.iter().map(|s| s.as_str()).collect();
    assert_snapshot!(to_braces(&refs), @"{lib,Foo/{a,}}.md");

    // Snapshot missing paths (without content)
    let missing_paths = get_paths_as_strings(&missing);
    let relative_missing = strip_prefix_from_paths(&missing_paths, docs_root);
    let missing_refs: Vec<&str> = relative_missing.iter().map(|s| s.as_str()).collect();
    assert_snapshot!(to_braces(&missing_refs), @"Foo/b.md");

    // Verify content
    let foo_doc = extracts
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap().ends_with("Foo.md"))
        .expect("Should find Foo.md");
    assert_eq!(foo_doc.content, "Docstring for Foo\n");

    let field_a_doc = extracts
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap().ends_with("Foo/a.md"))
        .expect("Should find Foo/a.md");
    assert_eq!(field_a_doc.content, "Docstring for field A\n");

    assert_eq!(
        missing[0].content, "\n",
        "Missing file should have empty content"
    );

    // Add missing paths and write everything
    extracts.extend(missing);

    let report = write_extracts(&extracts, false).unwrap();
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

    let (extracts, missing) = parse_and_extract(&file_path, &docs_root);

    // Snapshot what has docs
    let extracted_paths = get_paths_as_strings(&extracts);
    let relative = strip_prefix_from_paths(&extracted_paths, &docs_root);
    let refs: Vec<&str> = relative.iter().map(|s| s.as_str()).collect();
    assert_snapshot!(to_braces(&refs), @"{test,documented}.md");

    // Snapshot what's missing
    let missing_paths = get_paths_as_strings(&missing);
    let relative_missing = strip_prefix_from_paths(&missing_paths, &docs_root);
    let missing_refs: Vec<&str> = relative_missing.iter().map(|s| s.as_str()).collect();
    assert_snapshot!(to_braces(&missing_refs), @"undocumented.md");

    // Verify content
    let doc_extract = extracts
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap().ends_with("documented.md"))
        .unwrap();
    assert_eq!(doc_extract.content, "Documented function\n");

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

    let (mut extracts, missing) = parse_and_extract(&file_path, docs_root);

    // Snapshot missing paths
    let missing_paths = get_paths_as_strings(&missing);
    let relative_missing = strip_prefix_from_paths(&missing_paths, docs_root);
    let missing_refs: Vec<&str> = relative_missing.iter().map(|s| s.as_str()).collect();
    assert_snapshot!(to_braces(&missing_refs), @"{Format/{file_extension,},MarkdownFormat/{undocumented_method,}}.md");

    // All missing should have empty content
    for m in &missing {
        assert_eq!(m.content, "\n");
    }

    // Combine and write
    extracts.extend(missing);
    let report = write_extracts(&extracts, false).unwrap();
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

    let (mut extracts, missing) = parse_and_extract(&file_path, docs_root);

    // Should have only test.md (module)
    let extracted_paths = get_paths_as_strings(&extracts);
    let relative = strip_prefix_from_paths(&extracted_paths, docs_root);
    let refs: Vec<&str> = relative.iter().map(|s| s.as_str()).collect();
    assert_snapshot!(to_braces(&refs), @"test.md");

    // Should have no missing since my_function.md exists on disk
    assert!(missing.is_empty());

    extracts.extend(missing);
    write_extracts(&extracts, false).unwrap();

    // Verify existing file was not overwritten
    assert_file(docs_dir.join("my_function.md"), "Existing documentation\n");
}
