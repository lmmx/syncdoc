use super::*;
use crate::formats::markdown::MarkdownFormat;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

fn print_tree(cursor: &mut tree_sitter::TreeCursor, source: &[u8], depth: usize) {
    let node = cursor.node();
    let indent = "  ".repeat(depth);

    if node.child_count() == 0 {
        let text = &source[node.byte_range()];
        println!(
            "{}{}[{}]: {:?}",
            indent,
            node.kind(),
            node.id(),
            String::from_utf8_lossy(text).trim()
        );
    } else {
        println!("{}{}[{}]", indent, node.kind(), node.id());
    }

    if cursor.goto_first_child() {
        loop {
            print_tree(cursor, source, depth + 1);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

#[test]
fn test_find_documents_single_file() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path().with_extension("md");
    fs::rename(file.path(), &path).unwrap();

    let results = find_documents(vec![path.clone()], &["md".to_string()]).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], path);

    fs::remove_file(path).unwrap();
}

#[test]
fn test_extract_simple_sections() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# Hello\n\n?\n\n## World\n\n??\n\n### Hmm\n\n???").unwrap();
    let path = file.path();

    let format = MarkdownFormat;
    let sections = extract_sections(path, &format).unwrap();

    assert!(!sections.is_empty(), "Should find sections in markdown");
    assert_eq!(sections.len(), 3, "Should find 3 headings");

    assert_eq!(sections[0].title, "Hello");
    assert_eq!(sections[0].level, 1);

    assert_eq!(sections[1].title, "World");
    assert_eq!(sections[1].level, 2);

    assert_eq!(sections[2].title, "Hmm");
    assert_eq!(sections[2].level, 3);
}

#[test]
fn test_section_hierarchy() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        "# Parent\n\nContent\n\n## Child1\n\nMore\n\n## Child2\n\nEven more"
    )
    .unwrap();
    let path = file.path();

    let format = MarkdownFormat;
    let sections = extract_sections(path, &format).unwrap();

    assert_eq!(sections.len(), 3);

    // Parent has no parent
    assert_eq!(sections[0].parent_index, None);
    assert_eq!(sections[0].children_indices.len(), 2);

    // Both children have parent
    assert_eq!(sections[1].parent_index, Some(0));
    assert_eq!(sections[2].parent_index, Some(0));
}

#[test]
fn test_tree_sitter_parsing() {
    use tree_sitter::Parser;

    let markdown = "# Hello\n\nContent here\n\n## World\n\nMore content";

    let mut parser = Parser::new();
    let format = MarkdownFormat;
    parser.set_language(&format.language()).unwrap();

    let tree = parser.parse(markdown, None).unwrap();
    let root = tree.root_node();

    println!("Root node kind: {}", root.kind());
    println!("Root node S-expression:\n{}", root.to_sexp());

    // This will help us see what the actual node structure is
    let mut cursor = root.walk();
    print_tree(&mut cursor, markdown.as_bytes(), 0);
}
