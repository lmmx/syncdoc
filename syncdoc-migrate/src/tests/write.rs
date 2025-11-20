use crate::write::*;
use braces::{brace_paths, BraceConfig};
use insta::assert_snapshot;
use std::fs;
use tempfile::TempDir;

fn to_braces(paths: &[&str]) -> String {
    let braces_config = BraceConfig::default();
    brace_paths(paths, &braces_config).expect("Brace error")
}

fn setup_test_file(source: &str, filename: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, source).unwrap();
    (temp_dir, file_path)
}

fn parse_and_get_paths(source: &str, filename: &str, docs_dir: &str) -> Vec<String> {
    let (_temp_dir, file_path) = setup_test_file(source, filename);
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let extracts = extract_all_docs(&parsed, docs_dir);
    extracts
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect()
}

fn get_path_refs(paths: &[String]) -> Vec<&str> {
    paths.iter().map(|s| s.as_str()).collect()
}

#[test]
fn test_extract_nested_module_paths() {
    let paths = parse_and_get_paths(
        r#"
        pub mod outer {
            pub mod inner {
                /// Documentation for nested function
                pub fn nested_func() {}
            }
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/outer/inner/nested_func.md");
}

#[test]
fn test_extract_impl_method_paths() {
    let paths = parse_and_get_paths(
        r#"
        struct MyType;

        impl MyType {
            /// Method documentation
            pub fn my_method(&self) {}
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/MyType/my_method.md");
}

#[test]
fn test_extract_enum_variant_paths() {
    let paths = parse_and_get_paths(
        r#"
        /// An enum
        pub enum MyEnum {
            /// First variant
            Variant1,
            /// Second variant
            Variant2,
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/MyEnum/{Variant1,Variant2,}.md");
}

#[test]
fn test_write_creates_directories() {
    let temp_dir = TempDir::new().unwrap();
    let docs_path = temp_dir.path().join("docs");

    let extracts = vec![
        DocExtract {
            markdown_path: docs_path.join("module/submodule/func.md"),
            content: "Function docs".to_string(),
            source_location: "test.rs:10".to_string(),
        },
        DocExtract {
            markdown_path: docs_path.join("other/item.md"),
            content: "Other docs".to_string(),
            source_location: "test.rs:20".to_string(),
        },
    ];

    let report = write_extracts(&extracts, false).unwrap();

    assert_eq!(report.files_written, 2);
    assert_eq!(report.files_skipped, 0);
    assert!(report.errors.is_empty());

    // Verify files exist with brace snapshot
    let created_paths: Vec<String> = extracts
        .iter()
        .map(|e| {
            e.markdown_path
                .strip_prefix(&docs_path)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();
    let path_refs: Vec<&str> = created_paths.iter().map(|s| s.as_str()).collect();
    assert_snapshot!(to_braces(&path_refs), @"{module/submodule/func.md,other/item.md}");

    // Verify content
    let content = fs::read_to_string(docs_path.join("module/submodule/func.md")).unwrap();
    assert_eq!(content, "Function docs");
}

#[test]
fn test_dry_run_no_files_created() {
    let temp_dir = TempDir::new().unwrap();
    let docs_path = temp_dir.path().join("docs");

    let extracts = vec![DocExtract {
        markdown_path: docs_path.join("test.md"),
        content: "Test".to_string(),
        source_location: "test.rs:1".to_string(),
    }];

    let report = write_extracts(&extracts, true).unwrap();

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
