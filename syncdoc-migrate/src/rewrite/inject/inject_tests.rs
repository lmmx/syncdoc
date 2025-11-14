// syncdoc-migrate/src/rewrite/tests.rs

use super::*;
use quote::quote;

#[test]
fn test_inject_omnidoc_before_visibility() {
    let input = quote! {
        pub fn test() {}
    };

    let output = inject_omnidoc_attr(input, "docs");
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

    let output = inject_omnidoc_attr(input, "docs");
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

    let output = inject_omnidoc_attr(input, "docs");
    let output_str = output.to_string();

    assert!(output_str.contains("omnidoc"));
    assert!(output_str.contains("\"docs\""));
}
