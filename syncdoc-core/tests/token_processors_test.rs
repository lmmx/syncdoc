mod helpers;

use helpers::TestFixture;
use quote::quote;
use syncdoc_core::token_processors::TokenProcessor;

#[test]
fn test_basic_function_processing() {
    let fixture = TestFixture::new();
    fixture.create_doc_file("hello.md");

    let input = quote! { fn hello() { println!("world"); } };
    let processor = TokenProcessor::new(input.clone(), fixture.docs_path(), None);
    let output = processor.process();

    println!("Input: {}", input);
    println!("Output: {}", output);

    let output_str = output.to_string();
    assert!(output_str.contains("fn hello"));
    assert!(output_str.replace(" ", "").contains("include_str!"));
}

#[test]
fn test_async_function_processing() {
    let fixture = TestFixture::new();
    fixture.create_doc_file("hello.md");

    let input = quote! { async fn hello() { println!("world"); } };
    let processor = TokenProcessor::new(input.clone(), fixture.docs_path(), None);
    let output = processor.process();

    println!("Input: {}", input);
    println!("Output: {}", output);

    let output_str = output.to_string();
    assert!(
        output_str.contains("async fn hello"),
        "Should preserve async keyword"
    );
    assert!(
        output_str.replace(" ", "").contains("include_str!"),
        "Should add documentation"
    );
}

#[test]
fn test_impl_block_processing() {
    let fixture = TestFixture::new();
    fixture.create_doc_file("MyStruct/method.md");

    let input = quote! {
        impl MyStruct {
            fn method(&self) {
                println!("method");
            }
        }
    };

    let processor = TokenProcessor::new(input.clone(), fixture.docs_path(), None);
    let output = processor.process();

    println!("Impl block input: {}", input);
    println!("Impl block output: {}", output);

    let output_str = output.to_string();
    assert!(output_str.contains("fn method"), "Should preserve method");
    assert!(
        output_str.replace(" ", "").contains("include_str!"),
        "Should add documentation"
    );
    assert!(
        output_str.contains("MyStruct"),
        "Should include struct name in path"
    );
}

#[test]
fn test_nested_module_path_construction() {
    let fixture = TestFixture::new();
    fixture.create_doc_file("outer/outer_fn.md");
    fixture.create_doc_file("outer/inner/inner_fn.md");

    let input = quote! {
        mod outer {
            fn outer_fn() {}

            mod inner {
                fn inner_fn() {}
            }
        }
    };

    let processor = TokenProcessor::new(input.clone(), fixture.docs_path(), None);
    let output = processor.process();

    println!("Nested module input: {}", input);
    println!("Nested module output: {}", output);

    let output_str = output.to_string();

    // Should have docs for outer function
    assert!(output_str.contains("outer/outer_fn.md"));

    // Should have docs for inner function
    assert!(output_str.contains("outer/inner/inner_fn.md"));
}

#[test]
fn test_impl_block_path_construction() {
    let fixture = TestFixture::new();
    fixture.create_doc_file("Calculator/add.md");

    let input = quote! {
        impl Calculator {
            fn add(&self, a: i32, b: i32) -> i32 {
                a + b
            }
        }
    };

    let processor = TokenProcessor::new(input.clone(), fixture.docs_path(), None);
    let output = processor.process();

    println!("Impl path test input: {}", input);
    println!("Impl path test output: {}", output);

    let output_str = output.to_string();
    assert!(output_str.contains("Calculator/add.md"));
}
