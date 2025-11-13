// syncdoc-core/tests/negative.rs
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
fn test_ignores_function_calls_in_expressions() {
    let code = r#"
fn outer_function() {
    let result = some_fn_call();
    another_fn_call(42, "hello");
    nested::module::fn_call();
    obj.method_fn_call();
}
"#;

    assert_snapshot!(test_with_code("test_ignores_function_calls", code));
}

#[test]
fn test_ignores_fn_in_string_literals() {
    let code = r##"
fn real_function() {
    let msg = "This fn is not a function";
    let code_str = r#"fn fake_function() { return "not real"; }"#;
    println!("fn appears in this string too");
}

const TEMPLATE: &str = "fn template_function() {}";
"##;

    assert_snapshot!(test_with_code("test_ignores_fn_in_string_literals", code));
}

#[test]
fn test_ignores_fn_in_comments() {
    let code = r#"
// fn this_is_commented_out() {}
/* fn this_is_also_commented() {} */

fn actual_function() {
    // fn another_comment_function() {}
    println!("Hello");
}

/// Documentation comment with fn example() {}
fn documented_function() {}
"#;

    assert_snapshot!(test_with_code("test_ignores_fn_in_comments", code));
}

#[test]
fn test_basic_function_gets_documented() {
    let code = r#"
fn real_function() {
    println!("Real function");
}
"#;

    assert_snapshot!(test_with_code("test_basic_function_gets_documented", code));
}

#[test]
fn test_ignores_type_alias_with_fn() {
    let code = r#"
type FnPointer = fn() -> i32;
"#;

    assert_snapshot!(test_with_code("test_ignores_type_alias_with_fn", code));
}

#[test]
fn test_function_with_fn_type_parameter() {
    let code = r#"
fn function_with_fn_param(callback: fn(i32) -> String) -> String {
    callback(42)
}
"#;

    assert_snapshot!(test_with_code("test_function_with_fn_type_parameter", code));
}

#[test]
fn test_function_returning_fn_type() {
    let code = r#"
fn returns_fn() -> fn() -> i32 {
    || 42
}
"#;

    assert_snapshot!(test_with_code("test_function_returning_fn_type", code));
}

#[test]
fn test_trait_method_declarations() {
    let code = r#"
trait MyTrait {
    fn trait_method(&self);

    fn default_method(&self) {
        println!("This has a body and should be documented");
    }
}
"#;

    assert_snapshot!(test_with_code("test_trait_method_declarations", code));
}

#[test]
fn test_trait_with_default_method() {
    let code = r#"
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
"#;

    assert_snapshot!(test_with_code("test_trait_with_default_method", code));
}

#[test]
fn test_complex_edge_cases() {
    let code = r##"
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
"##;

    assert_snapshot!(test_with_code("test_complex_edge_cases", code));
}
