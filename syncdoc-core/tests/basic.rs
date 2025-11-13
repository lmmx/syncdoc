// syncdoc-core/tests/basic.rs
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

    // Return the written lib.rs for snapshotting
    crate_under_test.get_expanded_lib().unwrap()
}

#[test]
fn test_single_function() {
    let code = r#"
fn hello() {
    println!("world");
}
"#;

    assert_snapshot!(test_with_code("test_single_function", code));
}

#[test]
fn test_multiple_functions() {
    let code = r#"
fn foo(x: i32) -> i32 {
    bar(x + 1)
}

fn bar(y: i32) -> i32 {
    y * 2
}
"#;

    assert_snapshot!(test_with_code("test_multiple_functions", code));
}

#[test]
fn test_generic_function() {
    let code = r#"
fn generic<T: Clone>(value: T) -> T {
    value.clone()
}
"#;

    assert_snapshot!(test_with_code("test_generic_function", code));
}

#[test]
fn test_ignores_non_functions() {
    let code = r#"
const x: String = "fn not_a_function";
struct Foo { field: i32 }
fn actual_function() {}
"#;

    assert_snapshot!(test_with_code("test_ignores_non_functions", code));
}

#[test]
fn test_module_with_functions() {
    let code = r#"
mod calculations {
    pub fn fibonacci(n: u64) -> u64 {
        if n <= 1 {
            n
        } else {
            add_numbers(fibonacci(n - 1), fibonacci(n - 2))
        }
    }

    fn add_numbers(a: u64, b: u64) -> u64 {
        a + b
    }
}
"#;

    assert_snapshot!(test_with_code("test_module_with_functions", code));
}

#[test]
fn test_impl_block_methods() {
    let code = r#"
impl Calculator {
    pub fn new() -> Self {
        Self
    }

    pub fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    pub fn multiply(&self, x: i32, y: i32) -> i32 {
        x * y
    }

    fn internal_helper(&self, value: i32) -> i32 {
        value * 2
    }
}
"#;

    assert_snapshot!(test_with_code("test_impl_block_methods", code));
}

#[test]
fn test_impl_block_with_generics() {
    let code = r#"
impl<T> Container<T>
where
    T: Clone + std::fmt::Debug,
{
    pub fn new(value: T) -> Self {
        Self { inner: value }
    }

    pub fn get(&self) -> &T {
        &self.inner
    }

    pub fn set(&mut self, new_value: T) {
        self.inner = new_value;
    }
}
"#;

    assert_snapshot!(test_with_code("test_impl_block_with_generics", code));
}
