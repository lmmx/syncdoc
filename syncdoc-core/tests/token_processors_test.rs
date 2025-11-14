// syncdoc-core/tests/token_processors_test.rs
use insta::assert_snapshot;

mod helpers;
use helpers::TestCrate;

fn test_with_code(name: &str, code: &str) -> String {
    let crate_under_test = TestCrate::new(name);
    crate_under_test.write_lib(code);
    crate_under_test.auto_create_docs(code);

    let (success, stderr) = crate_under_test.cargo_check();

    if !success {
        panic!("Compilation failed!\nSTDERR:\n{}", stderr);
    }

    crate_under_test.get_expanded_lib().unwrap()
}

#[test]
fn test_basic_function_processing() {
    let code = r#"
fn hello() {
    println!("world");
}
"#;
    assert_snapshot!(test_with_code("test_basic_function_processing", code));
}

#[test]
fn test_async_function_processing() {
    let code = r#"
async fn hello() {
    println!("world");
}
"#;
    let result = test_with_code("test_async_function_processing", code);
    assert_snapshot!(result);
}

#[test]
fn test_impl_block_processing() {
    let code = r#"
impl MyStruct {
    fn method(&self) {
        println!("method");
    }
}
"#;
    let result = test_with_code("test_impl_block_processing", code);
    assert_snapshot!(result);
}

#[test]
fn test_nested_module_path_construction() {
    let code = r#"
mod outer {
    fn outer_fn() {}

    mod inner {
        fn inner_fn() {}
    }
}
"#;
    let result = test_with_code("test_nested_module_path_construction", code);
    assert_snapshot!(result);
}

#[test]
fn test_impl_block_path_construction() {
    let code = r#"
impl Calculator {
    fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}
"#;

    let result = test_with_code("test_impl_block_path_construction", code);

    assert_snapshot!(result);
}

#[test]
fn test_multiple_impl_blocks() {
    let code = r#"
impl FirstStruct {
    fn first_method(&self) {}
}

impl SecondStruct {
    fn second_method(&self) {}
}
"#;
    let result = test_with_code("test_multiple_impl_blocks", code);
    assert_snapshot!(result);
}

#[test]
fn test_impl_with_generics() {
    let code = r#"
impl<T> GenericStruct<T>
where
    T: Clone,
{
    fn process(&self, value: T) -> T {
        value.clone()
    }
}
"#;
    let result = test_with_code("test_impl_with_generics", code);
    assert_snapshot!(result);
}

#[test]
fn test_trait_impl() {
    let code = r#"
impl MyTrait for MyStruct {
    fn trait_method(&self) {
        println!("implementation");
    }
}
"#;

    let result = test_with_code("test_trait_impl", code);
    // For trait impls, it should still document the methods
    assert_snapshot!(result);
}

#[test]
fn test_nested_impl_in_module() {
    let code = r#"
mod my_module {
    impl MyStruct {
        fn module_method(&self) {}
    }
}
"#;

    let result = test_with_code("test_nested_impl_in_module", code);
    assert_snapshot!(result);
}

#[test]
fn test_complex_nested_structure() {
    let code = r#"
mod outer {
    pub fn outer_function() {}

    impl OuterStruct {
        pub fn outer_method(&self) {}
    }

    mod inner {
        pub fn inner_function() {}

        impl InnerStruct {
            pub fn inner_method(&self) {}
        }
    }
}
"#;
    let result = test_with_code("test_complex_nested_structure", code);
    assert_snapshot!(result);
}
