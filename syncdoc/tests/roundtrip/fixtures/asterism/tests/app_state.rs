use super::AppState;
use crate::section::Section;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_edit_persists_correctly() {
    // Create a test file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# Hello\n\n?\n\n## World\n\n??").unwrap();
    let path = file.path().to_path_buf();

    // Create sections matching the file
    let sections = vec![
        Section {
            title: "Hello".to_string(),
            level: 1,
            line_start: 2,
            line_end: 4,
            column_start: 0,
            column_end: 7,
            byte_start: 10,
            byte_end: 12,
            file_path: path.to_string_lossy().to_string(),
            parent_index: None,
            children_indices: vec![1],
            section_content: None,
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
        },
        Section {
            title: "World".to_string(),
            level: 2,
            line_start: 5,
            line_end: 6,
            column_start: 1,
            column_end: 8,
            byte_start: 23,
            byte_end: 25,
            file_path: path.to_string_lossy().to_string(),
            parent_index: Some(0),
            children_indices: vec![],
            section_content: None,
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
        },
    ];

    let mut app = AppState::new(vec![path.clone()], sections, 100);

    // Find first navigable node (should be first section)
    if let Some(first) = app.navigate_to_first() {
        app.current_node_index = first;
    }

    // Enter detail view for first section
    app.enter_detail_view();

    // Simulate editing: replace "?" with "Yeah"
    if let Some(ref mut editor_state) = app.editor_state {
        editor_state.lines = edtui::Lines::from("\nYeah\n");
    }

    // Save
    app.save_current().unwrap();

    // Verify file content
    let content = fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    println!("{lines:?}");

    assert_eq!(
        lines,
        vec!["# Hello", "", "", "Yeah", "", "## World", "", "??"]
    );
}

#[test]
fn test_edit_plan_captures_changes() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# Test\n\nOriginal").unwrap();
    let path = file.path().to_path_buf();

    let sections = vec![Section {
        title: "Test".to_string(),
        level: 1,
        line_start: 2,
        line_end: 3,
        column_start: 1,
        column_end: 7,
        byte_start: 9,
        byte_end: 17,
        file_path: path.to_string_lossy().to_string(),
        parent_index: None,
        children_indices: vec![],
        section_content: None,
        chunk_type: None,
        lhs_content: None,
        rhs_content: None,
    }];

    let mut app = AppState::new(vec![path.clone()], sections, 100);

    if let Some(first) = app.navigate_to_first() {
        app.current_node_index = first;
    }

    app.enter_detail_view();

    // Make an edit
    if let Some(ref mut editor_state) = app.editor_state {
        editor_state.lines = edtui::Lines::from("\nModified\n");
    }

    app.save_current().unwrap();
    app.exit_detail_view(true);

    // Generate plan
    let plan = app.generate_edit_plan();

    assert!(
        !plan.edits.is_empty(),
        "Edit plan should contain the saved edit"
    );
    assert_eq!(plan.edits[0].section_content, "\nModified\n");
}

#[test]
fn test_multiple_edits_correct_offsets() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# One\n\nA\n\n## Two\n\nB\n\n### Three\n\nC").unwrap();
    let path = file.path().to_path_buf();

    let sections = vec![
        Section {
            title: "One".to_string(),
            level: 1,
            line_start: 3,
            line_end: 5,
            column_start: 1,
            column_end: 6,
            byte_start: 8,
            byte_end: 10,
            file_path: path.to_string_lossy().to_string(),
            parent_index: None,
            children_indices: vec![1],
            section_content: None,
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
        },
        Section {
            title: "Two".to_string(),
            level: 2,
            line_start: 6,
            line_end: 8,
            column_start: 1,
            column_end: 7,
            byte_start: 19,
            byte_end: 21,
            file_path: path.to_string_lossy().to_string(),
            parent_index: Some(0),
            children_indices: vec![2],
            section_content: None,
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
        },
        Section {
            title: "Three".to_string(),
            level: 3,
            line_start: 9,
            line_end: 11,
            column_start: 1,
            column_end: 10,
            byte_start: 33,
            byte_end: 35,
            file_path: path.to_string_lossy().to_string(),
            parent_index: Some(1),
            children_indices: vec![],
            section_content: None,
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
        },
    ];

    let mut app = AppState::new(vec![path.clone()], sections, 100);

    // Find and edit first section
    if let Some(first) = app.navigate_to_first() {
        app.current_node_index = first;
    }
    app.enter_detail_view();
    if let Some(ref mut editor_state) = app.editor_state {
        editor_state.lines = edtui::Lines::from("\nAAA\n");
    }
    app.save_current().unwrap();
    app.exit_detail_view(true);

    // Find and edit third section (index 2)
    // After rebuild, need to find the section again
    if let Some(node_idx) = app.tree_nodes.iter().position(|n| {
        if let Some(section_idx) = n.section_index {
            app.sections
                .get(section_idx)
                .is_some_and(|s| s.title == "Three")
        } else {
            false
        }
    }) {
        app.current_node_index = node_idx;
    }

    app.enter_detail_view();
    if let Some(ref mut editor_state) = app.editor_state {
        editor_state.lines = edtui::Lines::from("\nCCC\n");
    }
    app.save_current().unwrap();

    // Verify file content
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("AAA"),
        "First edit should persist: {content}"
    );
    assert!(
        content.contains("CCC"),
        "Second edit should be at correct position: {content}"
    );
    assert!(content.contains("## Two"), "Middle section should remain");
}

