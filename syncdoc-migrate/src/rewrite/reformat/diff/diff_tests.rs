use super::*;

#[test]
fn test_compute_diff_simple() {
    let before = "line1\nline2\nline3\n";
    let after = "line1\nmodified\nline3\n";

    let hunks = compute_line_diff(before, after);
    assert_eq!(hunks.len(), 1);
    assert_eq!(hunks[0].before_start, 1);
    assert_eq!(hunks[0].before_count, 1);
    assert_eq!(hunks[0].after_start, 1);
    assert_eq!(hunks[0].after_count, 1);
}

#[test]
fn test_compute_diff_no_changes() {
    let text = "line1\nline2\nline3\n";
    let hunks = compute_line_diff(text, text);
    assert_eq!(hunks.len(), 0);
}

#[test]
fn test_apply_diff_replacement() {
    let original = "line1\n/// line2\nline3\n";
    let after = "line1\n/// modified\nline3\n";

    let hunks = compute_line_diff(original, after);
    let result = apply_diff(original, &hunks, after);

    assert_eq!(result, "line1\n/// modified\nline3");
}

#[test]
fn test_apply_diff_preserves_unchanged() {
    let original = "  line1  \n/// line2\n  line3  \n";
    let after = "line1\n/// modified\nline3\n";

    let hunks = compute_line_diff(
        "line1\n/// line2\nline3\n", // normalized for diff
        after,
    );
    let result = apply_diff(original, &hunks, after);

    // Should preserve spacing on unchanged lines
    assert!(result.starts_with("  line1  "));
    assert!(result.contains("modified"));
}

#[test]
fn test_apply_diff_multiple_hunks() {
    let before = "a\n/// b\nc\n/// d\ne\n";
    let after = "a\n/// B\nc\n/// D\ne\n";

    let hunks = compute_line_diff(before, after);
    let result = apply_diff(before, &hunks, after);

    assert!(result.contains("B"));
    assert!(result.contains("D"));
    assert!(result.contains("a"));
    assert!(result.contains("c"));
    assert!(result.contains("e"));
}

#[test]
fn test_apply_diff_preserves_regular_comment_lines() {
    let original = "// Regular comment\n/// line2\nline3\n";
    let after = "/// MODIFIED\nline3\n";

    let hunks = vec![DiffHunk {
        before_start: 0,
        before_count: 3,
        after_start: 0,
        after_count: 2,
    }];

    let result = apply_diff(original, &hunks, after);

    // Should preserve regular comment
    assert!(result.contains("// Regular comment"));
    assert!(result.contains("MODIFIED"));
    assert!(result.contains("line3"));
}

#[test]
fn test_apply_diff_does_not_preserve_doc_comments() {
    let original = "/// Doc comment\nline2\n";
    let after = "fn foo() {}\n";

    let hunks = vec![DiffHunk {
        before_start: 0,
        before_count: 2,
        after_start: 0,
        after_count: 1,
    }];

    let result = apply_diff(original, &hunks, after);

    // Doc comments should NOT be preserved (intentionally removed)
    assert!(!result.contains("/// Doc comment"));
    assert!(result.contains("fn foo()"));
}

#[test]
fn test_apply_diff_does_not_preserve_module_doc_comments() {
    let original = "//! Module doc\nfn foo() {}\n";
    let after = "#![doc = syncdoc::module_doc!()]\nfn foo() {}\n";

    let hunks = vec![DiffHunk {
        before_start: 0,
        before_count: 2,
        after_start: 0,
        after_count: 2,
    }];

    let result = apply_diff(original, &hunks, after);

    // Module doc comments should NOT be preserved (replaced with macro)
    assert!(!result.contains("//! Module doc"));
    assert!(result.contains("module_doc!"));
}

