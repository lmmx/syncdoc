use braces::{brace_paths, BraceConfig};
use insta::assert_snapshot;
use syncdoc_migrate::{
    discover::parse_file,
    write::{extract_all_docs, write_extracts},
};

mod helpers;
use helpers::*;

fn to_braces(paths: &[&str]) -> String {
    let braces_config = BraceConfig::default();
    brace_paths(paths, &braces_config).expect("Brace error")
}

fn parse_and_get_paths(source: &str, filename: &str, docs_dir: &str) -> Vec<String> {
    let (_temp_dir, file_path) = setup_test_file(source, filename);
    let parsed = parse_file(&file_path).unwrap();
    let extracts = extract_all_docs(&parsed, docs_dir);
    extracts
        .iter()
        .map(|e| e.markdown_path.to_str().unwrap().to_string())
        .collect()
}

fn get_path_refs(paths: &[String]) -> Vec<&str> {
    paths.iter().map(|s| s.as_str()).collect()
}

#[test]
fn test_extract_and_write_function_docs() {
    let paths = parse_and_get_paths(
        r#"
        /// A simple function
        pub fn my_function() {
            println!("Hello");
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/my_function.md");

    // Verify content separately
    let (_temp_dir, file_path) = setup_test_file(
        r#"
        /// A simple function
        pub fn my_function() {
            println!("Hello");
        }
        "#,
        "test.rs",
    );
    let parsed = parse_file(&file_path).unwrap();
    let extracts = extract_all_docs(&parsed, "docs");
    assert_eq!(extracts[0].content, "A simple function\n");
}

#[test]
fn test_extract_and_write_module_docs() {
    let paths = parse_and_get_paths(
        r#"
        /// Module documentation
        pub mod my_module {
            /// Inner function
            pub fn inner_func() {}
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/my_module/{inner_func,}.md");

    // Verify content
    let (_temp_dir, file_path) = setup_test_file(
        r#"
        /// Module documentation
        pub mod my_module {
            /// Inner function
            pub fn inner_func() {}
        }
        "#,
        "test.rs",
    );
    let parsed = parse_file(&file_path).unwrap();
    let extracts = extract_all_docs(&parsed, "docs");

    let module_doc = extracts
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap() == "docs/my_module.md")
        .expect("Should find module doc");
    assert_eq!(module_doc.content, "Module documentation\n");

    let func_doc = extracts
        .iter()
        .find(|e| e.markdown_path.to_str().unwrap() == "docs/my_module/inner_func.md")
        .expect("Should find inner function doc");
    assert_eq!(func_doc.content, "Inner function\n");
}

#[test]
fn test_extract_and_write_impl_method_docs() {
    let paths = parse_and_get_paths(
        r#"
        struct MyType;

        impl MyType {
            /// A method
            pub fn my_method(&self) {}

            /// Another method
            fn another_method() {}
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/MyType/{my,another}_method.md");
}

#[test]
fn test_extract_and_write_struct_and_field_docs() {
    let paths = parse_and_get_paths(
        r#"
        /// A documented struct
        pub struct MyStruct {
            /// First field
            pub field1: String,

            /// Second field
            field2: i32,

            /// Third field
            pub field3: bool,
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/MyStruct/{field1,field2,field3,}.md");
}

#[test]
fn test_extract_and_write_enum_and_variant_docs() {
    let paths = parse_and_get_paths(
        r#"
        /// An enum
        pub enum MyEnum {
            /// First variant
            Variant1,

            /// Second variant
            Variant2(i32),

            /// Third variant
            Variant3 { field: String },
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/MyEnum/{Variant1,Variant2,Variant3,}.md");
}

#[test]
fn test_extract_and_write_trait_method_docs() {
    let paths = parse_and_get_paths(
        r#"
        /// A trait
        pub trait MyTrait {
            /// Default method with body
            fn default_method(&self) {
                println!("default");
            }
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/MyTrait/{default_method,}.md");
}

#[test]
fn test_extract_const_static_type_alias() {
    let paths = parse_and_get_paths(
        r#"
        /// A constant
        const MY_CONST: i32 = 42;

        /// A static
        static MY_STATIC: &str = "hello";

        /// A type alias
        type MyType = Vec<String>;
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/{MY_CONST.md,MY_STATIC.md,MyType.md}");
}

#[test]
fn test_nested_modules_create_correct_paths() {
    let paths = parse_and_get_paths(
        r#"
        pub mod outer {
            pub mod inner {
                /// Deeply nested function
                pub fn deep_func() {}
            }
        }
        "#,
        "test.rs",
        "docs",
    );

    assert_snapshot!(to_braces(&get_path_refs(&paths)), @"docs/outer/inner/deep_func.md");
}

#[test]
fn test_write_extracts_creates_files() {
    let source = r#"
        /// A function
        fn my_func() {}

        pub mod submod {
            /// Submodule function
            fn sub_func() {}
        }
    "#;

    let (temp_dir, file_path) = setup_test_file(source, "test.rs");
    let docs_dir = temp_dir.path().join("docs");

    let parsed = parse_file(&file_path).unwrap();
    let extracts = extract_all_docs(&parsed, docs_dir.to_str().unwrap());

    let report = write_extracts(&extracts, false).unwrap();
    assert_report(&report, 2);

    assert_file(docs_dir.join("my_func.md"), "A function\n");
    assert_file(docs_dir.join("submod/sub_func.md"), "Submodule function\n");
}

#[test]
fn test_dry_run_does_not_create_files() {
    let source = r#"
        /// A function
        fn my_func() {}
    "#;

    let (temp_dir, file_path) = setup_test_file(source, "test.rs");
    let docs_dir = temp_dir.path().join("docs");

    let parsed = parse_file(&file_path).unwrap();
    let extracts = extract_all_docs(&parsed, docs_dir.to_str().unwrap());

    let report = write_extracts(&extracts, true).unwrap();

    assert_eq!(report.files_written, 1);
    assert!(!docs_dir.exists(), "Dry run should not create directories");
}
