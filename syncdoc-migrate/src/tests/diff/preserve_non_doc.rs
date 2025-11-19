// syncdoc-migrate/src/tests/diff/preserve_non_doc.rs

use super::helpers::*;
use crate::rewrite::reformat::diff::apply::apply_diff;
use insta::assert_snapshot;

#[test]
fn preserves_derive_attrs() {
    let original = r#"#[derive(Debug)]
/// Documentation
pub struct Test;"#;

    let transformed = r#"#[syncdoc::omnidoc]
pub struct Test;"#;

    let hunks = compute_hunks(original, transformed);
    let result = apply_diff(original, &hunks, transformed);

    assert!(
        result.contains("#[derive(Debug)]"),
        "Should preserve derive"
    );
    assert_snapshot!(result);
}

#[test]
fn preserves_cfg_attrs() {
    let original = r#"#[cfg(test)]
/// Documentation
pub fn test() {}"#;

    let transformed = r#"#[syncdoc::omnidoc]
pub fn test() {}"#;

    let hunks = compute_hunks(original, transformed);
    let result = apply_diff(original, &hunks, transformed);

    assert!(result.contains("#[cfg(test)]"), "Should preserve cfg");
}

#[test]
fn preserves_multiple_non_doc_attrs() {
    let original = r#"#[derive(Debug, Clone)]
#[cfg(feature = "test")]
/// Documentation
pub struct Test;"#;

    let transformed = r#"#[syncdoc::omnidoc]
pub struct Test;"#;

    let hunks = compute_hunks(original, transformed);
    let result = apply_diff(original, &hunks, transformed);

    assert!(result.contains("derive"), "Should preserve derive");
    assert!(result.contains("cfg"), "Should preserve cfg");
    assert_snapshot!(result);
}

#[test]
fn preserves_regular_comments() {
    let original = r#"// Regular comment
/// Doc comment
pub fn test() {}"#;

    let transformed = r#"#[syncdoc::omnidoc]
pub fn test() {}"#;

    let hunks = compute_hunks(original, transformed);
    let result = apply_diff(original, &hunks, transformed);

    assert!(
        result.contains("// Regular comment"),
        "Should preserve regular comments"
    );
}

#[test]
fn does_not_preserve_doc_comments() {
    let original = r#"/// Doc comment
//! Inner doc
pub fn test() {}"#;

    let transformed = r#"#[syncdoc::omnidoc]
pub fn test() {}"#;

    let hunks = compute_hunks(original, transformed);
    let result = apply_diff(original, &hunks, transformed);

    assert!(
        !result.contains("/// Doc comment"),
        "Should not preserve doc comments"
    );
    assert!(
        !result.contains("//! Inner doc"),
        "Should not preserve inner docs"
    );
}

#[test]
fn snapshot_attribute_preservation() {
    let original = r#"#[derive(Clone)]
/// Struct doc
pub struct Section {
    /// Field doc
    pub title: String,
}

#[derive(Clone)]
/// Enum doc
pub enum ChunkType {
    /// Variant A
    Added,
}"#;

    let transformed = r#"#[syncdoc::omnidoc]
pub struct Section {
    pub title: String,
}
#[syncdoc::omnidoc]
pub enum ChunkType {
    Added,
}"#;

    let hunks = compute_hunks(original, transformed);
    let result = apply_diff(original, &hunks, transformed);

    assert_snapshot!(result);
}
