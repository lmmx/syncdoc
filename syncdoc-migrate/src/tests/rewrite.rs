use crate::rewrite::*;
use syncdoc_core::parse::ModuleContent;

use crate::config::DocsPathMode;

use std::str::FromStr;

#[test]
fn test_rewrite_roundtrip() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");

    let source = r#"
        /// Documentation
        pub fn test() {
            println!("hello");
        }
    "#;

    fs::write(&file_path, source).unwrap();

    // Parse the file
    let parsed = crate::discover::parse_file(&file_path).unwrap();

    // Strip docs
    let stripped = rewrite_file(&parsed, "docs", DocsPathMode::TomlConfig, true, false);
    assert!(stripped.is_some());
    let stripped_code = stripped.unwrap();

    // Verify docs are removed
    assert!(!stripped_code.contains("Documentation"));

    // Parse stripped code to verify it's valid
    let tokens = TokenStream::from_str(&stripped_code).unwrap();
    let result: Result<ModuleContent> = tokens.into_token_iter().parse();
    assert!(result.is_ok());
}

#[test]
fn test_rewrite_none_when_no_ops() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, "fn test() {}").unwrap();

    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let result = rewrite_file(&parsed, "docs", DocsPathMode::TomlConfig, false, false);

    assert!(result.is_none());
}

#[test]
fn test_migration_preserves_blank_line_after_module_docstring() {
    use crate::discover::ParsedFile;
    use crate::rewrite::rewrite_file;
    use std::path::PathBuf;

    let original_source = r#"//! Module docstring

use std::io::Path;

#[derive(PartialEq)]
/// Options docstring
pub enum Options {
    A,
}

pub struct Foo;
"#;

    // Parse it
    let content = proc_macro2::TokenStream::from_str(original_source).unwrap();
    let parsed_content = content
        .into_token_iter()
        .parse::<syncdoc_core::parse::ModuleContent>()
        .unwrap();

    let parsed = ParsedFile {
        path: PathBuf::from("test.rs"),
        content: parsed_content,
        original_source: original_source.to_string(),
    };

    // Run migration (strip + annotate)
    let result = rewrite_file(&parsed, "docs", DocsPathMode::TomlConfig, true, true).unwrap();

    // Count blank lines between items in original
    let _original_lines: Vec<&str> = original_source.lines().collect();
    let result_lines: Vec<&str> = result.lines().collect();

    eprintln!("{}", result);

    assert!(
        result_lines[0].contains("module_doc"),
        "Should have module docstring as first line"
    );
    assert_eq!(
        result_lines[1].trim(),
        "",
        "Should have blank line after module docstring"
    );
}

#[test]
fn test_migration_preserves_blank_lines() {
    use crate::discover::ParsedFile;
    use crate::rewrite::rewrite_file;
    use std::path::PathBuf;

    let original_source = r#"//! Module docstring

use std::io::Path;

#[derive(PartialEq)]
/// Options docstring
pub enum Options {
    A,
}

pub struct Foo;
"#;

    // Parse it
    let content = proc_macro2::TokenStream::from_str(original_source).unwrap();
    let parsed_content = content
        .into_token_iter()
        .parse::<syncdoc_core::parse::ModuleContent>()
        .unwrap();

    let parsed = ParsedFile {
        path: PathBuf::from("test.rs"),
        content: parsed_content,
        original_source: original_source.to_string(),
    };

    // Run migration (strip + annotate)
    let result = rewrite_file(&parsed, "docs", DocsPathMode::TomlConfig, true, true).unwrap();

    // Count blank lines between items in original
    let _original_lines: Vec<&str> = original_source.lines().collect();
    let result_lines: Vec<&str> = result.lines().collect();

    eprintln!("{}", result);

    // Find the line with "A," and check there's a blank line after
    let multi_idx = result_lines.iter().position(|l| l.contains("A")).unwrap();
    assert_eq!(
        result_lines[multi_idx + 2].trim(),
        "",
        "Should have blank line after Options enum"
    );
}
