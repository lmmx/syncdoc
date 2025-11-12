// syncdoc-migrate/src/write/tests.rs

use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_extract_nested_module_paths() {
    let source = r#"
        pub mod outer {
            pub mod inner {
                /// Documentation for nested function
                pub fn nested_func() {}
            }
        }
    "#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, source).unwrap();

    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, "docs");

    // Should find the nested function
    assert!(!extractions.is_empty());

    let func_extraction = extractions
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap().contains("nested_func"))
        .expect("Should find nested_func extraction");

    assert_eq!(
        func_extraction.markdown_path,
        PathBuf::from("docs/outer/inner/nested_func.md")
    );
}

#[test]
fn test_extract_impl_method_paths() {
    let source = r#"
        struct MyType;

        impl MyType {
            /// Method documentation
            pub fn my_method(&self) {}
        }
    "#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, source).unwrap();

    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, "docs");

    let method_extraction = extractions
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap().contains("my_method"))
        .expect("Should find my_method extraction");

    assert_eq!(
        method_extraction.markdown_path,
        PathBuf::from("docs/MyType/my_method.md")
    );
}

#[test]
fn test_extract_enum_variant_paths() {
    let source = r#"
        /// An enum
        pub enum MyEnum {
            /// First variant
            Variant1,
            /// Second variant
            Variant2,
        }
    "#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, source).unwrap();

    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, "docs");

    // Should find enum and both variants
    assert!(extractions.len() >= 3);

    let enum_extraction = extractions
        .iter()
        .find(|e| e.markdown_path == PathBuf::from("docs/MyEnum.md"))
        .expect("Should find enum extraction");
    assert_eq!(enum_extraction.content, "An enum");

    let variant1 = extractions
        .iter()
        .find(|e| e.markdown_path == PathBuf::from("docs/MyEnum/Variant1.md"))
        .expect("Should find Variant1");
    assert_eq!(variant1.content, "First variant");
}

#[test]
fn test_write_creates_directories() {
    let temp_dir = TempDir::new().unwrap();
    let docs_path = temp_dir.path().join("docs");

    let extractions = vec![
        DocExtraction {
            markdown_path: docs_path.join("module/submodule/func.md"),
            content: "Function docs".to_string(),
            source_location: "test.rs:10".to_string(),
        },
        DocExtraction {
            markdown_path: docs_path.join("other/item.md"),
            content: "Other docs".to_string(),
            source_location: "test.rs:20".to_string(),
        },
    ];

    let report = write_extractions(&extractions, false).unwrap();

    assert_eq!(report.files_written, 2);
    assert_eq!(report.files_skipped, 0);
    assert!(report.errors.is_empty());

    // Verify files exist
    assert!(docs_path.join("module/submodule/func.md").exists());
    assert!(docs_path.join("other/item.md").exists());

    // Verify content
    let content = fs::read_to_string(docs_path.join("module/submodule/func.md")).unwrap();
    assert_eq!(content, "Function docs");
}

#[test]
fn test_dry_run_no_files_created() {
    let temp_dir = TempDir::new().unwrap();
    let docs_path = temp_dir.path().join("docs");

    let extractions = vec![DocExtraction {
        markdown_path: docs_path.join("test.md"),
        content: "Test".to_string(),
        source_location: "test.rs:1".to_string(),
    }];

    let report = write_extractions(&extractions, true).unwrap();

    assert_eq!(report.files_written, 1);
    assert!(!docs_path.join("test.md").exists());
}

#[test]
fn test_build_path_with_context() {
    let path = build_path(
        "docs",
        &["module".to_string(), "submodule".to_string()],
        "func",
    );
    assert_eq!(path, "docs/module/submodule/func.md");
}

#[test]
fn test_build_path_no_context() {
    let path = build_path("docs", &[], "func");
    assert_eq!(path, "docs/func.md");
}
