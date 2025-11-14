// syncdoc-migrate/src/rewrite/tests.rs

use super::*;
use quote::quote;

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
fn test_strip_inner_doc_attributes() {
    let input = quote! {
        //! Module level documentation

        pub enum MyEnum {
            Variant1,
        }
    };

    let output = strip_doc_attrs(input);
    let output_str = output.to_string();

    eprintln!("{}", output_str);

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
    let output_str = output.to_string();

    // Should strip both types
    assert!(!output_str.contains("Inner"));
    assert!(!output_str.contains("Outer"));
}
