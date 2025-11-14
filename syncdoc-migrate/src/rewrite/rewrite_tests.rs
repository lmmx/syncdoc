// syncdoc-migrate/src/rewrite/tests.rs

use super::*;
use syncdoc_core::parse::ModuleContent;

#[test]
fn test_rewrite_roundtrip() {
    use std::fs;
    use std::str::FromStr;
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
    let stripped = rewrite_file(&parsed, "docs", true, false);
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
    let result = rewrite_file(&parsed, "docs", false, false);

    assert!(result.is_none());
}
