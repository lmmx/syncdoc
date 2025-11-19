use crate::rewrite::reformat::bookend::*;

#[test]
fn test_needs_bookending_detects_triggers() {
    assert!(needs_bookending(
        "#![ doc = syncdoc :: module_doc ! ( path = \"docs\" ) ]"
    ));
    assert!(needs_bookending("#![doc = module_doc!(path = \"docs\")]"));
    assert!(!needs_bookending("#[syncdoc::omnidoc]"));
    assert!(!needs_bookending("//! Not an attribute"));
}

#[test]
fn test_extract_bookend_content() {
    let line = "#![ doc = syncdoc :: module_doc ! ( path = \"docs\" ) ]";
    let content = extract_bookend_content(line);
    assert_eq!(
        content,
        Some("doc=syncdoc::module_doc!(path=\"docs\")".to_string())
    );
}

#[test]
fn test_create_bookended_expr() {
    let content = "doc = syncdoc::module_doc!(path = \"docs\")";
    let expr = create_bookended_expr(content);
    assert_eq!(
        expr,
        "const _: i32 = { doc = syncdoc::module_doc!(path = \"docs\") };"
    );
}

#[test]
fn test_strip_bookends() {
    let formatted = "const _: i32 = { doc = syncdoc::module_doc!(path = \"docs\") };";
    let stripped = strip_bookends(formatted);
    assert_eq!(
        stripped,
        Some("doc = syncdoc::module_doc!(path = \"docs\")".to_string())
    );
}

#[test]
fn test_reconstruct_inner_attr() {
    let content = "doc = syncdoc::module_doc!(path = \"docs\")";
    let attr = reconstruct_inner_attr(content);
    assert_eq!(attr, "#![doc = syncdoc::module_doc!(path = \"docs\")]");
}

#[test]
fn test_bookend_roundtrip() {
    let line = "#![ doc = syncdoc :: module_doc ! ( path = \"docs\" ) ]";
    let reformatted = reformat_line(line);
    assert!(reformatted.is_some());
    let result = reformatted.unwrap();
    assert!(result.contains("module_doc!"));
    assert!(!result.contains(" :: "));
}

#[test]
fn test_reformat_bookended_lines_preserves_others() {
    let code = r#"#![ doc = syncdoc :: module_doc ! ( path = "docs" ) ]

#[syncdoc::omnidoc]
fn foo() {
    Ok(())
}
"#;

    let result = reformat_bookended_lines(code);
    assert!(result.contains("module_doc!"));
    assert!(result.contains("#[syncdoc::omnidoc]"));
    assert!(result.contains("fn foo()"));
}
