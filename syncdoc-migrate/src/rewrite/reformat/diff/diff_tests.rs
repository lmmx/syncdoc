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
    let original = "line1\nline2\nline3\n";
    let after = "line1\nmodified\nline3\n";

    let hunks = compute_line_diff(original, after);
    let result = apply_diff(original, &hunks, after);

    assert_eq!(result, "line1\nmodified\nline3");
}

#[test]
fn test_apply_diff_preserves_unchanged() {
    let original = "  line1  \nline2\n  line3  \n";
    let after = "line1\nmodified\nline3\n";

    let hunks = compute_line_diff(
        "line1\nline2\nline3\n", // normalized for diff
        after,
    );
    let result = apply_diff(original, &hunks, after);

    // Should preserve spacing on unchanged lines
    assert!(result.starts_with("  line1  "));
    assert!(result.contains("modified"));
}

#[test]
fn test_apply_diff_multiple_hunks() {
    let before = "a\nb\nc\nd\ne\n";
    let after = "a\nB\nc\nD\ne\n";

    let hunks = compute_line_diff(before, after);
    let result = apply_diff(before, &hunks, after);

    assert!(result.contains("B"));
    assert!(result.contains("D"));
    assert!(result.contains("a"));
    assert!(result.contains("c"));
    assert!(result.contains("e"));
}
