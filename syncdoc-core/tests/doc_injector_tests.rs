mod helpers;

use helpers::TestFixture;
use proc_macro2::TokenStream;
use quote::quote;
use rust_format::{Formatter, RustFmt};
use std::fs;
use syncdoc_core::syncdoc_impl;

fn format_and_print(tokens: TokenStream) -> String {
    let fmt_str = RustFmt::default()
        .format_tokens(tokens)
        .unwrap_or_else(|e| panic!("Format error: {}", e));
    println!("Generated code: {}", fmt_str);
    fmt_str
}

#[test]
fn test_basic_doc_injection() {
    let fixture = TestFixture::new();
    fixture.create_doc_file("test_function.md");

    let path = fixture.docs_path();
    let args = quote!(path = #path);
    let item = quote! {
        fn test_function(x: u32) -> u32 {
            x + 1
        }
    };

    let result = syncdoc_impl(args, item);
    assert!(result.is_ok());

    let output = result.unwrap();
    let output_str = format_and_print(output);

    assert!(output_str.replace(" ", "").contains("include_str!"));
    assert!(output_str.contains("test_function.md"));
    assert!(output_str.contains("fn test_function"));
}

#[test]
fn test_custom_name() {
    let fixture = TestFixture::new();
    fixture.create_doc_file("custom.md");

    let path = fixture.docs_path();
    let args = quote!(path = #path, name = "custom");
    let item = quote! {
        fn test_function() {}
    };

    let result = syncdoc_impl(args, item);
    assert!(result.is_ok());

    let output = result.unwrap();
    let output_str = format_and_print(output);

    assert!(output_str.contains("custom.md"));
}

#[test]
fn test_async_function_doc() {
    let fixture = TestFixture::new();
    fixture.create_doc_file("test_async.md");

    let path = fixture.docs_path();
    let args = quote!(path = #path);
    let item = quote! {
        async fn test_async() {
            println!("async test");
        }
    };

    let result = syncdoc_impl(args, item);
    assert!(result.is_ok());

    let output = result.unwrap();
    let output_str = format_and_print(output);

    assert!(output_str.contains("async fn test_async"));
    assert!(output_str.replace(" ", "").contains("include_str!"));
}

#[test]
fn test_unsafe_function_doc() {
    let fixture = TestFixture::new();
    fixture.create_doc_file("test_unsafe.md");

    let path = fixture.docs_path();
    let args = quote!(path = #path);
    let item = quote! {
        unsafe fn test_unsafe() {
            println!("unsafe test");
        }
    };

    let result = syncdoc_impl(args, item);
    assert!(result.is_ok());

    let output = result.unwrap();
    let output_str = format_and_print(output);

    assert!(output_str.contains("unsafe fn test_unsafe"));
    assert!(output_str.replace(" ", "").contains("include_str!"));
}

#[test]
fn test_pub_async_function_doc() {
    let fixture = TestFixture::new();
    fixture.create_doc_file("test_pub_async.md");

    let path = fixture.docs_path();
    let args = quote!(path = #path);
    let item = quote! {
        pub async fn test_pub_async() {
            println!("pub async test");
        }
    };

    let result = syncdoc_impl(args, item);
    assert!(result.is_ok());

    let output = result.unwrap();
    let output_str = format_and_print(output);

    assert!(output_str.contains("pub async fn test_pub_async"));
    assert!(output_str.replace(" ", "").contains("include_str!"));
}

#[test]
fn test_direct_file_path() {
    let fixture = TestFixture::new();
    let special_path = fixture._temp_dir.path().join("special.md");
    fs::write(&special_path, "# Special doc\n").expect("Failed to write special.md");

    let path = special_path.to_string_lossy().to_string();
    let args = quote!(path = #path);
    let item = quote! {
        fn test_function() {}
    };

    let result = syncdoc_impl(args, item);
    assert!(result.is_ok());

    let output = result.unwrap();
    let output_str = format_and_print(output);

    assert!(output_str.contains("special.md"));
    assert!(!output_str.contains("test_function.md"));
}
