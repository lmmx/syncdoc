// syncdoc-migrate/src/rewrite/tests.rs

use super::*;
use quote::quote;
use syncdoc_core::parse::ModuleContent;
use unsynn::*;

#[test]
fn test_strip_preserves_non_doc() {
    let input = quote! {
        #[derive(Debug)]
        #[cfg(test)]
        pub struct MyStruct {
            field: i32
        }
    };

    let output = strip_doc_attrs(input);
    let output_str = output.to_string();

    assert!(output_str.contains("derive"));
    assert!(output_str.contains("Debug"));
    assert!(output_str.contains("cfg"));
    assert!(output_str.contains("test"));
}

#[test]
fn test_strip_removes_all_doc() {
    let input = quote! {
        #[doc = "First doc"]
        #[doc = "Second doc"]
        #[derive(Debug)]
        #[doc = "Third doc"]
        pub fn test() {}
    };

    let output = strip_doc_attrs(input);
    let output_str = output.to_string();

    assert!(!output_str.contains("doc"));
    assert!(!output_str.contains("First"));
    assert!(!output_str.contains("Second"));
    assert!(!output_str.contains("Third"));
    assert!(output_str.contains("derive"));
    assert!(output_str.contains("Debug"));
}

#[test]
fn test_strip_removes_cfg_attr_doc() {
    let input = quote! {
        #[cfg_attr(doc, doc = include_str!("../docs/test.md"))]
        pub fn test() {}
    };

    let output = strip_doc_attrs(input);
    let output_str = output.to_string();

    assert!(!output_str.contains("cfg_attr"));
    assert!(!output_str.contains("doc"));
}

#[test]
fn test_inject_omnidoc_after_visibility() {
    let input = quote! {
        pub fn test() {}
    };

    let output = inject_omnidoc_attr(input, "docs");
    let output_str = output.to_string();

    // Should have omnidoc attribute with docs root
    assert!(output_str.contains("omnidoc"));
    assert!(output_str.contains("path"));
    assert!(output_str.contains("\"docs\""));

    // omnidoc should come after pub
    let pub_pos = output_str.find("pub").unwrap();
    let omnidoc_pos = output_str.find("omnidoc").unwrap();
    assert!(omnidoc_pos > pub_pos);
}

#[test]
fn test_inject_omnidoc_before_derive() {
    let input = quote! {
        #[derive(Debug)]
        pub struct MyStruct;
    };

    let output = inject_omnidoc_attr(input, "docs");
    let output_str = output.to_string();

    // omnidoc should come before derive
    let omnidoc_pos = output_str.find("omnidoc").unwrap();
    let derive_pos = output_str.find("derive").unwrap();
    assert!(omnidoc_pos < derive_pos);
}

#[test]
fn test_inject_omnidoc_no_visibility() {
    let input = quote! {
        fn private_func() {}
    };

    let output = inject_omnidoc_attr(input, "docs");
    let output_str = output.to_string();

    assert!(output_str.contains("omnidoc"));
    assert!(output_str.contains("\"docs\""));
}

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

#[test]
fn test_strip_inner_doc_attributes() {
    let input = quote! {
        //! Module level documentation

        pub enum MyEnum {
            Variant1,
        }
    };

    let output = strip_doc_attrs(input);
    let output_str = output.to_string();

    // Should strip inner doc comments
    assert!(!output_str.contains("!"));
    assert!(!output_str.contains("Module level"));
}

#[test]
fn test_strip_mixed_inner_outer_docs() {
    let input = quote! {
        //! Inner doc
        /// Outer doc
        pub fn test() {}
    };

    let output = strip_doc_attrs(input);
    // Should strip both types
    assert!(!output.to_string().contains("Inner"));
    assert!(!output.to_string().contains("Outer"));
}
