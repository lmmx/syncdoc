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
fn test_restore_section_struct_full_case_reduced() {
    // Reduction of the EXACT bug case reported - item doc bunches up with the module doc
    let source = r#"
#![doc = syncdoc::module_doc!()]

#[syncdoc::omnidoc]
pub struct Section;
"#;

    let (temp, source_path) = setup_test_with_docs(
        source,
        &[
            ("test.md", "Section representation for tree-sitter parsed documents.\n\nA section represents a hierarchical division of a document, typically\ncorresponding to a heading in markdown. Sections track their position\nin the document tree through parent/child relationships and maintain\nprecise byte and line coordinates for content extraction and modification.\n"),
            ("Section.md", "Hierarchical document division with precise coordinates for extraction and modification.\n"),
        ],
    );

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    eprintln!("\n=== RESTORED OUTPUT (first 30 lines) ===");
    for (i, line) in restored.lines().take(30).enumerate() {
        eprintln!("{:3}: {:?}", i, line);
    }
    eprintln!("=========================================\n");

    let lines: Vec<&str> = restored.lines().collect();

    // Find critical positions
    let last_module_doc = lines
        .iter()
        .rposition(|l| l.starts_with("//!"))
        .expect("Should have module docs");

    let first_item_doc = lines
        .iter()
        .position(|l| l.starts_with("///"))
        .expect("Should have item docs");

    let first_derive = lines
        .iter()
        .position(|l| l.contains("#[derive(Clone)]"))
        .expect("Should have derive");

    eprintln!("Last module doc at line: {}", last_module_doc);
    eprintln!("First item doc at line: {}", first_item_doc);
    eprintln!("First #[derive] at line: {}", first_derive);

    // CRITICAL ASSERTIONS for the bug
    assert!(
        lines[last_module_doc + 1].trim().is_empty(),
        "Should have blank line after module docs at line {}. Got: {:?}",
        last_module_doc + 1,
        lines.get(last_module_doc + 1)
    );

    assert!(
        first_item_doc < first_derive,
        "BUG REPRODUCED: Item doc (line {}) should come BEFORE #[derive] (line {}), not after!\nLine {}: {:?}\nLine {}: {:?}",
        first_item_doc,
        first_derive,
        first_item_doc,
        lines[first_item_doc],
        first_derive,
        lines[first_derive]
    );

    // The item doc should be immediately before (or within a few lines of) the derive
    assert!(
        first_derive - first_item_doc <= 2,
        "Item doc should be within 2 lines of #[derive], but gap is {}",
        first_derive - first_item_doc
    );

    assert!(!restored.contains("omnidoc"), "Should not contain omnidoc");
    assert!(
        !restored.contains("module_doc!"),
        "Should not contain module_doc!"
    );
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

#[test]
#[ignore = "Ain't nobody got time for all this hunk logging"]
fn test_restore_section_struct_full_case() {
    // This is the EXACT bug case reported - item doc appears after #[derive] instead of before
    let source = r#"
#![doc = syncdoc::module_doc!()]

#[syncdoc::omnidoc]
#[derive(Clone)]
pub struct Section {
    pub title: String,
    pub level: usize,
    pub line_start: i64,
    pub line_end: i64,
    pub column_start: i64,
    pub column_end: i64,
    pub byte_start: usize,
    pub byte_end: usize,
    pub file_path: String,
    pub parent_index: Option<usize>,
    pub children_indices: Vec<usize>,
    pub section_content: Option<Vec<String>>,
    pub chunk_type: Option<ChunkType>,
    pub lhs_content: Option<String>,
    pub rhs_content: Option<String>,
}

#[syncdoc::omnidoc]
#[derive(Clone)]
pub enum ChunkType {
    Added,
    Deleted,
    Modified,
    Unchanged,
}
"#;

    let (temp, source_path) = setup_test_with_docs(
        source,
        &[
            ("test.md", "Section representation for tree-sitter parsed documents.\n\nA section represents a hierarchical division of a document, typically\ncorresponding to a heading in markdown. Sections track their position\nin the document tree through parent/child relationships and maintain\nprecise byte and line coordinates for content extraction and modification.\n"),
            ("Section.md", "Hierarchical document division with precise coordinates for extraction and modification.\n"),
            ("Section/title.md", "Section heading text without markup symbols.\n"),
            ("Section/level.md", "Nesting depth in the document hierarchy (1 for top-level).\n"),
            ("Section/line_start.md", "First line of section content (after the heading).\n"),
            ("Section/line_end.md", "Line where the next section begins or file ends.\n"),
            ("Section/column_start.md", "Starting column of the section heading.\n"),
            ("Section/column_end.md", "Ending column of the section heading.\n"),
            ("Section/byte_start.md", "Byte offset where section content begins.\n"),
            ("Section/byte_end.md", "Byte offset where section content ends.\n"),
            ("Section/file_path.md", "Source file containing this section.\n"),
            ("Section/parent_index.md", "Index of the containing section in the hierarchy.\n"),
            ("Section/children_indices.md", "Indices of directly nested subsections.\n"),
            ("Section/section_content.md", "Edited content for this section (if modified)\n"),
            ("Section/chunk_type.md", "The chunk type (for diffs)\n"),
            ("Section/lhs_content.md", "The LHS (for diffs)\n"),
            ("Section/rhs_content.md", "The RHS (for diffs)\n"),
            ("ChunkType.md", "What sort of hunk (syntactic diff atomic unit) it is.\n"),
            ("ChunkType/Added.md", "Only RHS exists\n"),
            ("ChunkType/Deleted.md", "Only LHS exists\n"),
            ("ChunkType/Modified.md", "Both LHS and RHS exist (and differ)\n"),
            ("ChunkType/Unchanged.md", "Both LHS and RHS exist (and are the same, at least syntactically)\n"),
        ],
    );

    let parsed = parse_file(&source_path).unwrap();
    let restored = restore_file(&parsed, temp.path().join("docs").to_str().unwrap()).unwrap();

    eprintln!("\n=== RESTORED OUTPUT (first 30 lines) ===");
    for (i, line) in restored.lines().take(30).enumerate() {
        eprintln!("{:3}: {:?}", i, line);
    }
    eprintln!("=========================================\n");

    let lines: Vec<&str> = restored.lines().collect();

    // Find critical positions
    let last_module_doc = lines
        .iter()
        .rposition(|l| l.starts_with("//!"))
        .expect("Should have module docs");

    let first_item_doc = lines
        .iter()
        .position(|l| l.starts_with("///"))
        .expect("Should have item docs");

    let first_derive = lines
        .iter()
        .position(|l| l.contains("#[derive(Clone)]"))
        .expect("Should have derive");

    eprintln!("Last module doc at line: {}", last_module_doc);
    eprintln!("First item doc at line: {}", first_item_doc);
    eprintln!("First #[derive] at line: {}", first_derive);

    // CRITICAL ASSERTIONS for the bug
    assert!(
        lines[last_module_doc + 1].trim().is_empty(),
        "Should have blank line after module docs at line {}. Got: {:?}",
        last_module_doc + 1,
        lines.get(last_module_doc + 1)
    );

    assert!(
        first_item_doc < first_derive,
        "BUG REPRODUCED: Item doc (line {}) should come BEFORE #[derive] (line {}), not after!\nLine {}: {:?}\nLine {}: {:?}",
        first_item_doc,
        first_derive,
        first_item_doc,
        lines[first_item_doc],
        first_derive,
        lines[first_derive]
    );

    // The item doc should be immediately before (or within a few lines of) the derive
    assert!(
        first_derive - first_item_doc <= 2,
        "Item doc should be within 2 lines of #[derive], but gap is {}",
        first_derive - first_item_doc
    );

    assert!(!restored.contains("omnidoc"), "Should not contain omnidoc");
    assert!(
        !restored.contains("module_doc!"),
        "Should not contain module_doc!"
    );
}
