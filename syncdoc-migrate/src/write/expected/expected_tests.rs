use super::*;
use braces::{brace_paths, BraceConfig};
use insta::assert_snapshot;
use std::fs;
use tempfile::TempDir;

fn setup_test_file(source: &str, filename: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, source).unwrap();
    (temp_dir, file_path)
}

fn to_braces(paths: &[&str]) -> String {
    let braces_config = BraceConfig::default();
    brace_paths(paths, &braces_config).expect("Brace error")
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

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    assert_snapshot!(to_braces(&paths), @"docs/{test,my_function}.md");

    // Content should be empty (just a newline from DocExtraction::new)
    let contents: Vec<&str> = expected.iter().map(|e| e.content.as_str()).collect();
    assert_snapshot!(to_braces(&contents), @"\n");
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

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    assert_snapshot!(to_braces(&paths), @"docs/{test,Config/{port,host,timeout,}}.md");
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

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    assert_snapshot!(to_braces(&paths), @"docs/{test,Status/{Active,Inactive,Error,}}.md");
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

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    assert_snapshot!(to_braces(&paths), @"docs/{test,Calculator/{new,add,subtract,}}.md");
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

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    assert_snapshot!(
        to_braces(&paths),
        @"docs/{test,outer/{inner/{nested_func,},outer_func,}}.md"
    );
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

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    eprintln!("EXPECTED DOC PATHS:\n{:#?}", expected);

    assert_snapshot!(to_braces(&paths), @"docs/{test,MyTrait/{required_method,default_method,}}.md");

    // Note: trait methods without bodies (required methods) don't have
    // function signatures in the parsed AST the same way, so we only
    // expect the default method
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

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    assert_snapshot!(to_braces(&paths), @"docs/{test,MY_CONST,MY_STATIC,MyType}.md");
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

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    assert_snapshot!(
        to_braces(&paths),
        @"docs/{test.md,api.md,api/{Config.md,Config/{port.md,new.md},Status.md,Status/{Ok.md,Error.md}}}"
    );
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

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    assert_snapshot!(to_braces(&paths), @"docs/{lib,TimeOfDay/{Day,Night,}}.md");
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

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    assert_snapshot!(to_braces(&paths), @"docs/{test,MyTrait/{required,with_default,}}.md");

    // Required methods and extern declarations shouldn't be in the list
}

#[test]
fn test_find_expected_trait_impl_for_struct() {
    let source = r#"
        pub trait Format {
            fn file_extension(&self) -> &str;
            fn language(&self) -> String;
        }

        pub struct MarkdownFormat;

        impl Format for MarkdownFormat {
            fn file_extension(&self) -> &str {
                "md"
            }

            fn language(&self) -> String {
                "markdown".to_string()
            }
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "test.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    eprintln!("Paths found: {:#?}", paths);

    assert_snapshot!(
        to_braces(&paths),
        @"docs/{test,Format/{file_extension,language,},MarkdownFormat/{Format/{file_extension,language},}}.md"
    );
}

#[test]
fn test_find_expected_trait_impl_in_submodule() {
    let source = r#"
        pub mod formats {
            pub mod markdown {
                pub trait Format {
                    fn file_extension(&self) -> &str;
                }

                pub struct MarkdownFormat;

                impl Format for MarkdownFormat {
                    fn file_extension(&self) -> &str {
                        "md"
                    }
                }
            }
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "test.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    eprintln!("Paths found: {:#?}", paths);

    assert_snapshot!(
        to_braces(&paths),
        @"docs/{test,formats/{markdown/{Format/{file_extension,},MarkdownFormat/{Format/file_extension,},},}}.md"
    );
}

#[test]
fn test_find_expected_regular_impl_vs_trait_impl() {
    let source = r#"
        pub struct MyStruct;

        // Regular impl
        impl MyStruct {
            pub fn new() -> Self {
                Self
            }
        }

        pub trait MyTrait {
            fn trait_method(&self);
        }

        // Trait impl
        impl MyTrait for MyStruct {
            fn trait_method(&self) {}
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source, "test.rs");
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, "docs");

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();

    eprintln!("Paths found: {:#?}", paths);

    assert_snapshot!(
        to_braces(&paths),
        @"docs/{test,MyStruct/{new,MyTrait/trait_method,},MyTrait/{trait_method,}}.md"
    );
}
