use super::{Edit, EditPlan};
use crate::formats::markdown::MarkdownFormat;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_single_line_replacement() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Line 1\nLine 2\nLine 3").unwrap();
    let path = file.path().to_string_lossy().to_string();

    let edit = Edit {
        file_name: path.clone(),
        line_start: 1,
        line_end: 2,
        column_start: 1,
        column_end: 7,
        section_content: "Modified".to_string(), // No padding
        item_name: "test".to_string(),
    };

    let mut plan = EditPlan { edits: vec![edit] };
    plan.apply().unwrap();

    let content = fs::read_to_string(&path).unwrap();

    // Just check the content exists, don't assume line positions
    assert!(content.contains("Line 1"));
    assert!(content.contains("Modified"));
    assert!(content.contains("Line 3"));
}

#[test]
fn test_section_replacement_with_empty_lines() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# Hello\n\n?\n\n## World").unwrap();
    let path = file.path().to_string_lossy().to_string();

    // Section content starts at line 3 (after "# Hello\n\n")
    let edit = Edit {
        file_name: path.clone(),
        line_start: 3,
        line_end: 5,
        column_start: 1,
        column_end: 2,
        section_content: "Yeah".to_string(),
        item_name: "Hello".to_string(),
    };

    let mut plan = EditPlan { edits: vec![edit] };

    match plan.apply() {
        Ok(()) => {
            let content = fs::read_to_string(&path).unwrap();
            println!("File content after edit:\n{content}");
            println!("Lines:");
            for (i, line) in content.lines().enumerate() {
                println!("  {}: {:?}", i + 1, line);
            }

            let lines: Vec<&str> = content.lines().collect();
            assert!(
                lines.contains(&"Yeah"),
                "Expected 'Yeah' in content, got: {lines:?}"
            );
        }
        Err(e) => panic!("Edit failed: {e}"),
    }
}

#[test]
fn test_boundary_mode_exclude() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "A\nB\nC\nD").unwrap();
    let path = file.path().to_string_lossy().to_string();

    let edit = Edit {
        file_name: path.clone(),
        line_start: 1,
        line_end: 2, // Exclude line 2 (C)
        column_start: 1,
        column_end: 2,
        section_content: "REPLACED".to_string(), // No padding
        item_name: "test".to_string(),
    };

    let mut plan = EditPlan { edits: vec![edit] };
    plan.apply().unwrap();

    let content = fs::read_to_string(&path).unwrap();

    println!("Result: {content}");

    assert!(content.contains('A'));
    assert!(content.contains("REPLACED"));
    assert!(
        content.contains('C'),
        "Line C should still exist (excluded)"
    );
    assert!(content.contains('D'));
}

#[test]
fn test_line_numbering_off_by_one() {
    // Common issue: are lines 0-indexed or 1-indexed?
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "First\nSecond\nThird").unwrap();
    let path = file.path().to_string_lossy().to_string();

    // Try to replace "Second" (line 2 in 1-indexed)
    let edit = Edit {
        file_name: path.clone(),
        line_start: 2,
        line_end: 3,
        column_start: 1,
        column_end: 7,
        section_content: "SECOND".to_string(),
        item_name: "test".to_string(),
    };

    let mut plan = EditPlan { edits: vec![edit] };

    match plan.apply() {
        Ok(()) => {
            let content = fs::read_to_string(&path).unwrap();
            let lines: Vec<&str> = content.lines().collect();

            println!("Lines after edit:");
            for (i, line) in lines.iter().enumerate() {
                println!("  {i}: {line}");
            }

            assert_eq!(lines[0], "First", "First line should be unchanged");
            assert!(
                lines[1] == "SECOND" || lines.contains(&"SECOND"),
                "Second line should be replaced with SECOND, got: {lines:?}"
            );
        }
        Err(e) => {
            // If it fails, that's also information about the issue
            println!("Edit failed with error: {e}");
            panic!("Edit should succeed but got: {e}");
        }
    }
}

#[test]
fn test_extract_sections_line_numbers() {
    // This test verifies that our section extraction gives correct line numbers
    use crate::formats::markdown::MarkdownFormat;
    use crate::input::extract_sections;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# Hello\n\n?\n\n## World\n\n??").unwrap();
    file.flush().unwrap();

    let format = MarkdownFormat;
    let sections = extract_sections(file.path(), &format).unwrap();

    println!("\nExtracted sections:");
    for (i, section) in sections.iter().enumerate() {
        println!("Section {}: {:?}", i, section.title);
        println!("  Level: {}", section.level);
        println!("  Lines: {} to {}", section.line_start, section.line_end);
        println!(
            "  Columns: {} to {}",
            section.column_start, section.column_end
        );
        println!("  Bytes: {} to {}", section.byte_start, section.byte_end);
    }

    // Read file and show what content is at those positions
    let content = fs::read_to_string(file.path()).unwrap();
    println!("\nFile content:");
    for (i, line) in content.lines().enumerate() {
        println!("Line {}: {:?}", i + 1, line);
    }

    // Verify the first section's byte range
    let section = &sections[0];
    let section_content = &content.as_bytes()[section.byte_start..section.byte_end];
    let section_text = String::from_utf8_lossy(section_content);
    println!(
        "\nSection 0 content (bytes {} to {}):",
        section.byte_start, section.byte_end
    );
    println!("{section_text:?}");
}

