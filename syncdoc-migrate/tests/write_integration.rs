use std::fs;
use tempfile::TempDir;
use syncdoc_migrate::{discover::parse_file, write::{extract_all_docs, write_extractions}};

fn setup_test_file(source: &str) -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, source).unwrap();
    (temp_dir, file_path)
}

#[test]
fn test_extract_and_write_function_docs() {
    let source = r#"
        /// A simple function
        pub fn my_function() {
            println!("Hello");
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source);
    let parsed = parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, "docs");

    assert_eq!(extractions.len(), 1);
    assert_eq!(extractions[0].markdown_path.to_str().unwrap(), "docs/my_function.md");
    assert_eq!(extractions[0].content, "A simple function");
}

#[test]
fn test_extract_and_write_module_docs() {
    let source = r#"
        /// Module documentation
        pub mod my_module {
            /// Inner function
            pub fn inner_func() {}
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source);
    let parsed = parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, "docs");

    assert_eq!(extractions.len(), 2);

    let module_doc = extractions.iter()
        .find(|e| e.markdown_path.to_str().unwrap() == "docs/my_module.md")
        .expect("Should find module doc");
    assert_eq!(module_doc.content, "Module documentation");

    let func_doc = extractions.iter()
        .find(|e| e.markdown_path.to_str().unwrap() == "docs/my_module/inner_func.md")
        .expect("Should find inner function doc");
    assert_eq!(func_doc.content, "Inner function");
}

#[test]
fn test_extract_and_write_impl_method_docs() {
    let source = r#"
        struct MyType;

        impl MyType {
            /// A method
            pub fn my_method(&self) {}

            /// Another method
            fn another_method() {}
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source);
    let parsed = parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, "docs");

    assert_eq!(extractions.len(), 2);

    assert!(extractions.iter().any(|e|
        e.markdown_path.to_str().unwrap() == "docs/MyType/my_method.md" &&
        e.content == "A method"
    ));

    assert!(extractions.iter().any(|e|
        e.markdown_path.to_str().unwrap() == "docs/MyType/another_method.md" &&
        e.content == "Another method"
    ));
}

#[test]
fn test_extract_and_write_struct_and_field_docs() {
    let source = r#"
        /// A documented struct
        pub struct MyStruct {
            /// First field
            pub field1: String,
            /// Second field
            field2: i32,
            /// Third field
            pub field3: bool,
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source);
    let parsed = parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, "docs");

    assert_eq!(extractions.len(), 4, "Should extract struct + 3 fields");

    let struct_doc = extractions.iter()
        .find(|e| e.markdown_path.to_str().unwrap() == "docs/MyStruct.md")
        .expect("Should find struct doc");
    assert_eq!(struct_doc.content, "A documented struct");

    for (field_name, expected_content) in [
        ("field1", "First field"),
        ("field2", "Second field"),
        ("field3", "Third field"),
    ] {
        let field_doc = extractions.iter()
            .find(|e| e.markdown_path.to_str().unwrap() == format!("docs/MyStruct/{}.md", field_name))
            .expect(&format!("Should find {} doc", field_name));
        assert_eq!(field_doc.content, expected_content);
    }
}

#[test]
fn test_extract_and_write_enum_and_variant_docs() {
    let source = r#"
        /// An enum
        pub enum MyEnum {
            /// First variant
            Variant1,
            /// Second variant
            Variant2(i32),
            /// Third variant
            Variant3 { field: String },
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source);
    let parsed = parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, "docs");

    assert_eq!(extractions.len(), 4, "Should extract enum + 3 variants");

    let enum_doc = extractions.iter()
        .find(|e| e.markdown_path.to_str().unwrap() == "docs/MyEnum.md")
        .expect("Should find enum doc");
    assert_eq!(enum_doc.content, "An enum");

    for (variant_name, expected_content) in [
        ("Variant1", "First variant"),
        ("Variant2", "Second variant"),
        ("Variant3", "Third variant"),
    ] {
        let variant_doc = extractions.iter()
            .find(|e| e.markdown_path.to_str().unwrap() == format!("docs/MyEnum/{}.md", variant_name))
            .expect(&format!("Should find {} doc", variant_name));
        assert_eq!(variant_doc.content, expected_content);
    }
}

