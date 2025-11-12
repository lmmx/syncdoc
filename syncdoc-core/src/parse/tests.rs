//! Unit tests for parse module
use super::*;
use proc_macro2::TokenStream;
use quote::quote;

fn parse_fn_sig(input: TokenStream) -> Result<FnSig> {
    let mut iter = input.into_token_iter();
    iter.parse::<FnSig>()
}

fn fn_sig_to_tokens(sig: FnSig) -> TokenStream {
    let mut tokens = TokenStream::new();
    sig.to_tokens(&mut tokens);
    tokens
}

#[test]
fn test_basic_function() {
    let input = quote! { fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse");

    let output = fn_sig_to_tokens(parsed);
    let output_str = output.to_string();
    assert!(output_str.contains("fn"));
    assert!(output_str.contains("hello"));
    assert!(output_str.contains("()"));
    assert!(output_str.contains("{ }"));
}

#[test]
fn test_async_function() {
    let input = quote! { async fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse async fn");

    assert!(parsed.async_kw.is_some());
    assert!(parsed.unsafe_kw.is_none());
    assert_eq!(parsed.name.to_string(), "hello");

    let output = fn_sig_to_tokens(parsed);
    assert!(output.to_string().contains("async"));
}

#[test]
fn test_path_arg_parsing() {
    let input = quote!(path = "../docs");
    let mut iter = input.into_token_iter();

    match iter.parse::<SyncDocInner>() {
        Ok(parsed) => {
            assert!(parsed.args.is_some(), "Should have parsed arguments");
            let args = parsed.args.as_ref().unwrap();
            assert_eq!(args.0.len(), 1, "Should have 1 argument");

            if let SyncDocArg::Path(path_arg) = &args.0[0].value {
                assert_eq!(path_arg.value.as_str(), "../docs");
            } else {
                panic!("Expected Path argument");
            }
        }
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_syncdoc_inner_parsing() {
    let input = quote!(path = "../docs", name = "custom");
    let mut iter = input.into_token_iter();

    match iter.parse::<SyncDocInner>() {
        Ok(parsed) => {
            assert!(parsed.args.is_some());
            let args = parsed.args.as_ref().unwrap();
            assert_eq!(args.0.len(), 2, "Should have 2 arguments");

            let mut found_path = false;
            let mut found_name = false;

            for arg in &args.0 {
                match &arg.value {
                    SyncDocArg::Path(path_arg) => {
                        assert_eq!(path_arg.value.as_str(), "../docs");
                        found_path = true;
                    }
                    SyncDocArg::Name(name_arg) => {
                        assert_eq!(name_arg.value.as_str(), "custom");
                        found_name = true;
                    }
                    SyncDocArg::CfgAttr(_) => {
                        // Not testing cfg-attr in this test
                    }
                }
            }

            assert!(found_path, "Should find Path argument");
            assert!(found_name, "Should find Name argument");
        }
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_pub_async_function() {
    let input = quote! { pub async fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse pub async fn");

    assert!(parsed.visibility.is_some());
    assert!(parsed.async_kw.is_some());
    assert_eq!(parsed.name.to_string(), "hello");
}

#[test]
fn test_unsafe_function() {
    let input = quote! { unsafe fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse unsafe fn");

    assert!(parsed.unsafe_kw.is_some());
    assert_eq!(parsed.name.to_string(), "hello");
}

#[test]
fn test_const_function() {
    let input = quote! { const fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse const fn");

    assert!(parsed.const_kw.is_some());
    assert_eq!(parsed.name.to_string(), "hello");
}