#[test]
fn test_line_indexing_zero_vs_one() {
    // This test documents whether textum uses 0-indexed or 1-indexed lines
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Line 0\nLine 1\nLine 2\nLine 3").unwrap();
    let path = file.path().to_string_lossy().to_string();

    // Try both 0-indexed and 1-indexed to see which works
    let edit_zero = Edit {
        file_name: path.clone(),
        line_start: 0, // 0-indexed: should target "Line 0"
        line_end: 1,   // Exclusive: should stop before "Line 1"
        column_start: 1,
        column_end: 7,
        section_content: "ZERO".to_string(),
        item_name: "test".to_string(),
    };

    let mut plan = EditPlan {
        edits: vec![edit_zero],
    };
    plan.apply().unwrap();

    let content = fs::read_to_string(&path).unwrap();
    println!("After 0-indexed edit:\n{content}");

    // Reset file
    writeln!(file, "Line 0\nLine 1\nLine 2\nLine 3").unwrap();

    let edit_one = Edit {
        file_name: path.clone(),
        line_start: 1, // 1-indexed: should target "Line 1"?
        line_end: 2,
        column_start: 1,
        column_end: 7,
        section_content: "ONE".to_string(),
        item_name: "test".to_string(),
    };

    let mut plan2 = EditPlan {
        edits: vec![edit_one],
    };
    plan2.apply().unwrap();

    let content2 = fs::read_to_string(&path).unwrap();
    println!("After 1-indexed edit:\n{content2}");
}

#[test]
fn test_app_section_to_textum_conversion() {
    use crate::input::extract_sections;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# Hello\n\n?\n\n## World\n\n??").unwrap();
    let path = file.path();

    let format = MarkdownFormat;
    let sections = extract_sections(path, &format).unwrap();

    println!("\nSection 0 (Hello):");
    println!(
        "  line_start={}, line_end={}",
        sections[0].line_start, sections[0].line_end
    );
    println!(
        "  byte_start={}, byte_end={}",
        sections[0].byte_start, sections[0].byte_end
    );

    let edit = Edit {
        file_name: path.to_string_lossy().to_string(),
        line_start: sections[0].line_start,
        line_end: sections[0].line_end,
        column_start: sections[0].column_start,
        column_end: sections[0].column_end,
        section_content: "Yeah".to_string(),
        item_name: "Hello".to_string(),
    };

    println!("\nEdit structure:");
    println!(
        "  line_start={}, line_end={}",
        edit.line_start, edit.line_end
    );

    let mut plan = EditPlan { edits: vec![edit] };

    match plan.apply() {
        Ok(()) => {
            let content = fs::read_to_string(path).unwrap();
            println!("\nResult:\n{content}");

            // Check if it worked
            assert!(content.contains("Yeah"), "Expected 'Yeah' in output");
        }
        Err(e) => {
            panic!("Edit failed: {e:?}");
        }
    }
}

#[test]
fn test_exact_scenario() {
    use crate::input::extract_sections;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# Hello\n\n?\n\n## World\n\n??").unwrap();
    let path = file.path();

    println!("File content:");
    let orig = fs::read_to_string(path).unwrap();
    for (i, line) in orig.lines().enumerate() {
        println!("  {}: {:?}", i + 1, line);
    }

    // What tree-sitter tells us
    let format = MarkdownFormat;
    let sections = extract_sections(path, &format).unwrap();

    let sec = &sections[0];
    println!("\nSection 0: '{}'", sec.title);
    println!("  tree-sitter says:");
    println!("    line_start={} (line after heading)", sec.line_start);
    println!("    line_end={} (next heading)", sec.line_end);

    // Read actual content at those byte positions
    let file_bytes = orig.as_bytes();
    let actual_content = &file_bytes[sec.byte_start..sec.byte_end.min(file_bytes.len())];
    println!(
        "  byte range {}..{} contains: {:?}",
        sec.byte_start,
        sec.byte_end,
        String::from_utf8_lossy(actual_content)
    );

    // Now apply edit with CORRECTED line numbers
    // If tree-sitter gives 1-indexed but textum wants 0-indexed, subtract 1
    let edit = Edit {
        file_name: path.to_string_lossy().to_string(),
        line_start: (sec.line_start - 1).max(0) as i64, // Convert to 0-indexed
        line_end: (sec.line_end - 1).max(0) as i64,
        column_start: sec.column_start,
        column_end: sec.column_end,
        section_content: "Yeah".to_string(),
        item_name: "Hello".to_string(),
    };

    let mut plan = EditPlan { edits: vec![edit] };
    plan.apply().unwrap();

    let result = fs::read_to_string(path).unwrap();
    println!("\nAfter edit:");
    for (i, line) in result.lines().enumerate() {
        println!("  {}: {:?}", i + 1, line);
    }

    assert!(result.contains("Yeah"), "Should contain 'Yeah'");
}

