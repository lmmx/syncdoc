use crate::formats::difftastic::parse_difftastic_json;

#[test]
fn test_parse_single_file_diff() {
    let json = r##"[{"chunks":[[{"lhs":{"line_number":0,"changes":[{"start":0,"end":1,"content":"#","highlight":"normal"},{"start":1,"end":2,"content":" ","highlight":"normal"},{"start":2,"end":3,"content":"1","highlight":"normal"}]},"rhs":{"line_number":0,"changes":[{"start":0,"end":1,"content":"#","highlight":"normal"},{"start":1,"end":2,"content":" ","highlight":"normal"},{"start":2,"end":3,"content":"I","highlight":"normal"}]}}]],"language":"Text","path":"test.md","status":"changed"}]"##;

    let sections = parse_difftastic_json(json).unwrap();

    assert!(!sections.is_empty(), "Should parse sections");

    // Should have 1 hunk section
    assert_eq!(sections.len(), 1, "Should have 1 hunk");
    assert_eq!(sections[0].level, 1, "Hunk should be level 1");
    assert!(sections[0].title.contains("Hunk"), "Should be a hunk");
    assert_eq!(sections[0].file_path, "test.md", "Should track file path");

    // Hunks are top-level, no parent
    assert_eq!(sections[0].parent_index, None, "Hunks have no parent");
}

#[test]
fn test_parse_multi_hunk_file() {
    // Your actual test.md example with multiple changes
    let json = r##"{"chunks":[[{"lhs":{"line_number":0,"changes":[{"start":0,"end":1,"content":"#","highlight":"normal"},{"start":1,"end":2,"content":" ","highlight":"normal"},{"start":2,"end":3,"content":"1","highlight":"normal"}]},"rhs":{"line_number":0,"changes":[{"start":0,"end":1,"content":"#","highlight":"normal"},{"start":1,"end":2,"content":" ","highlight":"normal"},{"start":2,"end":3,"content":"I","highlight":"normal"}]}},{"lhs":{"line_number":2,"changes":[{"start":0,"end":1,"content":"?","highlight":"normal"}]},"rhs":{"line_number":2,"changes":[{"start":0,"end":1,"content":"!","highlight":"normal"}]}}],[{"lhs":{"line_number":4,"changes":[{"start":0,"end":1,"content":"#","highlight":"normal"},{"start":1,"end":2,"content":"#","highlight":"normal"}]},"rhs":{"line_number":4,"changes":[{"start":0,"end":1,"content":"#","highlight":"normal"},{"start":1,"end":2,"content":"#","highlight":"normal"}]}}]],"language":"Text","path":"test.md","status":"changed"}"##;

    let sections = parse_difftastic_json(json).unwrap();

    // Should have 2 hunks (2 chunk arrays)
    assert_eq!(sections.len(), 2, "Should have 2 hunks");

    assert_eq!(sections[0].level, 1, "First hunk is level 1");
    assert_eq!(sections[1].level, 1, "Second hunk is level 1");

    assert!(sections[0].title.contains("Hunk 1"));
    assert!(sections[1].title.contains("Hunk 2"));

    // Both hunks from same file
    assert_eq!(sections[0].file_path, "test.md");
    assert_eq!(sections[1].file_path, "test.md");
}

#[test]
fn test_parse_ndjson_multi_file() {
    // Your actual git diff output with foo.py and test.md
    let json = r##"{"language":"Python","path":"foo.py","status":"created"}
{"chunks":[[{"lhs":{"line_number":0,"changes":[{"start":0,"end":1,"content":"#","highlight":"normal"}]},"rhs":{"line_number":0,"changes":[{"start":0,"end":1,"content":"#","highlight":"normal"}]}}]],"language":"Text","path":"test.md","status":"changed"}"##;

    let sections = parse_difftastic_json(json).unwrap();

    assert!(!sections.is_empty(), "Should parse NDJSON format");

    // Should have: 1 placeholder for foo.py + 1 hunk for test.md = 2 sections
    assert_eq!(sections.len(), 2, "Should have 2 sections total");

    // Check foo.py placeholder (created, no chunks)
    assert!(sections[0].title.contains("created") || sections[0].title.contains("File"));
    assert_eq!(sections[0].file_path, "foo.py");

    // Check test.md hunk
    assert!(sections[1].title.contains("Hunk"));
    assert_eq!(sections[1].file_path, "test.md");
}

#[test]
fn test_hunk_structure() {
    let json = r#"[{"chunks":[[{"lhs":{"line_number":0,"changes":[{"start":0,"end":1,"content":"a","highlight":"normal"}]},"rhs":{"line_number":0,"changes":[{"start":0,"end":1,"content":"b","highlight":"normal"}]}}],[{"lhs":{"line_number":5,"changes":[{"start":0,"end":1,"content":"c","highlight":"normal"}]},"rhs":{"line_number":5,"changes":[{"start":0,"end":1,"content":"d","highlight":"normal"}]}}]],"language":"Text","path":"multi.md","status":"changed"}]"#;

    let sections = parse_difftastic_json(json).unwrap();

    // Should have 2 hunks (2 chunk arrays)
    assert_eq!(sections.len(), 2, "Should have 2 hunks");

    // Both are top-level sections with same file path
    assert_eq!(sections[0].level, 1);
    assert_eq!(sections[1].level, 1);
    assert_eq!(sections[0].file_path, "multi.md");
    assert_eq!(sections[1].file_path, "multi.md");

    // No parent-child relationships between hunks
    assert_eq!(sections[0].parent_index, None);
    assert_eq!(sections[1].parent_index, None);
    assert!(sections[0].children_indices.is_empty());
    assert!(sections[1].children_indices.is_empty());
}

#[test]
fn test_unchanged_file_skipped() {
    let json = r#"[{"language":"Text","path":"unchanged.md","status":"unchanged"}]"#;
    let sections = parse_difftastic_json(json).unwrap();
    assert!(sections.is_empty(), "Should skip unchanged files");
}

#[test]
fn test_created_file_no_chunks() {
    let json = r#"[{"language":"Python","path":"new.py","status":"created"}]"#;
    let sections = parse_difftastic_json(json).unwrap();

    // Should have 1 placeholder hunk for the created file
    assert_eq!(sections.len(), 1, "Should have placeholder hunk");
    assert!(sections[0].title.contains("created") || sections[0].title.contains("File"));
    assert_eq!(sections[0].file_path, "new.py");
}

#[test]
fn test_deleted_file_no_chunks() {
    let json = r#"[{"language":"Python","path":"old.py","status":"deleted"}]"#;
    let sections = parse_difftastic_json(json).unwrap();

    assert_eq!(sections.len(), 1, "Should have placeholder hunk");
    assert!(sections[0].title.contains("deleted") || sections[0].title.contains("File"));
    assert_eq!(sections[0].file_path, "old.py");
}

#[test]
fn test_file_path_tracking() {
    let json = r#"{"chunks":[[{"lhs":{"line_number":0,"changes":[{"start":0,"end":1,"content":"a","highlight":"normal"}]},"rhs":{"line_number":0,"changes":[{"start":0,"end":1,"content":"b","highlight":"normal"}]}}]],"language":"Rust","path":"src/main.rs","status":"changed"}"#;

    let sections = parse_difftastic_json(json).unwrap();

    assert_eq!(sections.len(), 1);
    assert_eq!(
        sections[0].file_path, "src/main.rs",
        "Should preserve full file path"
    );
}
