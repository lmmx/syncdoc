use super::*;
use proc_macro2::TokenStream;
use quote::quote;

fn parse_attrs(tokens: TokenStream) -> Option<Many<Attribute>> {
    let mut iter = tokens.into_token_iter();
    iter.parse::<Many<Attribute>>().ok()
}

#[test]
fn test_extract_empty() {
    let result = extract_doc_content(&None);
    assert_eq!(result, None);

    // Just skip testing with manually constructed empty Many
    // since the type is complex. Instead test that parsing empty
    // token stream gives us None
    let tokens = quote! {};
    let attrs = parse_attrs(tokens);
    let result = extract_doc_content(&attrs);
    assert_eq!(result, None);
}

#[test]
fn test_extract_single() {
    let tokens = quote! {
        #[doc = "This is documentation"]
    };

    let attrs = parse_attrs(tokens);
    let result = extract_doc_content(&attrs);

    assert_eq!(result, Some("This is documentation".to_string()));
}

#[test]
fn test_extract_multiple() {
    let tokens = quote! {
        #[doc = "First line"]
        #[doc = "Second line"]
        #[doc = "Third line"]
    };

    let attrs = parse_attrs(tokens);
    let result = extract_doc_content(&attrs);

    assert_eq!(
        result,
        Some("First line\nSecond line\nThird line".to_string())
    );
}

#[test]
fn test_extract_preserves_formatting() {
    let tokens = quote! {
        #[doc = "# Header"]
        #[doc = ""]
        #[doc = "- List item 1"]
        #[doc = "- List item 2"]
    };

    let attrs = parse_attrs(tokens);
    let result = extract_doc_content(&attrs);

    assert_eq!(
        result,
        Some("# Header\n\n- List item 1\n- List item 2".to_string())
    );
}

#[test]
fn test_extract_ignores_non_doc() {
    let tokens = quote! {
        #[derive(Debug)]
        #[doc = "Actual documentation"]
        #[cfg(test)]
    };

    let attrs = parse_attrs(tokens);
    let result = extract_doc_content(&attrs);

    assert_eq!(result, Some("Actual documentation".to_string()));
}

#[test]
fn test_has_doc_attrs() {
    let tokens = quote! {
        #[derive(Debug)]
    };
    let attrs = parse_attrs(tokens);
    assert!(!has_doc_attrs(&attrs));

    let tokens = quote! {
        #[doc = "Has docs"]
    };
    let attrs = parse_attrs(tokens);
    assert!(has_doc_attrs(&attrs));
}

#[test]
fn test_extract_strips_leading_spaces() {
    // Simulate what Rust actually does with doc comments
    let tokens = quote! {
        #[doc = " First line"]
        #[doc = " Second line with  intentional indent"]
        #[doc = "  Third line with more indent"]
    };

    let attrs = parse_attrs(tokens);
    let result = extract_doc_content(&attrs);

    assert_eq!(
        result,
        Some(
            "First line\nSecond line with  intentional indent\n Third line with more indent"
                .to_string()
        )
    );
}
