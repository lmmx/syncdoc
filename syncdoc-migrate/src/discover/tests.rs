use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_discover_finds_rs_files() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // Create some .rs files
    fs::write(base.join("main.rs"), "fn main() {}").unwrap();
    fs::write(base.join("lib.rs"), "pub fn foo() {}").unwrap();

    // Create a subdirectory with more .rs files
    let sub_dir = base.join("module");
    fs::create_dir(&sub_dir).unwrap();
    fs::write(sub_dir.join("mod.rs"), "pub mod inner;").unwrap();

    // Create non-.rs files (should be ignored)
    fs::write(base.join("README.txt"), "readme").unwrap();
    fs::write(base.join("data.json"), "{}").unwrap();

    let rust_files = discover_rust_files(base).unwrap();

    // Should find exactly 3 .rs files
    assert_eq!(rust_files.len(), 3);

    // Verify all are .rs files
    for file in &rust_files {
        assert_eq!(file.extension(), Some(std::ffi::OsStr::new("rs")));
    }
}

#[test]
fn test_parse_valid_module() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");

    let source = r#"
        fn foo() {
            println!("hello");
        }

        pub fn bar(x: i32) -> i32 {
            x + 1
        }
    "#;

    fs::write(&file_path, source).unwrap();

    let result = parse_file(&file_path);
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert_eq!(parsed.path, file_path);
    assert_eq!(parsed.original_source, source);
    assert!(!parsed.content.items.0.is_empty());
}

#[test]
fn test_parse_invalid_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.rs");

    // Write malformed Rust code
    let source = r#"
        fn unclosed_function( {
            this is not valid rust
    "#;

    fs::write(&file_path, source).unwrap();

    let result = parse_file(&file_path);
    assert!(result.is_err());

    match result {
        Err(ParseError::ParseFailed(_)) => {
            // Expected error type
        }
        _ => panic!("Expected ParseError::ParseFailed"),
    }
}
