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
fn test_inject_syncdoc_after_visibility() {
    let input = quote! {
        pub fn test() {}
    };

    let output = inject_syncdoc_attr(input, "docs/test.md");
    let output_str = output.to_string();

    // Should have syncdoc attribute
    assert!(output_str.contains("syncdoc"));
    assert!(output_str.contains("path"));
    assert!(output_str.contains("docs/test.md"));

    // syncdoc should come after pub
    let pub_pos = output_str.find("pub").unwrap();
    let syncdoc_pos = output_str.find("syncdoc").unwrap();
    assert!(syncdoc_pos > pub_pos);
}

#[test]
fn test_inject_omnidoc_after_visibility() {
    let input = quote! {
        pub struct MyStruct {
            field: i32
        }
    };

    let output = inject_omnidoc_attr(input, "docs/MyStruct.md");
    let output_str = output.to_string();

    assert!(output_str.contains("omnidoc"));
    assert!(output_str.contains("docs/MyStruct.md"));
}

#[test]
fn test_inject_before_derive() {
    let input = quote! {
        #[derive(Debug)]
        pub struct MyStruct;
    };

    let output = inject_syncdoc_attr(input, "docs/MyStruct.md");
    let output_str = output.to_string();

    // syncdoc should come before derive
    let syncdoc_pos = output_str.find("syncdoc").unwrap();
    let derive_pos = output_str.find("derive").unwrap();
    assert!(syncdoc_pos < derive_pos);
}

#[test]
fn test_inject_no_visibility() {
    let input = quote! {
        fn private_func() {}
    };

    let output = inject_syncdoc_attr(input, "docs/private_func.md");
    let output_str = output.to_string();

    assert!(output_str.contains("syncdoc"));
    assert!(output_str.contains("docs/private_func.md"));
}

#[test]
fn test_needs_omnidoc_logic() {
    use proc_macro2::TokenStream;
    use std::str::FromStr;
    use syncdoc_core::parse::*;

    // Module should need omnidoc
    let module_code = "mod test { fn inner() {} }";
    let tokens = TokenStream::from_str(module_code).unwrap();
    let module: ModuleItem = tokens.into_token_iter().parse().unwrap();
    assert!(needs_omnidoc(&module));

    // Function should NOT need omnidoc
    let func_code = "fn test() {}";
    let tokens = TokenStream::from_str(func_code).unwrap();
    let func: ModuleItem = tokens.into_token_iter().parse().unwrap();
    assert!(!needs_omnidoc(&func));

    // Enum with variants should need omnidoc
    let enum_code = "enum Test { A, B }";
    let tokens = TokenStream::from_str(enum_code).unwrap();
    let enum_item: ModuleItem = tokens.into_token_iter().parse().unwrap();
    assert!(needs_omnidoc(&enum_item));

    // Struct with named fields should need omnidoc
    let struct_code = "struct Test { field: i32 }";
    let tokens = TokenStream::from_str(struct_code).unwrap();
    let struct_item: ModuleItem = tokens.into_token_iter().parse().unwrap();
    assert!(needs_omnidoc(&struct_item));

    // Tuple struct should NOT need omnidoc
    let tuple_struct_code = "struct Test(i32);";
    let tokens = TokenStream::from_str(tuple_struct_code).unwrap();
    let tuple_struct: ModuleItem = tokens.into_token_iter().parse().unwrap();
    assert!(!needs_omnidoc(&tuple_struct));
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
