// syncdoc-migrate/src/tests/diff/skip_transformed.rs

use crate::rewrite::reformat::diff::apply::should_skip_from_transformed;
use insta::assert_snapshot;

#[test]
fn skips_non_doc_outer_attrs() {
    assert!(should_skip_from_transformed("#[derive(Debug)]"));
    assert!(should_skip_from_transformed("#[cfg(test)]"));
    assert!(should_skip_from_transformed("# [derive (Debug)]")); // rustfmt'd
}

#[test]
fn skips_non_doc_inner_attrs() {
    assert!(should_skip_from_transformed("#![allow(dead_code)]"));
    assert!(should_skip_from_transformed("# ! [allow (dead_code)]")); // rustfmt'd
}

#[test]
fn skips_regular_comments() {
    assert!(should_skip_from_transformed("// Regular comment"));
    assert!(should_skip_from_transformed("    // Indented comment"));
}

#[test]
fn does_not_skip_doc_attrs() {
    assert!(!should_skip_from_transformed(
        "#[doc = \"/// Documentation\"]"
    ));
    assert!(!should_skip_from_transformed(
        "#![doc = \"//! Module doc\"]"
    ));
}

#[test]
fn does_not_skip_omnidoc() {
    assert!(!should_skip_from_transformed("#[syncdoc::omnidoc]"));
    assert!(!should_skip_from_transformed("#[omnidoc]"));
}

#[test]
fn does_not_skip_doc_comments() {
    assert!(!should_skip_from_transformed("/// Documentation"));
    assert!(!should_skip_from_transformed("//! Module doc"));
}

#[test]
fn does_not_skip_regular_code() {
    assert!(!should_skip_from_transformed("pub fn test() {}"));
    assert!(!should_skip_from_transformed("struct Foo;"));
}

#[test]
fn snapshot_skip_decisions() {
    let test_lines = vec![
        "#[derive(Debug)]",
        "#[cfg(test)]",
        "# [derive (Clone)]", // rustfmt'd spacing
        "#[doc = \"/// Doc\"]",
        "#[syncdoc::omnidoc]",
        "#![allow(dead_code)]",
        "# ! [cfg (test)]", // rustfmt'd inner
        "#![doc = \"//! Module\"]",
        "// Regular comment",
        "/// Doc comment",
        "//! Inner doc",
        "pub fn test() {}",
    ];

    let results: Vec<_> = test_lines
        .iter()
        .map(|line| {
            let skip = should_skip_from_transformed(line);
            format!("{:40} -> {}", line, if skip { "SKIP" } else { "KEEP" })
        })
        .collect();

    assert_snapshot!(results.join("\n"));
}