#[test]
fn test_tree_structure_single_file() {
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
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
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
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
        },
    ];

    let app = AppState::new(vec![path], sections, 100);

    // Single file mode should only show sections, no file nodes
    assert_eq!(app.tree_nodes.len(), 2);
    assert!(app.tree_nodes[0].navigable);
    assert!(app.tree_nodes[1].navigable);
}

#[test]
fn test_tree_structure_multi_file() {
    let mut file1 = NamedTempFile::new().unwrap();
    let mut file2 = NamedTempFile::new().unwrap();

    writeln!(file1, "# One\n\nA").unwrap();
    writeln!(file2, "# Two\n\nB").unwrap();

    let path1 = file1.path().to_path_buf();
    let path2 = file2.path().to_path_buf();

    let sections = vec![
        Section {
            title: "One".to_string(),
            level: 1,
            line_start: 2,
            line_end: 3,
            column_start: 0,
            column_end: 5,
            byte_start: 8,
            byte_end: 10,
            file_path: path1.to_string_lossy().to_string(),
            parent_index: None,
            children_indices: vec![],
            section_content: None,
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
        },
        Section {
            title: "Two".to_string(),
            level: 1,
            line_start: 2,
            line_end: 3,
            column_start: 0,
            column_end: 5,
            byte_start: 8,
            byte_end: 10,
            file_path: path2.to_string_lossy().to_string(),
            parent_index: None,
            children_indices: vec![],
            section_content: None,
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
        },
    ];

    let app = AppState::new(vec![path1, path2], sections, 100);

    // Multi-file mode should show file nodes + sections
    // 2 files + 2 sections = 4 nodes
    assert_eq!(app.tree_nodes.len(), 4);

    // File nodes should not be navigable
    let mut file_nodes = 0;
    let mut section_nodes = 0;
    for node in &app.tree_nodes {
        if node.navigable {
            section_nodes += 1;
        } else {
            file_nodes += 1;
        }
    }

    assert_eq!(file_nodes, 2, "Should have 2 file nodes");
    assert_eq!(section_nodes, 2, "Should have 2 section nodes");
}

#[test]
fn test_navigation_skips_files() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# One\n\nA\n\n## Two\n\nB").unwrap();
    let path = file.path().to_path_buf();

    // Create a fake multi-file scenario with one file
    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "# Three\n\nC").unwrap();
    let path2 = file2.path().to_path_buf();

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
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
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
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
        },
        Section {
            title: "Three".to_string(),
            level: 1,
            line_start: 2,
            line_end: 3,
            column_start: 0,
            column_end: 7,
            byte_start: 10,
            byte_end: 12,
            file_path: path2.to_string_lossy().to_string(),
            parent_index: None,
            children_indices: vec![],
            section_content: None,
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
        },
    ];

    let mut app = AppState::new(vec![path, path2], sections, 100);

    // Start at first position (should be file node, non-navigable)
    app.current_node_index = 0;
    assert!(
        !app.tree_nodes[0].navigable,
        "First node should be file (non-navigable)"
    );

    // Navigate to next - should skip to first section
    if let Some(next) = app.find_next_node() {
        app.current_node_index = next;
    }

    assert!(
        app.tree_nodes[app.current_node_index].navigable,
        "Should skip to navigable node"
    );
    assert_eq!(
        app.get_current_section().map(|s| s.title.as_str()),
        Some("One"),
        "Should be on 'One' section"
    );
}
