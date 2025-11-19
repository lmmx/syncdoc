use super::AppState;
use crate::section::Section;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_section_move_up() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# One\n\nA\n\n## Two\n\nB\n\n### Three\n\nC").unwrap();
    let path = file.path().to_path_buf();

    let sections = vec![
        Section {
            title: "One".to_string(),
            level: 1,
            line_start: 2,
            line_end: 4,
            column_start: 0,
            column_end: 5,
            byte_start: 8,
            byte_end: 10,
            file_path: path.to_string_lossy().to_string(),
            parent_index: None,
            children_indices: vec![1],
            section_content: None,
        },
        Section {
            title: "Two".to_string(),
            level: 2,
            line_start: 5,
            line_end: 7,
            column_start: 0,
            column_end: 7,
            byte_start: 19,
            byte_end: 21,
            file_path: path.to_string_lossy().to_string(),
            parent_index: Some(0),
            children_indices: vec![2],
            section_content: None,
        },
        Section {
            title: "Three".to_string(),
            level: 3,
            line_start: 8,
            line_end: 10,
            column_start: 0,
            column_end: 11,
            byte_start: 33,
            byte_end: 35,
            file_path: path.to_string_lossy().to_string(),
            parent_index: Some(1),
            children_indices: vec![],
            section_content: None,
        },
    ];

    let mut app = AppState::new(vec![path.clone()], sections, 100);

    // Move section 2 (Three) up
    app.current_section_index = 2;
    app.start_move();
    assert_eq!(app.move_state, MoveState::Selected);

    app.move_section_up();
    assert_eq!(app.move_state, MoveState::Moved);
    assert_eq!(app.sections[1].title, "Three");
    assert_eq!(app.sections[2].title, "Two");
}

#[test]
fn test_section_move_level() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# One\n\nA\n\n## Two\n\nB").unwrap();
    let path = file.path().to_path_buf();

    let sections = vec![
        Section {
            title: "One".to_string(),
            level: 1,
            line_start: 2,
            line_end: 4,
            column_start: 0,
            column_end: 5,
            byte_start: 8,
            byte_end: 10,
            file_path: path.to_string_lossy().to_string(),
            parent_index: None,
            children_indices: vec![1],
            section_content: None,
        },
        Section {
            title: "Two".to_string(),
            level: 2,
            line_start: 5,
            line_end: 7,
            column_start: 0,
            column_end: 7,
            byte_start: 19,
            byte_end: 21,
            file_path: path.to_string_lossy().to_string(),
            parent_index: Some(0),
            children_indices: vec![],
            section_content: None,
        },
    ];

    let mut app = AppState::new(vec![path.clone()], sections, 100);

    // Change level of section 1 (Two)
    app.current_section_index = 1;
    app.start_move();

    // Move in (decrease level number)
    assert_eq!(app.sections[1].level, 2);
    app.move_section_in();
    assert_eq!(app.sections[1].level, 1);

    // Move out (increase level number)
    app.move_section_out();
    assert_eq!(app.sections[1].level, 2);
}

#[test]
fn test_section_reorder_and_save() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        "# Alpha\n\nContent A\n\n## Beta\n\nContent B\n\n### Gamma\n\nContent C"
    )
    .unwrap();
    file.flush().unwrap();
    let path = file.path().to_path_buf();

    let format = MarkdownFormat;
    let sections = input::extract_sections(&path, &format).unwrap();

    let mut app = AppState::new(vec![path.clone()], sections, 100);

    println!("Initial sections:");
    for (i, s) in app.sections.iter().enumerate() {
        println!("  {}: {} (level {})", i, s.title, s.level);
    }

    // Move "Gamma" to position 0
    app.current_section_index = 2;
    app.start_move();
    app.move_section_to_top();

    println!("\nAfter move:");
    for (i, s) in app.sections.iter().enumerate() {
        println!("  {}: {} (level {})", i, s.title, s.level);
    }

    assert_eq!(app.sections[0].title, "Gamma");
    assert_eq!(app.sections[1].title, "Alpha");
    assert_eq!(app.sections[2].title, "Beta");

    // Save the reordering
    app.save_section_reorder().unwrap();

    // Read file and verify order
    let content = fs::read_to_string(&path).unwrap();
    println!("\nFile content:\n{}", content);

    let lines: Vec<&str> = content.lines().collect();
    assert!(lines.iter().any(|l| l.starts_with("### Gamma")));
    assert!(lines.iter().any(|l| l.starts_with("# Alpha")));
    assert!(lines.iter().any(|l| l.starts_with("## Beta")));

    // Verify order in file
    let gamma_pos = lines
        .iter()
        .position(|l| l.starts_with("### Gamma"))
        .unwrap();
    let alpha_pos = lines.iter().position(|l| l.starts_with("# Alpha")).unwrap();
    let beta_pos = lines.iter().position(|l| l.starts_with("## Beta")).unwrap();

    assert!(gamma_pos < alpha_pos, "Gamma should come before Alpha");
    assert!(alpha_pos < beta_pos, "Alpha should come before Beta");
}

#[test]
fn test_section_level_change_and_save() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# Title\n\nContent\n\n## Subtitle\n\nMore content").unwrap();
    file.flush().unwrap();
    let path = file.path().to_path_buf();

    let format = MarkdownFormat;
    let sections = input::extract_sections(&path, &format).unwrap();

    let mut app = AppState::new(vec![path.clone()], sections, 100);

    // Change "Subtitle" from level 2 to level 1
    app.current_section_index = 1;
    app.start_move();
    app.move_section_in();

    assert_eq!(app.sections[1].level, 1);

    app.save_section_reorder().unwrap();

    let content = fs::read_to_string(&path).unwrap();
    println!("File content:\n{}", content);

    assert!(
        content.contains("# Subtitle"),
        "Should be level 1 heading now"
    );
}

#[test]
fn test_cancel_move() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# One\n\n## Two").unwrap();
    let path = file.path().to_path_buf();

    let format = MarkdownFormat;
    let sections = input::extract_sections(&path, &format).unwrap();

    let mut app = AppState::new(vec![path.clone()], sections, 100);

    app.current_section_index = 1;
    app.start_move();
    assert_eq!(app.move_state, MoveState::Selected);

    app.move_section_up();
    assert_eq!(app.move_state, MoveState::Moved);

    // Cancel should reset state
    app.cancel_move();
    assert_eq!(app.move_state, MoveState::None);
    assert_eq!(app.moving_section_index, None);
}
