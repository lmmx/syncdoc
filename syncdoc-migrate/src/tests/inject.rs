use crate::config::DocsPathMode;
use crate::rewrite::inject::*;
use crate::rewrite_file;
use quote::quote;
use std::str::FromStr;
use unsynn::*;

#[test]
fn test_inject_omnidoc_before_visibility() {
    let input = quote! {
        pub fn test() {}
    };

    let output = inject_omnidoc_attr(input, "docs", DocsPathMode::InlinePaths);
    let output_str = output.to_string();

    eprintln!("{}", output_str);

    // Should have omnidoc attribute with docs root
    assert!(output_str.contains("omnidoc"));
    assert!(output_str.contains("path"));
    assert!(output_str.contains("\"docs\""));

    // omnidoc should come after pub
    let pub_pos = output_str.find("pub").unwrap();
    let omnidoc_pos = output_str.find("omnidoc").unwrap();
    assert!(omnidoc_pos < pub_pos);
}

#[test]
fn test_inject_omnidoc_before_derive() {
    let input = quote! {
        #[derive(Debug)]
        pub struct MyStruct;
    };

    let output = inject_omnidoc_attr(input, "docs", DocsPathMode::InlinePaths);
    let output_str = output.to_string();

    eprintln!("{}", output_str);

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

    let output = inject_omnidoc_attr(input, "docs", DocsPathMode::InlinePaths);
    let output_str = output.to_string();

    assert!(output_str.contains("omnidoc"));
    assert!(output_str.contains("\"docs\""));
}

#[test]
fn test_inject_omnidoc_toml_config_mode() {
    let input = quote! {
        pub fn test() {}
    };

    let output = inject_omnidoc_attr(input, "docs", DocsPathMode::TomlConfig);
    let output_str = output.to_string();

    eprintln!("{}", output_str);

    // Should have omnidoc attribute WITHOUT path parameter
    assert!(output_str.contains("omnidoc"));
    assert!(!output_str.contains("path"));
    assert!(!output_str.contains("\"docs\""));
}

#[test]
fn test_inject_omnidoc_idempotent() {
    let input = quote! {
        #[syncdoc::omnidoc]
        pub fn test() {}
    };

    let output = inject_omnidoc_attr(input.clone(), "docs", DocsPathMode::InlinePaths);

    // Should return unchanged (no duplicate attribute)
    assert_eq!(output.to_string(), input.to_string());
}

#[test]
fn test_inject_omnidoc_idempotent_with_path() {
    let input = quote! {
        #[syncdoc::omnidoc(path = "docs")]
        pub fn test() {}
    };

    let output = inject_omnidoc_attr(input.clone(), "docs", DocsPathMode::InlinePaths);

    // Should return unchanged
    assert_eq!(output.to_string(), input.to_string());
}

#[test]
fn test_inject_omnidoc_idempotent_short_form() {
    let input = quote! {
        #[omnidoc]
        pub fn test() {}
    };

    let output = inject_omnidoc_attr(input.clone(), "docs", DocsPathMode::TomlConfig);

    // Should return unchanged (recognizes short form too)
    assert_eq!(output.to_string(), input.to_string());
}

#[test]
fn test_inject_omnidoc_with_other_attrs() {
    let input = quote! {
        #[derive(Debug)]
        #[syncdoc::omnidoc]
        #[cfg(test)]
        pub fn test() {}
    };

    let output = inject_omnidoc_attr(input.clone(), "docs", DocsPathMode::InlinePaths);

    // Should return unchanged even with other attributes
    assert_eq!(output.to_string(), input.to_string());
}

#[test]
fn test_multiple_annotations_same_file() {
    use crate::discover::ParsedFile;
    use std::path::PathBuf;

    let source = r#"
#[syncdoc::omnidoc]
pub fn existing() {}

pub fn new_function() {}
"#;

    let content = proc_macro2::TokenStream::from_str(source).unwrap();
    let parsed_content = content
        .into_token_iter()
        .parse::<syncdoc_core::parse::ModuleContent>()
        .unwrap();

    let parsed = ParsedFile {
        path: PathBuf::from("test.rs"),
        content: parsed_content,
        original_source: source.to_string(),
    };

    // Run annotation twice
    let result1 = rewrite_file(&parsed, "docs", DocsPathMode::TomlConfig, false, true).unwrap();

    // Parse result and run again
    let content1 = proc_macro2::TokenStream::from_str(&result1).unwrap();
    let parsed_content1 = content1
        .into_token_iter()
        .parse::<syncdoc_core::parse::ModuleContent>()
        .unwrap();

    let parsed1 = ParsedFile {
        path: PathBuf::from("test.rs"),
        content: parsed_content1,
        original_source: result1.clone(),
    };

    let result2 = rewrite_file(&parsed1, "docs", DocsPathMode::TomlConfig, false, true).unwrap();

    // Second run should produce identical output (idempotent)
    assert_eq!(result1, result2);

    // Count omnidoc occurrences - should only have 2 (one for each function)
    let omnidoc_count = result2.matches("omnidoc").count();
    assert_eq!(omnidoc_count, 2, "Should have exactly 2 omnidoc attributes");
}
