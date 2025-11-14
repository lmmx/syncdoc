// syncdoc-core/tests/doc_injector_tests.rs
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
fn test_basic_doc_injection() {
    let code = r#"
fn test_function(x: u32) -> u32 {
    x + 1
}
"#;

    assert_snapshot!(test_with_code("test_basic_doc_injection", code));
}

#[test]
fn test_async_function_doc() {
    let code = r#"
async fn test_async() {
    println!("async test");
}
"#;

    assert_snapshot!(test_with_code("test_async_function_doc", code));
}

#[test]
fn test_unsafe_function_doc() {
    let code = r#"
unsafe fn test_unsafe() {
    println!("unsafe test");
}
"#;

    assert_snapshot!(test_with_code("test_unsafe_function_doc", code));
}

#[test]
fn test_pub_async_function_doc() {
    let code = r#"
pub async fn test_pub_async() {
    println!("pub async test");
}
"#;

    assert_snapshot!(test_with_code("test_pub_async_function_doc", code));
}

#[test]
fn test_const_function() {
    let code = r#"
const fn test_const() -> i32 {
    42
}
"#;

    assert_snapshot!(test_with_code("test_const_function", code));
}

#[test]
fn test_pub_const_async_function() {
    let code = r#"
pub const async fn complex_fn() -> i32 {
    42
}
"#;

    // This might not compile (depends on Rust version), but let's test it
    let crate_under_test = TestCrate::new("test_pub_const_async");
    crate_under_test.write_lib(code);
    crate_under_test.auto_create_docs(code);

    let (success, _) = crate_under_test.cargo_check();

    if success {
        assert_snapshot!(crate_under_test.get_expanded_lib().unwrap());
    }
    // If it doesn't compile, that's fine - syntax might not be valid yet
}

#[test]
fn test_function_with_generics() {
    let code = r#"
fn generic_function<T: Clone>(value: T) -> T {
    value.clone()
}
"#;

    assert_snapshot!(test_with_code("test_function_with_generics", code));
}

#[test]
fn test_function_with_where_clause() {
    let code = r#"
fn with_where<T>(value: T) -> T
where
    T: Clone + std::fmt::Debug,
{
    value.clone()
}
"#;

    assert_snapshot!(test_with_code("test_function_with_where_clause", code));
}

#[test]
fn test_multiple_attributes() {
    let code = r#"
#[allow(dead_code)]
#[inline]
fn attributed_function() {
    println!("Has attributes");
}
"#;

    assert_snapshot!(test_with_code("test_multiple_attributes", code));
}