#[test]
fn test_apply_diff_preserves_indented_regular_comments() {
    let original = "fn foo() {\n    // TODO: implement this\n    let x = 1;\n}\n";
    let after = "fn foo() {\n    let y = 2;\n}\n";

    let hunks = vec![DiffHunk {
        before_start: 1,
        before_count: 2,
        after_start: 1,
        after_count: 1,
    }];

    let result = apply_diff(original, &hunks, after);

    // Regular comments should be preserved
    assert!(result.contains("// TODO: implement this"));
}

#[test]
fn test_apply_diff_mixed_comments() {
    let original = "// Regular\n/// Doc\n//! Inner doc\n// Another regular\ncode();\n";
    let after = "code();\n";

    let hunks = vec![DiffHunk {
        before_start: 0,
        before_count: 5,
        after_start: 0,
        after_count: 1,
    }];

    let result = apply_diff(original, &hunks, after);

    // Only regular comments preserved
    assert!(result.contains("// Regular"));
    assert!(result.contains("// Another regular"));
    // Doc comments removed
    assert!(!result.contains("/// Doc"));
    assert!(!result.contains("//! Inner doc"));
}

#[test]
fn test_split_module_and_item_doc_hunk() {
    let original = "//! Module doc\n\npub enum Foo {\n    Bar,\n}\n";
    let after =
        "#![doc = syncdoc::module_doc!()]\n#[syncdoc::omnidoc]\npub enum Foo {\n    Bar,\n}\n";

    let hunks = compute_line_diff(original, after);
    let result = apply_diff(original, &hunks, after);

    eprintln!("Result:\n{}", result);

    // Module doc should have blank line after
    let lines: Vec<&str> = result.lines().collect();
    let module_doc_idx = lines.iter().position(|l| l.contains("module_doc")).unwrap();

    // Next line should be blank
    assert_eq!(lines[module_doc_idx + 1].trim(), "");

    // Then omnidoc attribute (no extra blank before it)
    assert!(lines[module_doc_idx + 2].contains("omnidoc"));

    // Then the enum declaration
    assert!(lines[module_doc_idx + 3].contains("pub enum"));
}

#[test]
fn test_split_preserves_original_blank_line() {
    let original = "//! Module\n\nuse foo::Bar;\n";
    let after = "#![doc = syncdoc::module_doc!()]\n\nuse foo::Bar;\n";

    let hunks = compute_line_diff(original, after);
    let result = apply_diff(original, &hunks, after);

    // Should have exactly one blank line after module doc
    let lines: Vec<&str> = result.lines().collect();
    assert!(lines[0].contains("module_doc"));
    assert_eq!(lines[1].trim(), "");
    assert!(lines[2].contains("use"));

    // Should not have double blank lines
    assert_ne!(lines.get(2).map(|l| l.trim()), Some(""));
}

#[test]
fn test_strip_doc_attr_bookends_module() {
    let line = r#"#[doc = "//! Module doc"]"#;
    let result = super::strip_doc_attr_bookends(line);
    assert_eq!(result, "//! Module doc");
}

#[test]
fn test_strip_doc_attr_bookends_item() {
    let line = r#"#[doc = "/// Item doc"]"#;
    let result = super::strip_doc_attr_bookends(line);
    assert_eq!(result, "/// Item doc");
}

#[test]
fn test_strip_doc_attr_bookends_preserves_indent() {
    let line = r#"    #[doc = "//! Indented"]"#;
    let result = super::strip_doc_attr_bookends(line);
    assert_eq!(result, "    //! Indented");
}

#[test]
fn test_strip_doc_attr_bookends_non_doc() {
    let line = r#"#[derive(Debug)]"#;
    let result = super::strip_doc_attr_bookends(line);
    assert_eq!(result, r#"#[derive(Debug)]"#);
}

#[test]
fn test_strip_doc_attr_bookends_empty_line() {
    let line = r#"#[doc = "//! "]"#;
    let result = super::strip_doc_attr_bookends(line);
    assert_eq!(result, "//!");

    let line2 = r#"#[doc = "/// "]"#;
    let result2 = super::strip_doc_attr_bookends(line2);
    assert_eq!(result2, "///");
}
