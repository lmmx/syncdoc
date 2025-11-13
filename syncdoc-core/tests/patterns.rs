// syncdoc-core/tests/patterns.rs
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
fn test_basic_function() {
    let code = r#"
fn hello() {
    println!("world");
}
"#;

    assert_snapshot!(test_with_code("test_basic_function_pattern", code));
}

#[test]
fn test_pub_function() {
    let code = r#"
pub fn hello() {
    println!("world");
}
"#;

    assert_snapshot!(test_with_code("test_pub_function", code));
}

#[test]
fn test_async_function() {
    let code = r#"
async fn hello() {
    println!("world");
}
"#;

    assert_snapshot!(test_with_code("test_async_function_pattern", code));
}

#[test]
fn test_pub_async_function() {
    let code = r#"
pub async fn hello() {
    println!("world");
}
"#;

    assert_snapshot!(test_with_code("test_pub_async_function_pattern", code));
}

#[test]
fn test_unsafe_function() {
    let code = r#"
unsafe fn hello() {
    println!("world");
}
"#;

    assert_snapshot!(test_with_code("test_unsafe_function_pattern", code));
}

#[test]
fn test_const_function() {
    let code = r#"
const fn hello() -> i32 {
    42
}
"#;

    assert_snapshot!(test_with_code("test_const_function_pattern", code));
}

#[test]
fn test_extern_c_function() {
    let code = r#"
extern "C" fn hello() {
    println!("world");
}
"#;

    assert_snapshot!(test_with_code("test_extern_c_function", code));
}

#[test]
fn test_pub_crate_function() {
    let code = r#"
pub(crate) fn hello() {
    println!("world");
}
"#;

    assert_snapshot!(test_with_code("test_pub_crate_function", code));
}

#[test]
fn test_impl_block_methods() {
    let code = r#"
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
"#;

    assert_snapshot!(test_with_code("test_impl_block_methods_pattern", code));
}

#[test]
fn test_trait_methods() {
    let code = r#"
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
"#;

    assert_snapshot!(test_with_code("test_trait_methods_pattern", code));
}

#[test]
fn test_mixed_content_with_functions() {
    let code = r#"
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

pub unsafe fn another_function() -> Result<(), String> {
    Ok(())
}

type MyType = HashMap<String, i32>;
"#;

    assert_snapshot!(test_with_code("test_mixed_content_with_functions", code));
}
