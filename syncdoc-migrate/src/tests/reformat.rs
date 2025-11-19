use crate::rewrite::reformat::*;

#[test]
fn test_rustfmt_basic() {
    let input = "fn foo(){Ok(())}";
    let result = rustfmt(input);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("fn foo()"));
    assert!(formatted.contains("{\n"));
}

#[test]
fn test_rewrite_preserving_format_simple() {
    let original = r#"//! Hello

/// world
fn foo() {
    Ok(())
}
"#;

    let transformed = r#"#![doc = syncdoc::module_doc!(path = "docs")]
#[syncdoc::omnidoc]
fn foo() { Ok(()) }
"#;

    let result = rewrite_preserving_format(original, transformed);
    assert!(result.is_ok());
    let rewritten = result.unwrap();

    // Should have module_doc
    assert!(rewritten.contains("module_doc!"));

    // Should have omnidoc
    assert!(rewritten.contains("omnidoc"));

    // Should preserve function body formatting if possible
    assert!(rewritten.contains("fn foo()"));
}
