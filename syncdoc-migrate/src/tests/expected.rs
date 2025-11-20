use crate::write::expected::*;
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

fn parse_and_get_paths(source: &str, filename: &str, docs_dir: &str) -> Vec<String> {
    let (_temp_dir, file_path) = setup_test_file(source, filename);
    let parsed = crate::discover::parse_file(&file_path).unwrap();
    let expected = find_expected_doc_paths(&parsed, docs_dir);
    expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect()
}

fn get_path_refs(paths: &[String]) -> Vec<&str> {
    paths.iter().map(|s| s.as_str()).collect()
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

    let paths: Vec<&str> = expected
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap())
        .collect();
    assert_snapshot!(to_braces(&paths), @"docs/{test,my_function}.md");

    // Content should be empty (just a newline from DocExtract::new)
    let contents: Vec<&str> = expected.iter().map(|e| e.content.as_str()).collect();
    assert_snapshot!(to_braces(&contents), @"\n");
}

#[test]
fn test_find_expected_struct_with_fields() {
    let paths = parse_and_get_paths(
        r#"
        pub struct Config {
            pub port: u16,
            host: String,
            timeout: u64,
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/{test,Config/{port,host,timeout,}}.md");
}

#[test]
fn test_find_expected_enum_with_variants() {
    let paths = parse_and_get_paths(
        r#"
        pub enum Status {
            Active,
            Inactive,
            Error(String),
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/{test,Status/{Active,Inactive,Error,}}.md");
}

#[test]
fn test_find_expected_impl_block() {
    let paths = parse_and_get_paths(
        r#"
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
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/{test,Calculator/{new,add,subtract,}}.md");
}

#[test]
fn test_find_expected_nested_module() {
    let paths = parse_and_get_paths(
        r#"
        pub mod outer {
            pub mod inner {
                pub fn nested_func() {}
            }

            pub fn outer_func() {}
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/{test,outer/{inner/{nested_func,},outer_func,}}.md");
}

#[test]
fn test_find_expected_trait_with_methods() {
    let paths = parse_and_get_paths(
        r#"
        pub trait MyTrait {
            fn required_method(&self);

            fn default_method(&self) {
                println!("default");
            }
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/{test,MyTrait/{required_method,default_method,}}.md");
}

#[test]
fn test_find_expected_const_static_type_alias() {
    let paths = parse_and_get_paths(
        r#"
        const MY_CONST: i32 = 42;
        static MY_STATIC: &str = "hello";
        type MyType = Vec<String>;
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/{test,MY_CONST,MY_STATIC,MyType}.md");
}

#[test]
fn test_find_expected_complex_structure() {
    let paths = parse_and_get_paths(
        r#"
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
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(
        to_braces(&get_path_refs(&paths)),
        @"docs/{test.md,api.md,api/{Config.md,Config/{port.md,new.md},Status.md,Status/{Ok.md,Error.md}}}"
    );
}

#[test]
fn test_find_expected_lib_rs() {
    let paths = parse_and_get_paths(
        r#"
        pub enum TimeOfDay {
            Day,
            Night,
        }
        "#,
        "lib.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/{lib,TimeOfDay/{Day,Night,}}.md");
}

#[test]
fn test_find_expected_only_functions_with_bodies() {
    let paths = parse_and_get_paths(
        r#"
        trait MyTrait {
            fn required();
            fn with_default() {}
        }

        extern "C" {
            fn external_fn();
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/{test,MyTrait/{required,with_default,}}.md");
}

#[test]
fn test_find_expected_trait_impl_for_struct() {
    let paths = parse_and_get_paths(
        r#"
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
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(
        to_braces(&get_path_refs(&paths)),
        @"docs/{test,Format/{file_extension,language,},MarkdownFormat/{Format/{file_extension,language},}}.md"
    );
}

#[test]
fn test_find_expected_trait_impl_in_submodule() {
    let paths = parse_and_get_paths(
        r#"
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
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(
        to_braces(&get_path_refs(&paths)),
        @"docs/{test,formats/{markdown/{Format/{file_extension,},MarkdownFormat/{Format/file_extension,},},}}.md"
    );
}

#[test]
fn test_find_expected_regular_impl_vs_trait_impl() {
    let paths = parse_and_get_paths(
        r#"
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
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(
        to_braces(&get_path_refs(&paths)),
        @"docs/{test,MyStruct/{new,MyTrait/trait_method,},MyTrait/{trait_method,}}.md"
    );
}
