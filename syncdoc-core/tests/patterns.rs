use syncdoc_core::inject_all_docs_impl;
use insta::assert_snapshot;
use proc_macro2::TokenStream;
use quote::quote;
use rust_format::{Formatter, RustFmt};

fn apply_doc_injection(input: TokenStream, path: &str) -> String {
    let args = quote!(path = #path);
    let output = inject_all_docs_impl(args, input).expect("Should inject docs successfully");
    let fmt_str = RustFmt::default()
        .format_tokens(output)
        .unwrap_or_else(|e| panic!("Format error: {}", e));
    fmt_str
}

#[test]
fn test_basic_function() {
    let input = quote! {
        fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_pub_function() {
    let input = quote! {
        pub fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_async_function() {
    let input = quote! {
        async fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_pub_async_function() {
    let input = quote! {
        pub async fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_unsafe_function() {
    let input = quote! {
        unsafe fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_const_function() {
    let input = quote! {
        const fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_extern_c_function() {
    let input = quote! {
        extern "C" fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_pub_crate_function() {
    let input = quote! {
        pub(crate) fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_impl_block_methods() {
    let input = quote! {
        impl MyStruct {
            fn method(&self) {
                println!("method");
            }

            pub async fn async_method(&mut self) -> i32 {
                42
            }

            unsafe fn unsafe_method() {
                println!("unsafe");
            }
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_trait_methods() {
    let input = quote! {
        trait MyTrait {
            fn required_method(&self);

            async fn async_trait_method(&self) -> String {
                "default".to_string()
            }

            unsafe fn unsafe_trait_method();

            const fn const_trait_method() -> i32 {
                0
            }
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_mixed_content_with_functions() {
    let input = quote! {
        use std::collections::HashMap;

        const SOME_CONST: i32 = 42;

        struct MyStruct {
            field: String,
        }

        async fn actual_function() {
            println!("This should be documented");
        }

        enum MyEnum {
            Variant1,
            Variant2(i32),
        }

        pub unsafe fn another_function() -> Result<(), Error> {
            Ok(())
        }

        type MyType = HashMap<String, i32>;
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}