#[test]
fn test_extract_and_write_trait_method_docs() {
    let source = r#"
        /// A trait
        pub trait MyTrait {
            /// Default method with body
            fn default_method(&self) {
                println!("default");
            }
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source);
    let parsed = parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, "docs");

    assert_eq!(extractions.len(), 2);

    assert!(extractions.iter().any(|e|
        e.markdown_path.to_str().unwrap() == "docs/MyTrait.md" &&
        e.content == "A trait"
    ));

    assert!(extractions.iter().any(|e|
        e.markdown_path.to_str().unwrap() == "docs/MyTrait/default_method.md" &&
        e.content == "Default method with body"
    ));
}

#[test]
fn test_extract_const_static_type_alias() {
    let source = r#"
        /// A constant
        const MY_CONST: i32 = 42;

        /// A static
        static MY_STATIC: &str = "hello";

        /// A type alias
        type MyType = Vec<String>;
    "#;

    let (_temp_dir, file_path) = setup_test_file(source);
    let parsed = parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, "docs");

    assert_eq!(extractions.len(), 3);

    assert!(extractions.iter().any(|e|
        e.markdown_path.to_str().unwrap() == "docs/MY_CONST.md" &&
        e.content == "A constant"
    ));

    assert!(extractions.iter().any(|e|
        e.markdown_path.to_str().unwrap() == "docs/MY_STATIC.md" &&
        e.content == "A static"
    ));

    assert!(extractions.iter().any(|e|
        e.markdown_path.to_str().unwrap() == "docs/MyType.md" &&
        e.content == "A type alias"
    ));
}

#[test]
fn test_nested_modules_create_correct_paths() {
    let source = r#"
        pub mod outer {
            pub mod inner {
                /// Deeply nested function
                pub fn deep_func() {}
            }
        }
    "#;

    let (_temp_dir, file_path) = setup_test_file(source);
    let parsed = parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, "docs");

    assert_eq!(extractions.len(), 1);
    assert_eq!(
        extractions[0].markdown_path.to_str().unwrap(),
        "docs/outer/inner/deep_func.md"
    );
    assert_eq!(extractions[0].content, "Deeply nested function");
}

#[test]
fn test_write_extractions_creates_files() {
    let temp_dir = TempDir::new().unwrap();
    let docs_dir = temp_dir.path().join("docs");

    let source = r#"
        /// A function
        fn my_func() {}

        pub mod submod {
            /// Submodule function
            fn sub_func() {}
        }
    "#;

    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, source).unwrap();

    let parsed = parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, docs_dir.to_str().unwrap());

    let report = write_extractions(&extractions, false).unwrap();

    assert_eq!(report.files_written, 2);
    assert_eq!(report.files_skipped, 0);
    assert!(report.errors.is_empty());

    // Verify files exist
    assert!(docs_dir.join("my_func.md").exists());
    assert!(docs_dir.join("submod/sub_func.md").exists());

    // Verify content
    let content = fs::read_to_string(docs_dir.join("my_func.md")).unwrap();
    assert_eq!(content, "A function");

    let sub_content = fs::read_to_string(docs_dir.join("submod/sub_func.md")).unwrap();
    assert_eq!(sub_content, "Submodule function");
}

#[test]
fn test_dry_run_does_not_create_files() {
    let temp_dir = TempDir::new().unwrap();
    let docs_dir = temp_dir.path().join("docs");

    let source = r#"
        /// A function
        fn my_func() {}
    "#;

    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, source).unwrap();

    let parsed = parse_file(&file_path).unwrap();
    let extractions = extract_all_docs(&parsed, docs_dir.to_str().unwrap());

    let report = write_extractions(&extractions, true).unwrap();

    assert_eq!(report.files_written, 1);
    assert!(!docs_dir.exists(), "Dry run should not create directories");
}