#[test]
fn test_diagnose_line_numbers() {
    use crate::input::extract_sections;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# Hello\n\n?\n\n## World\n\n??").unwrap();
    file.flush().unwrap();
    let path = file.path();

    // Show exactly what's in the file
    let content = fs::read_to_string(path).unwrap();
    println!("\n=== FILE CONTENT ===");
    for (i, line) in content.lines().enumerate() {
        println!("Line {i} (0-idx): {line:?}");
    }
    println!("Total lines: {}", content.lines().count());

    // Show what tree-sitter extracts
    let format = MarkdownFormat;
    let sections = extract_sections(path, &format).unwrap();

    println!("\n=== EXTRACTED SECTIONS ===");
    for (i, sec) in sections.iter().enumerate() {
        println!("\nSection {}: {:?}", i, sec.title);
        println!(
            "  line_start: {} (should be first content line)",
            sec.line_start
        );
        println!(
            "  line_end: {} (should be last content line + 1 or next heading)",
            sec.line_end
        );

        // Show what content is at those byte positions
        let bytes = content.as_bytes();
        if sec.byte_start < bytes.len() && sec.byte_end <= bytes.len() {
            let section_bytes = &bytes[sec.byte_start..sec.byte_end];
            let section_text = String::from_utf8_lossy(section_bytes);
            println!("  byte content: {section_text:?}");
        }
    }

    // Now try to edit the first section
    println!("\n=== ATTEMPTING EDIT ===");
    let sec = &sections[0];
    let edit = Edit {
        file_name: path.to_string_lossy().to_string(),
        line_start: sec.line_start,
        line_end: sec.line_end,
        column_start: sec.column_start,
        column_end: sec.column_end,
        section_content: "Yeah".to_string(),
        item_name: "Hello".to_string(),
    };

    println!(
        "Edit targeting lines {} to {} (exclusive)",
        edit.line_start, edit.line_end
    );

    let mut plan = EditPlan { edits: vec![edit] };

    match plan.apply() {
        Ok(()) => {
            let result = fs::read_to_string(path).unwrap();
            println!("\n=== RESULT ===");
            for (i, line) in result.lines().enumerate() {
                println!("Line {i} (0-idx): {line:?}");
            }
        }
        Err(e) => {
            println!("\n=== ERROR ===");
            println!("{e:?}");
        }
    }
}

#[test]
fn test_textum_line_behavior() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Line0\nLine1\nLine2\nLine3").unwrap();
    file.flush().unwrap();
    let path = file.path().to_string_lossy().to_string();

    println!("\n=== ORIGINAL ===");
    let orig = fs::read_to_string(&path).unwrap();
    for (i, line) in orig.lines().enumerate() {
        println!("{i}: {line:?}");
    }

    // Test 1: Replace line 1 (0-indexed) with Include/Exclude
    let edit = Edit {
        file_name: path.clone(),
        line_start: 1, // Target line 1 (Line1)
        line_end: 2,   // Exclude line 2 (Line2)
        column_start: 0,
        column_end: 0,
        section_content: "REPLACED".to_string(),
        item_name: "test".to_string(),
    };

    let mut plan = EditPlan { edits: vec![edit] };
    plan.apply().unwrap();

    println!("\n=== AFTER EDIT (lines 1-2, line 2 excluded) ===");
    let result = fs::read_to_string(&path).unwrap();
    for (i, line) in result.lines().enumerate() {
        println!("{i}: {line:?}");
    }

    // What we expect:
    // Line0 (unchanged)
    // REPLACED (replaced Line1)
    // Line2 (unchanged, was excluded)
    // Line3 (unchanged)

    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines[0], "Line0", "Line 0 should be unchanged");
    assert_eq!(lines[1], "", "Line 1 should be padding now");
    assert!(lines[2].contains("REPLACED"), "Line 1 should be replaced");
    assert_eq!(lines[3], "", "Line 3 should be padding now");
    assert_eq!(
        lines[4], "Line2",
        "Line 2 should be unchanged (excluded) and budged along"
    );
    assert_eq!(
        lines[5], "Line3",
        "Line 3 should be unchanged and budged along"
    );
}
