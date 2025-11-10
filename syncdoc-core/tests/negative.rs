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
fn test_ignores_function_calls_in_expressions() {
    let input = quote! {
        fn outer_function() {
            let result = some_fn_call();
            another_fn_call(42, "hello");
            nested::module::fn_call();
            obj.method_fn_call();
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_ignores_fn_in_string_literals() {
    let input = quote! {
        fn real_function() {
            let msg = "This fn is not a function";
            let code = r#"fn fake_function() { return "not real"; }"#;
            println!("fn appears in this string too");
        }

        const TEMPLATE: &str = "fn template_function() {}";
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_ignores_fn_in_comments() {
    let input = quote! {
        // fn this_is_commented_out() {}
        /* fn this_is_also_commented() {} */

        fn actual_function() {
            // fn another_comment_function() {}
            println!("Hello");
        }

        /// Documentation comment with fn example() {}
        fn documented_function() {}
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_basic_function_gets_documented() {
    let input = quote! {
        fn real_function() {
            println!("Real function");
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_ignores_type_alias_with_fn() {
    let input = quote! {
        type FnPointer = fn() -> i32;
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_function_with_fn_type_parameter() {
    let input = quote! {
        fn function_with_fn_param(callback: fn(i32) -> String) -> String {
            callback(42)
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_function_returning_fn_type() {
    let input = quote! {
        fn returns_fn() -> fn() -> i32 {
            || 42
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_trait_method_declarations() {
    let input = quote! {
        trait MyTrait {
            fn trait_method(&self);

            fn default_method(&self) {
                println!("This has a body and should be documented");
            }
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_trait_with_default_method() {
    let input = quote! {
        trait MyTrait {
            fn default_method(&self) {
                println!("This has a body and should be documented");
            }
        }

        struct MyStruct;

        impl MyTrait for MyStruct {}

        fn main() {
            let my_struct = MyStruct;
            my_struct.default_method();
        }
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}

#[test]
fn test_complex_edge_cases() {
    let input = quote! {
        fn legitimate_function() {
            // fn this_is_just_a_comment
            let variable = "fn not_a_function";
            some_function_call();

            if condition {
                another_fn_call();
            }

            match value {
                Pattern => yet_another_fn_call(),
                _ => final_fn_call(),
            }
        }

        struct MyStruct {
            field: String,
        }

        const CODE_SAMPLE: &str = r#"
            fn example() {
                println!("This fn is in a string");
            }
        "#;
    };

    assert_snapshot!(apply_doc_injection(input, "../docs"));
}