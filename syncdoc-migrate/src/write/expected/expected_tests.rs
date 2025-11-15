use super::*;
use std::fs;
use tempfile::TempDir;

fn setup_test_file(source: &str, filename: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, source).unwrap();
    (temp_dir, file_path)
}

#[test]
fn test_find_expected_simple_function() {
    let source = r#"
        fn my_function() {
            println!("hello");
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "test.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    eprintln!("EXPECTED DOC PATHS:\n{:?}", expected);

    // Should find module file + function
    assert_eq!(expected.len(), 2);

    let paths: Vec<String> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect();

    assert!(paths.contains(&"docs/test.md".to_string()));
    assert!(paths.contains(&"docs/my_function.md".to_string()));

    // Content should be empty (just a newline from DocExtraction::new)
    assert!(expected.iter().all(|e| e.content == "\n"));
}

#[test]
fn test_find_expected_struct_with_fields() {
    let source = r#"
        pub struct Config {
            pub port: u16,
            host: String,
            timeout: u64,
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "test.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    eprintln!("EXPECTED DOC PATHS:\n{:?}", expected);

    // Module + struct + 3 fields
    assert_eq!(expected.len(), 5);

    let paths: Vec<String> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect();

    assert!(paths.contains(&"docs/test.md".to_string()));
    assert!(paths.contains(&"docs/Config.md".to_string()));
    assert!(paths.contains(&"docs/Config/port.md".to_string()));
    assert!(paths.contains(&"docs/Config/host.md".to_string()));
    assert!(paths.contains(&"docs/Config/timeout.md".to_string()));
}

#[test]
fn test_find_expected_enum_with_variants() {
    let source = r#"
        pub enum Status {
            Active,
            Inactive,
            Error(String),
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "test.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    eprintln!("EXPECTED DOC PATHS:\n{:?}", expected);

    // Module + enum + 3 variants
    assert_eq!(expected.len(), 5);

    let paths: Vec<String> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect();

    assert!(paths.contains(&"docs/Status.md".to_string()));
    assert!(paths.contains(&"docs/Status/Active.md".to_string()));
    assert!(paths.contains(&"docs/Status/Inactive.md".to_string()));
    assert!(paths.contains(&"docs/Status/Error.md".to_string()));
}

#[test]
fn test_find_expected_impl_block() {
    let source = r#"
        struct Calculator;

        impl Calculator {
            pub fn new() -> Self {
                Self
            }

            pub fn add(&self, a: i32, b: i32) -> i32 {
                a + b
            }

            fn subtract(&self, a: i32, b: i32) -> i32 {
                a - b
            }
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "test.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    eprintln!("EXPECTED DOC PATHS:\n{:?}", expected);

    // Module + struct + 3 methods
    assert_eq!(expected.len(), 5);

    let paths: Vec<String> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect();

    assert!(paths.contains(&"docs/test.md".to_string()));
    assert!(paths.contains(&"docs/Calculator.md".to_string()));
    assert!(paths.contains(&"docs/Calculator/new.md".to_string()));
    assert!(paths.contains(&"docs/Calculator/add.md".to_string()));
    assert!(paths.contains(&"docs/Calculator/subtract.md".to_string()));
}

#[test]
fn test_find_expected_nested_module() {
    let source = r#"
        pub mod outer {
            pub mod inner {
                pub fn nested_func() {}
            }

            pub fn outer_func() {}
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "test.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    eprintln!("EXPECTED DOC PATHS:\n{:?}", expected);

    let paths: Vec<String> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect();

    // Should have proper nested paths
    assert!(paths.contains(&"docs/test.md".to_string()));
    assert!(paths.contains(&"docs/outer.md".to_string()));
    assert!(paths.contains(&"docs/outer/inner.md".to_string()));
    assert!(paths.contains(&"docs/outer/inner/nested_func.md".to_string()));
    assert!(paths.contains(&"docs/outer/outer_func.md".to_string()));
}

#[test]
fn test_find_expected_trait_with_methods() {
    let source = r#"
        pub trait MyTrait {
            fn required_method(&self);

            fn default_method(&self) {
                println!("default");
            }
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "test.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    eprintln!("EXPECTED DOC PATHS:\n{:?}", expected);

    let paths: Vec<String> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect();

    eprintln!("EXPECTED DOC PATHS:\n{:#?}", expected);

    assert!(paths.contains(&"docs/test.md".to_string()));
    assert!(paths.contains(&"docs/MyTrait.md".to_string()));

    // Note: trait methods without bodies (required methods) don't have
    // function signatures in the parsed AST the same way, so we only
    // expect the default method
    assert!(paths.contains(&"docs/MyTrait/default_method.md".to_string()));

    // This is actually correct behavior - required methods are just
    // declarations, not function items, so they wouldn't get omnidoc
    // attributes in practice
}

#[test]
fn test_find_expected_const_static_type_alias() {
    let source = r#"
        const MY_CONST: i32 = 42;
        static MY_STATIC: &str = "hello";
        type MyType = Vec<String>;
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "test.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    eprintln!("EXPECTED DOC PATHS:\n{:?}", expected);

    let paths: Vec<String> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect();

    assert!(paths.contains(&"docs/MY_CONST.md".to_string()));
    assert!(paths.contains(&"docs/MY_STATIC.md".to_string()));
    assert!(paths.contains(&"docs/MyType.md".to_string()));
}

#[test]
fn test_find_expected_complex_structure() {
    let source = r#"
        pub mod api {
            pub struct Config {
                pub port: u16,
            }

            impl Config {
                pub fn new(port: u16) -> Self {
                    Self { port }
                }
            }

            pub enum Status {
                Ok,
                Error,
            }
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "test.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    eprintln!("EXPECTED DOC PATHS:\n{:?}", expected);

    let paths: Vec<String> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect();

    // Check all expected paths exist
    assert!(paths.contains(&"docs/test.md".to_string()));
    assert!(paths.contains(&"docs/api.md".to_string()));
    assert!(paths.contains(&"docs/api/Config.md".to_string()));
    assert!(paths.contains(&"docs/api/Config/port.md".to_string()));
    assert!(paths.contains(&"docs/api/Config/new.md".to_string()));
    assert!(paths.contains(&"docs/api/Status.md".to_string()));
    assert!(paths.contains(&"docs/api/Status/Ok.md".to_string()));
    assert!(paths.contains(&"docs/api/Status/Error.md".to_string()));
}

#[test]
fn test_find_expected_lib_rs() {
    let source = r#"
        pub enum TimeOfDay {
            Day,
            Night,
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "lib.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    eprintln!("EXPECTED DOC PATHS:\n{:?}", expected);

    let paths: Vec<String> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect();

    // lib.rs should map to docs/lib.md
    assert!(paths.contains(&"docs/lib.md".to_string()));
    assert!(paths.contains(&"docs/TimeOfDay.md".to_string()));
    assert!(paths.contains(&"docs/TimeOfDay/Day.md".to_string()));
    assert!(paths.contains(&"docs/TimeOfDay/Night.md".to_string()));
}

#[test]
fn test_find_expected_only_functions_with_bodies() {
    let source = r#"
        trait MyTrait {
            fn required();
            fn with_default() {}
        }

        extern "C" {
            fn external_fn();
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "test.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    let paths: Vec<String> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect();

    // Should only find items that would actually get #[omnidoc]
    assert!(paths.contains(&"docs/test.md".to_string()));
    assert!(paths.contains(&"docs/MyTrait.md".to_string()));
    assert!(paths.contains(&"docs/MyTrait/with_default.md".to_string()));

    // Required methods and extern declarations shouldn't be in the list
    assert!(!paths.contains(&"docs/MyTrait/required.md".to_string()));
    assert!(!paths.contains(&"docs/external_fn.md".to_string()));
}
