//! The UI renders the application state into something visible and vim-able.
//!
//! The draw function dispatches based on the current view (list or editor).
//! The list view shows a unified tree with files and sections using box-drawing characters.

use crate::app_state::{AppState, MoveState, View};
use crate::config::Config;
use crate::formats::Format;
use crate::section::NodeType;
use edtui::{EditorTheme, EditorView, SyntaxHighlighter};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Renders the active view based on current application state.
pub fn draw(f: &mut Frame, app: &mut AppState, _cfg: &Config) {
    match app.current_view {
        View::List => draw_list(f, app),
        View::Command => draw_list_with_command(f, app),
        View::Detail => draw_detail(f, app),
    }
}

/// Generate box-drawing prefix for tree structure
fn get_tree_prefix(level: usize, parent_states: &[bool]) -> String {
    if level == 0 {
        return String::new();
    }

    let mut prefix = String::new();

    // Draw indentation and vertical lines for parent levels
    for i in 0..level.saturating_sub(1) {
        prefix.push_str("  "); // Two spaces for indentation
        if i < parent_states.len() && parent_states[i] {
            prefix.push_str("│ ");
        } else {
            prefix.push_str("  ");
        }
    }

    // Add final indentation before the branch
    prefix.push_str("  ");

    // Draw branch for current level
    prefix.push_str("   ");

    prefix
}

