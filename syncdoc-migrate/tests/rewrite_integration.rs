// syncdoc-migrate/tests/rewrite_integration.rs

use insta::assert_snapshot;
use proc_macro2::TokenStream;
use rust_format::{Formatter, RustFmt};
use std::fs;
use std::str::FromStr;
use syncdoc_migrate::{parse_file, rewrite::rewrite_file};
use tempfile::TempDir;

fn format_code(code: &str) -> String {
    if let Ok(tokens) = TokenStream::from_str(code) {
        RustFmt::default()
            .format_tokens(tokens)
            .unwrap_or_else(|_| code.to_string())
    } else {
        code.to_string()
    }
}

fn test_rewrite(source: &str, strip: bool, annotate: bool) -> String {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, source).unwrap();

    let parsed = parse_file(&file_path).unwrap();
    let result = rewrite_file(&parsed, "docs", strip, annotate);

    match result {
        Some(code) => format_code(&code),
        None => "NO_REWRITE_NEEDED".to_string(),
    }
}

#[test]
fn test_strip_function_docs() {
    let source = r#"
        /// This is a function
        /// with multiple doc lines
        pub fn hello() {
            println!("world");
        }
    "#;

    assert_snapshot!(test_rewrite(source, true, false));
}

#[test]
fn test_strip_preserves_other_attrs() {
    let source = r#"
        /// Documentation
        #[derive(Debug, Clone)]
        #[cfg(test)]
        pub struct MyStruct {
            field: i32
        }
    "#;

    assert_snapshot!(test_rewrite(source, true, false));
}

#[test]
fn test_strip_struct_with_field_docs() {
    let source = r#"
        /// Struct documentation
        pub struct MyStruct {
            /// Field documentation
            pub field: i32,
            /// Another field
            another: String,
        }
    "#;

    assert_snapshot!(test_rewrite(source, true, false));
}

#[test]
fn test_strip_enum_with_variant_docs() {
    let source = r#"
        /// Enum documentation
        pub enum MyEnum {
            /// Variant A
            VariantA,
            /// Variant B with data
            VariantB(i32),
        }
    "#;

    assert_snapshot!(test_rewrite(source, true, false));
}

#[test]
fn test_annotate_function() {
    let source = r#"
        pub fn hello() {
            println!("world");
        }
    "#;

    assert_snapshot!(test_rewrite(source, false, true));
}

#[test]
fn test_annotate_struct_with_fields() {
    let source = r#"
        pub struct MyStruct {
            pub field: i32,
            another: String,
        }
    "#;

    assert_snapshot!(test_rewrite(source, false, true));
}

#[test]
fn test_annotate_enum_with_variants() {
    let source = r#"
        pub enum MyEnum {
            VariantA,
            VariantB(i32),
        }
    "#;

    assert_snapshot!(test_rewrite(source, false, true));
}

#[test]
fn test_annotate_module() {
    let source = r#"
        pub mod submodule {
            pub fn inner_func() {}
        }
    "#;

    assert_snapshot!(test_rewrite(source, false, true));
}

#[test]
fn test_annotate_impl_block() {
    let source = r#"
        impl MyStruct {
            pub fn method(&self) -> i32 {
                42
            }
        }
    "#;

    assert_snapshot!(test_rewrite(source, false, true));
}

#[test]
fn test_annotate_trait() {
    let source = r#"
        pub trait MyTrait {
            fn required_method(&self);

            fn default_method(&self) {
                println!("default");
            }
        }
    "#;

    assert_snapshot!(test_rewrite(source, false, true));
}

#[test]
fn test_strip_and_annotate_function() {
    let source = r#"
        /// Old documentation
        pub fn hello() {
            println!("world");
        }
    "#;

    assert_snapshot!(test_rewrite(source, true, true));
}

#[test]
fn test_strip_and_annotate_struct() {
    let source = r#"
        /// Struct docs
        pub struct MyStruct {
            /// Field docs
            pub field: i32,
        }
    "#;

    assert_snapshot!(test_rewrite(source, true, true));
}

#[test]
fn test_unit_struct() {
    let source = r#"
        pub struct UnitStruct;
    "#;

    assert_snapshot!(test_rewrite(source, false, true));
}

#[test]
fn test_tuple_struct() {
    let source = r#"
        pub struct TupleStruct(pub i32, String);
    "#;

    assert_snapshot!(test_rewrite(source, false, true));
}

#[test]
fn test_empty_struct() {
    let source = r#"
        pub struct EmptyStruct {}
    "#;

    assert_snapshot!(test_rewrite(source, false, true));
}

#[test]
fn test_complex_mixed_file() {
    let source = r#"
        /// Module documentation
        pub mod calculations {
            /// Function in module
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        }

        /// Struct documentation
        #[derive(Debug)]
        pub struct Calculator {
            /// The current value
            value: i32,
        }

        /// Impl block
        impl Calculator {
            /// Creates a new calculator
            pub fn new() -> Self {
                Self { value: 0 }
            }
        }

        /// Standalone function
        pub fn standalone() {}
    "#;

    assert_snapshot!(test_rewrite(source, true, true));
}

#[test]
fn test_no_ops_returns_none() {
    let source = r#"
        pub fn hello() {}
    "#;

    assert_eq!(test_rewrite(source, false, false), "NO_REWRITE_NEEDED");
}
