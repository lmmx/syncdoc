// syncdoc-migrate/src/tests/diff/module_level.rs

use super::helpers::*;
use crate::rewrite::reformat::diff::apply::is_module_level_hunk;
use insta::assert_snapshot;

#[test]
fn identifies_module_doc_macro() {
    let after_lines = lines(
        r#"#![doc = syncdoc::module_doc!()]
pub fn test() {}"#,
    );

    let hunk = hunk(0, 1, 0, 1);

    assert!(
        is_module_level_hunk(&hunk, &after_lines),
        "Should identify module_doc macro"
    );
}

#[test]
fn identifies_inner_doc_attr() {
    let after_lines = lines(
        r#"#![doc = "//! Module doc"]
pub fn test() {}"#,
    );

    let hunk = hunk(0, 1, 0, 1);

    assert!(
        is_module_level_hunk(&hunk, &after_lines),
        "Should identify inner doc attribute"
    );
}

#[test]
fn rejects_outer_attributes() {
    let after_lines = lines(
        r#"#[syncdoc::omnidoc]
pub fn test() {}"#,
    );

    let hunk = hunk(0, 1, 0, 1);

    assert!(
        !is_module_level_hunk(&hunk, &after_lines),
        "Should reject outer attributes"
    );
}

#[test]
fn rejects_regular_code() {
    let after_lines = lines(
        r#"pub fn test() {
    println!("hello");
}"#,
    );

    let hunk = hunk(0, 3, 0, 3);

    assert!(
        !is_module_level_hunk(&hunk, &after_lines),
        "Should reject regular code"
    );
}

#[test]
fn handles_mixed_inner_outer() {
    let after_lines = lines(
        r#"#![doc = syncdoc::module_doc!()]
#[syncdoc::omnidoc]
pub struct Test;"#,
    );

    // Hunk spanning both
    let hunk = hunk(0, 3, 0, 3);

    assert!(
        is_module_level_hunk(&hunk, &after_lines),
        "Should identify mixed hunk as module-level if it contains module docs"
    );
}

#[test]
fn snapshot_module_level_detection() {
    let test_cases = vec![
        ("module_doc_macro", r#"#![doc = syncdoc::module_doc!()]"#),
        ("inner_doc", r#"#![doc = "//! Doc"]"#),
        ("outer_omnidoc", r#"#[syncdoc::omnidoc]"#),
        ("outer_doc", r#"#[doc = "/// Doc"]"#),
        ("regular_code", r#"pub fn test() {}"#),
        (
            "mixed",
            r#"#![doc = syncdoc::module_doc!()]
#[syncdoc::omnidoc]"#,
        ),
    ];

    let results: Vec<_> = test_cases
        .iter()
        .map(|(name, code)| {
            let after_lines = lines(code);
            let hunk = hunk(0, after_lines.len(), 0, after_lines.len());
            let is_module = is_module_level_hunk(&hunk, &after_lines);
            format!("{}: {}", name, if is_module { "MODULE" } else { "ITEM" })
        })
        .collect();

    assert_snapshot!(results.join("\n"));
}