#[allow(clippy::too_many_lines)]
fn draw_list(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    // Determine if we're in difftastic mode by checking if any section has chunk_type
    let is_difftastic = app.sections.iter().any(|s| s.chunk_type.is_some())
        || app.sections.iter().any(|s| s.title.contains("@@"));

    let format: Box<dyn Format> = if is_difftastic {
        Box::new(crate::formats::difftastic::DifftasticFormat)
    } else {
        Box::new(crate::formats::markdown::MarkdownFormat)
    };

    let mut is_last_at_level = vec![false; app.tree_nodes.len()];

    // Calculate which nodes are last children of their parent for box-drawing
    for (i, node) in app.tree_nodes.iter().enumerate() {
        // A node is "last" if there's no subsequent sibling (same parent, same tree_level)
        let current_parent = node
            .section_index
            .and_then(|idx| app.sections[idx].parent_index);

        let current_level = node.tree_level;
        let mut has_sibling_after = false;
        for j in (i + 1)..app.tree_nodes.len() {
            let next_level = app.tree_nodes[j].tree_level;

            // If we encounter a node at a lower tree level, stop searching
            if next_level < current_level {
                break;
            }

            // If same tree level, check if it's a sibling (same parent)
            if next_level == current_level {
                let next_parent = app.tree_nodes[j]
                    .section_index
                    .and_then(|idx| app.sections[idx].parent_index);
                if next_parent == current_parent {
                    has_sibling_after = true;
                    break;
                }
            }
        }

        is_last_at_level[i] = !has_sibling_after;
    }

    // Track which parent levels still have siblings coming
    let mut parent_has_siblings: Vec<bool> = Vec::new();

    let items: Vec<ListItem> = app
        .tree_nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            // Update parent tracking
            while parent_has_siblings.len() > node.tree_level {
                parent_has_siblings.pop();
            }
            while parent_has_siblings.len() < node.tree_level {
                parent_has_siblings.push(false);
            }
            if node.tree_level > 0 && !parent_has_siblings.is_empty() {
                let parent_idx = parent_has_siblings.len() - 1;
                parent_has_siblings[parent_idx] = !is_last_at_level[i];
            }

            let tree_prefix = get_tree_prefix(node.tree_level, &parent_has_siblings);

            let line = match &node.node_type {
                NodeType::Directory { name, .. } => {
                    let spans = vec![
                        Span::raw(tree_prefix),
                        Span::styled(
                            format!("📁 {name}"),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ];
                    Line::from(spans)
                }
                NodeType::File { name, .. } => {
                    let spans = vec![
                        Span::raw(tree_prefix),
                        Span::styled(
                            format!("  📄 {name}"),
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ];
                    Line::from(spans)
                }
                NodeType::Section(section) => {
                    // Calculate indentation based on section level
                    let indent = "  ".repeat(section.level.saturating_sub(1));

                    let mut highlighted_line = format
                        .as_ref()
                        .format_section_display(section.level, &section.title);

                    // Prepend indent + tree prefix
                    let mut spans = vec![Span::raw(indent), Span::raw(tree_prefix)];
                    spans.append(&mut highlighted_line.spans);

                    Line::from(spans)
                }
            };

            // Determine style based on selection and move state
            let style = if node.section_index == app.moving_section_index {
                match app.move_state {
                    MoveState::Selected => Style::default()
                        .fg(Color::Rgb(255, 165, 0)) // Orange
                        .add_modifier(Modifier::BOLD),
                    MoveState::Moved => {
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    }
                    MoveState::None => {
                        if i == app.current_node_index {
                            Style::default().add_modifier(Modifier::REVERSED)
                        } else {
                            Style::default()
                        }
                    }
                }
            } else if i == app.current_node_index && node.navigable {
                Style::default().add_modifier(Modifier::REVERSED)
            } else if !node.navigable {
                // Dim non-navigable nodes slightly
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let title = match (
        app.file_mode == crate::app_state::FileMode::Multi,
        &app.move_state,
    ) {
        (true, MoveState::None) => format!("Files & Sections ({} files)", app.files.len()),
        (true, _) => format!("Files & Sections (MOVING - {} files)", app.files.len()),
        (false, MoveState::None) => "Sections".to_string(),
        (false, _) => "Sections (MOVING)".to_string(),
    };

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(list, chunks[0]);

    let help = if app.move_state == MoveState::None {
        "↑/↓: Navigate | ←/→: Parent/Child | Enter: Edit | Ctrl+↑/↓/←/→: Start Move | q: Quit"
    } else {
        "Ctrl+↑/↓: Move | Ctrl+←/→: Level | Ctrl+Home/End: Top/Bottom | :w Save | Esc: Cancel"
    };

    let help_widget = Paragraph::new(help).block(Block::default().borders(Borders::ALL));
    f.render_widget(help_widget, chunks[1]);
}

#[allow(clippy::too_many_lines)]
fn draw_list_with_command(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    // Determine if we're in difftastic mode by checking if any section has chunk_type
    let is_difftastic = app.sections.iter().any(|s| s.chunk_type.is_some())
        || app.sections.iter().any(|s| s.title.contains("@@"));

    let format: Box<dyn Format> = if is_difftastic {
        Box::new(crate::formats::difftastic::DifftasticFormat)
    } else {
        Box::new(crate::formats::markdown::MarkdownFormat)
    };

    // Calculate which nodes are last at their level
    let mut is_last_at_level: Vec<bool> = vec![false; app.tree_nodes.len()];
    for (i, node) in app.tree_nodes.iter().enumerate() {
        let current_level = node.tree_level;
        let mut found_next = false;
        for j in (i + 1)..app.tree_nodes.len() {
            if app.tree_nodes[j].tree_level < current_level {
                break;
            }
            if app.tree_nodes[j].tree_level == current_level {
                found_next = true;
                break;
            }
        }
        is_last_at_level[i] = !found_next;
    }

    let mut parent_has_siblings: Vec<bool> = Vec::new();

    let items: Vec<ListItem> = app
        .tree_nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            while parent_has_siblings.len() > node.tree_level {
                parent_has_siblings.pop();
            }
            while parent_has_siblings.len() < node.tree_level {
                parent_has_siblings.push(false);
            }
            if node.tree_level > 0 && !parent_has_siblings.is_empty() {
                let parent_idx = parent_has_siblings.len() - 1;
                parent_has_siblings[parent_idx] = !is_last_at_level[i];
            }

            let tree_prefix = if app.file_mode == crate::app_state::FileMode::Multi {
                get_tree_prefix(node.tree_level, &parent_has_siblings)
            } else {
                String::new()
            };

            let line = match &node.node_type {
                NodeType::Directory { name, .. } => {
                    let spans = vec![
                        Span::raw(tree_prefix),
                        Span::styled(
                            format!("📁 {name}"),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ];
                    Line::from(spans)
                }
                NodeType::File { name, .. } => {
                    let spans = vec![
                        Span::raw(tree_prefix),
                        Span::styled(
                            format!("  📄 {name}"),
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ];
                    Line::from(spans)
                }
                NodeType::Section(section) => {
                    let mut highlighted_line = format
                        .as_ref()
                        .format_section_display(section.level, &section.title);

                    // Prepend tree prefix
                    let mut spans = vec![Span::raw(tree_prefix)];
                    spans.append(&mut highlighted_line.spans);

                    Line::from(spans)
                }
            };

            let style = if node.section_index == app.moving_section_index {
                match app.move_state {
                    MoveState::Selected => Style::default()
                        .fg(Color::Rgb(255, 165, 0))
                        .add_modifier(Modifier::BOLD),
                    MoveState::Moved => {
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    }
                    MoveState::None => {
                        if i == app.current_node_index {
                            Style::default().add_modifier(Modifier::REVERSED)
                        } else {
                            Style::default()
                        }
                    }
                }
            } else if i == app.current_node_index && node.navigable {
                Style::default().add_modifier(Modifier::REVERSED)
            } else if !node.navigable {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let title = match (
        app.file_mode == crate::app_state::FileMode::Multi,
        &app.move_state,
    ) {
        (true, MoveState::None) => format!("Files & Sections ({} files)", app.files.len()),
        (true, _) => format!("Files & Sections (MOVING - {} files)", app.files.len()),
        (false, MoveState::None) => "Sections".to_string(),
        (false, _) => "Sections (MOVING)".to_string(),
    };

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(list, chunks[0]);

    // Show command buffer instead of help
    let command_text = format!(":{}", app.command_buffer);
    let command_widget =
        Paragraph::new(command_text).block(Block::default().borders(Borders::ALL).title("Command"));
    f.render_widget(command_widget, chunks[1]);
}

fn draw_detail(f: &mut Frame, app: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Breadcrumb
            Constraint::Min(0),    // Editor
            Constraint::Length(3), // Help
        ])
        .split(f.area());

    // Breadcrumb navigation
    if let Some(section_idx) = app.get_current_section_index() {
        let section = &app.sections[section_idx];
        let mut breadcrumb_parts = Vec::new();
        let mut current_idx = Some(section_idx);

        while let Some(idx) = current_idx {
            breadcrumb_parts.push(app.sections[idx].title.clone());
            current_idx = app.sections[idx].parent_index;
        }

        breadcrumb_parts.reverse();
        let breadcrumb = breadcrumb_parts.join(" > ");

        let breadcrumb_widget = Paragraph::new(breadcrumb)
            .block(Block::default().borders(Borders::ALL).title("Navigation"));
        f.render_widget(breadcrumb_widget, chunks[0]);

        // Editor
        let max_width = app.get_max_line_width();
        let title = format!("Section: {} (max line: {} chars)", section.title, max_width);

        if let Some(ref mut editor_state) = app.editor_state {
            let block = Block::default().borders(Borders::ALL).title(title);
            let inner = block.inner(chunks[1]);
            f.render_widget(block, chunks[1]);

            let syntax_highlighter = SyntaxHighlighter::new("dracula", "md");
            let editor = EditorView::new(editor_state)
                .theme(EditorTheme::default())
                .syntax_highlighter(Some(syntax_highlighter))
                .wrap(true);

            f.render_widget(editor, inner);
        }
    }

    // Help/command line
    let help_text = if app.current_view == View::Command {
        format!(":{}", app.command_buffer)
    } else if let Some(ref msg) = app.message {
        msg.clone()
    } else {
        ":w Save | :x Save & Exit | :q Quit | :q! Force Quit | :wn Save & Next | :wp Save & Prev"
            .to_string()
    };

    let help = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}
