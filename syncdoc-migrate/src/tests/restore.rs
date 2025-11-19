use crate::discover::parse_file;
use crate::restore::*;
use std::fs;
use tempfile::TempDir;

fn setup_test_with_docs(source: &str, docs: &[(&str, &str)]) -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().unwrap();
    let docs_dir = temp.path().join("docs");
    fs::create_dir(&docs_dir).unwrap();

    // Write markdown files
    for (path, content) in docs {
        let full_path = docs_dir.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full_path, content).unwrap();
    }

    // Write source file
    let source_path = temp.path().join("test.rs");
    fs::write(&source_path, source).unwrap();

    (temp, source_path)
}

#[test]
fn test_restore_simple_function() {
    let source = r#"
#[syncdoc::omnidoc]
pub fn my_function() {
    println!("hello");
}
"#;

    let (temp, source_path) =
        setup_test_with_docs(source, &[("my_function.md", "A simple function\n")]);

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    assert!(restored.contains("/// A simple function"));
    assert!(!restored.contains("omnidoc"));
    assert!(restored.contains("pub fn my_function()"));
}

#[test]
fn test_restore_module_level_docs() {
    let source = r#"
#![doc = syncdoc::module_doc!()]

pub fn test() {}
"#;

    let (temp, source_path) =
        setup_test_with_docs(source, &[("test.md", "Module documentation\n")]);

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    assert!(restored.contains("//! Module documentation"));
    assert!(!restored.contains("module_doc!"));
}

#[test]
fn test_restore_struct_with_fields() {
    let source = r#"
#[syncdoc::omnidoc]
pub struct MyStruct {
    pub field_a: i32,
    field_b: String,
}
"#;

    let (temp, source_path) = setup_test_with_docs(
        source,
        &[
            ("MyStruct.md", "A struct\n"),
            ("MyStruct/field_a.md", "Field A\n"),
            ("MyStruct/field_b.md", "Field B\n"),
        ],
    );

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    assert!(restored.contains("/// A struct"));
    assert!(restored.contains("/// Field A"));
    assert!(restored.contains("/// Field B"));
    assert!(!restored.contains("omnidoc"));
}

#[test]
fn test_restore_enum_with_variants() {
    let source = r#"
#[syncdoc::omnidoc]
pub enum MyEnum {
    VariantA,
    VariantB(i32),
}
"#;

    let (temp, source_path) = setup_test_with_docs(
        source,
        &[
            ("MyEnum.md", "An enum\n"),
            ("MyEnum/VariantA.md", "Variant A\n"),
            ("MyEnum/VariantB.md", "Variant B\n"),
        ],
    );

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    assert!(restored.contains("/// An enum"));
    assert!(restored.contains("/// Variant A"));
    assert!(restored.contains("/// Variant B"));
    assert!(!restored.contains("omnidoc"));
}

#[test]
fn test_restore_preserves_non_omnidoc_attrs() {
    let source = r#"
#[derive(Debug)]
#[syncdoc::omnidoc]
#[cfg(test)]
pub struct MyStruct;
"#;

    let (temp, source_path) = setup_test_with_docs(source, &[("MyStruct.md", "A struct\n")]);

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    assert!(restored.contains("#[derive(Debug)]"));
    assert!(restored.contains("#[cfg(test)]"));
    assert!(restored.contains("/// A struct"));
    assert!(!restored.contains("omnidoc"));
}

#[test]
fn test_restore_nested_module() {
    let source = r#"
#[syncdoc::omnidoc]
pub mod outer {
    pub fn inner_fn() {}
}
"#;

    let (temp, source_path) = setup_test_with_docs(
        source,
        &[
            ("outer.md", "Outer module\n"),
            ("outer/inner_fn.md", "Inner function\n"),
        ],
    );

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    assert!(restored.contains("/// Outer module"));
    assert!(restored.contains("/// Inner function"));
    assert!(!restored.contains("omnidoc"));
}

#[test]
fn test_restore_impl_block() {
    let source = r#"
#[syncdoc::omnidoc]
impl MyStruct {
    pub fn new() -> Self {
        Self
    }

    pub fn method(&self) {}
}
"#;

    let (temp, source_path) = setup_test_with_docs(
        source,
        &[
            ("MyStruct/new.md", "Constructor\n"),
            ("MyStruct/method.md", "A method\n"),
        ],
    );

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    assert!(restored.contains("/// Constructor"));
    assert!(restored.contains("/// A method"));
    assert!(!restored.contains("omnidoc"));
}

#[test]
fn test_restore_trait_impl() {
    let source = r#"
#[syncdoc::omnidoc]
impl Display for MyStruct {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "MyStruct")
    }
}
"#;

    let (temp, source_path) = setup_test_with_docs(
        source,
        &[("MyStruct/Display/fmt.md", "Format implementation\n")],
    );

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    assert!(restored.contains("/// Format implementation"));
    assert!(!restored.contains("omnidoc"));
}

#[test]
fn test_restore_trait_definition() {
    let source = r#"
#[syncdoc::omnidoc]
pub trait MyTrait {
    fn required(&self);

    fn default_method(&self) {
        println!("default");
    }
}
"#;

    let (temp, source_path) = setup_test_with_docs(
        source,
        &[
            ("MyTrait.md", "A trait\n"),
            ("MyTrait/required.md", "Required method\n"),
            ("MyTrait/default_method.md", "Default method\n"),
        ],
    );

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    assert!(restored.contains("/// A trait"));
    assert!(restored.contains("/// Required method"));
    assert!(restored.contains("/// Default method"));
    assert!(!restored.contains("omnidoc"));
}

#[test]
fn test_restore_handles_missing_markdown() {
    let source = r#"
#[syncdoc::omnidoc]
pub fn documented() {}

#[syncdoc::omnidoc]
pub fn undocumented() {}
"#;

    let (temp, source_path) = setup_test_with_docs(source, &[("documented.md", "Has docs\n")]);

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    // Should have docs for documented function
    assert!(restored.contains("/// Has docs"));

    // Should not have docs for undocumented (file missing)
    // But function should still be present
    assert!(restored.contains("pub fn undocumented()"));

    // No omnidoc attributes should remain
    assert!(!restored.contains("omnidoc"));
}

#[test]
fn test_restore_multiline_docs() {
    let source = r#"
#[syncdoc::omnidoc]
pub fn test() {}
"#;

    let (temp, source_path) =
        setup_test_with_docs(source, &[("test.md", "Line 1\nLine 2\nLine 3\n")]);

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    assert!(restored.contains("/// Line 1"));
    assert!(restored.contains("/// Line 2"));
    assert!(restored.contains("/// Line 3"));
}

#[test]
fn test_restore_enum_struct_variant() {
    let source = r#"
#[syncdoc::omnidoc]
pub enum MyEnum {
    StructVariant {
        field_a: i32,
        field_b: String,
    },
}
"#;

    let (temp, source_path) = setup_test_with_docs(
        source,
        &[
            ("MyEnum.md", "An enum\n"),
            ("MyEnum/StructVariant.md", "A struct variant\n"),
            ("MyEnum/StructVariant/field_a.md", "Field A\n"),
            ("MyEnum/StructVariant/field_b.md", "Field B\n"),
        ],
    );

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    assert!(restored.contains("/// An enum"));
    assert!(restored.contains("/// A struct variant"));
    assert!(restored.contains("/// Field A"));
    assert!(restored.contains("/// Field B"));
    assert!(!restored.contains("omnidoc"));
}
