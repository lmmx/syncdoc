//! Difftastic format implementation for displaying structural diffs.
//!
//! This module provides support for parsing difftastic JSON output and
//! converting it into sections that can be navigated and edited in asterism.

use crate::formats::Format;
use crate::section::{ChunkType, Section};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Write;
use std::path::Path;
use std::{fs, io};

/// Represents a file in difftastic output
#[derive(Debug, Serialize, Deserialize)]
pub struct DifftFile {
    /// Programming language identified by difftastic for syntax highlighting.
    pub language: String,
    /// File path relative to the comparison root.
    pub path: String,
    /// Grouped diff hunks, each containing lines that changed together.
    #[serde(default)]
    pub chunks: Option<Vec<Vec<DifftLine>>>,
    /// Change classification: "unchanged", "changed", "created", or "deleted".
    pub status: String,
}

/// Represents a line in a diff chunk
#[derive(Debug, Serialize, Deserialize)]
pub struct DifftLine {
    /// Left-hand (original) side of the comparison, absent for pure additions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lhs: Option<DifftSide>,
    /// Right-hand (modified) side of the comparison, absent for pure deletions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rhs: Option<DifftSide>,
}

/// Represents one side (left or right) of a diff line
#[derive(Debug, Serialize, Deserialize)]
pub struct DifftSide {
    /// Original line position in the source file (1-indexed).
    pub line_number: u32,
    /// Structural changes within this line, ordered by column position.
    pub changes: Vec<DifftChange>,
}

/// Represents a change within a line
#[derive(Debug, Serialize, Deserialize)]
pub struct DifftChange {
    /// Column offset where this change begins (0-indexed).
    pub start: u32,
    /// Column offset where this change ends (exclusive).
    pub end: u32,
    /// Text content of this change segment.
    pub content: String,
    /// Syntax category for rendering: "delimiter", "string", "keyword", "comment", "type", "normal"
    /// or "`tree_sitter_error`".
    pub highlight: String,
}

/// Difftastic format handler
pub struct DifftasticFormat;

impl Format for DifftasticFormat {
    fn file_extension(&self) -> &'static str {
        "diff"
    }

    fn language(&self) -> tree_sitter::Language {
        // Difftastic doesn't use tree-sitter parsing
        tree_sitter_md::LANGUAGE.into()
    }

    fn section_query(&self) -> &'static str {
        ""
    }

    fn title_query(&self) -> &'static str {
        ""
    }

    fn format_section_display(&self, level: usize, title: &str) -> Line<'static> {
        // Check if this is a hunk header with format: (N) @@ -X,Y +A,B @@
        if title.contains("@@") && title.starts_with('(') {
            if let Some(close_paren) = title.find(')') {
                let hunk_num = &title[..=close_paren];
                let rest = &title[close_paren + 1..].trim();

                // Determine color based on the diff header
                let color = Self::determine_hunk_color_from_header(rest);

                let spans = vec![
                    Span::styled(hunk_num.to_string(), Style::default().fg(color)),
                    Span::raw(" "),
                    Span::raw((*rest).to_string()),
                ];

                return Line::from(spans);
            }
        }

        // For file nodes or other sections
        let color = if level == 0 {
            Color::Cyan // Files
        } else {
            Color::LightYellow // Hunks
        };

        let spans = vec![
            Span::styled("● ", Style::default().fg(color)),
            Span::raw(title.to_string()),
        ];

        Line::from(spans)
    }
}

impl DifftasticFormat {
    /// Determine hunk color from the header string itself
    fn determine_hunk_color_from_header(header: &str) -> Color {
        // Parse @@ -X,Y +A,B @@
        if let Some(hunk_part) = header.strip_prefix("@@").and_then(|s| s.split("@@").next()) {
            let parts: Vec<&str> = hunk_part.split_whitespace().collect();
            if parts.len() >= 2 {
                let lhs = parts[0].trim_start_matches('-');
                let rhs = parts[1].trim_start_matches('+');

                let lhs_count = lhs
                    .split(',')
                    .nth(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(1);
                let rhs_count = rhs
                    .split(',')
                    .nth(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(1);

                // Parse the line numbers too to detect -0,0 +1,N
                let lhs_start = lhs
                    .split(',')
                    .next()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);

                if lhs_start == 0 && lhs_count == 0 && rhs_count > 0 {
                    return Color::Green; // Addition
                } else if lhs_count > 0 && rhs_count == 0 && rhs.starts_with("0,") {
                    return Color::Red; // Deletion
                } else if lhs_count == 0 && rhs_count > 0 {
                    return Color::Green; // Pure addition
                } else if lhs_count > 0 && rhs_count == 0 {
                    return Color::Red; // Pure deletion
                }
            }
        }
        Color::LightYellow // Modification
    }
}

#[allow(clippy::too_many_arguments)]
fn create_chunk_section(
    file_path: &str,
    title: String,
    line_num: i64,
    column_start: i64,
    column_end: i64,
    chunk_type: ChunkType,
    lhs_text: Option<String>,
    rhs_text: Option<String>,
) -> Section {
    Section {
        title,
        level: 2,
        line_start: line_num,
        line_end: line_num + 1,
        column_start,
        column_end,
        byte_start: 0,
        byte_end: 0,
        file_path: file_path.to_string(),
        parent_index: None,
        children_indices: Vec::new(),
        section_content: None,
        chunk_type: Some(chunk_type),
        lhs_content: lhs_text,
        rhs_content: rhs_text,
    }
}

/// Parse difftastic JSON output into sections
///
/// Files become non-navigable containers, hunks become navigable sections.
///
/// # Errors
///
/// Returns an error if JSON parsing fails or if the format is invalid.
pub fn parse_difftastic_json(json_str: &str) -> io::Result<Vec<Section>> {
    let files: Vec<DifftFile> = if let Ok(files) = serde_json::from_str::<Vec<DifftFile>>(json_str)
    {
        // Array format: [{file1}, {file2}]
        files
    } else if json_str.trim().starts_with('[') {
        // Failed to parse as array, invalid format
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid JSON array format",
        ));
    } else {
        // Try parsing as newline-delimited JSON (NDJSON/JSON Lines)
        json_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<DifftFile>(line).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to parse JSON line: {e}"),
                    )
                })
            })
            .collect::<Result<Vec<DifftFile>, io::Error>>()?
    };

    let mut sections = Vec::new();
    let mut global_line = 0i64;

    for file in &files {
        // Skip unchanged files
        if file.status == "unchanged" {
            continue;
        }

        let file_path = &file.path;

        // Create hunk sections directly (no file section)
        if let Some(chunks) = &file.chunks {
            let mut hunk_counter = 0;
            for chunk in chunks {
                // Each element in the chunk array is a separate hunk
                for change in chunk {
                    hunk_counter += 1;

                    // Format this individual change as a proper git diff hunk
                    let hunk_title = format_hunk_header(change, hunk_counter);
                    let hunk_content = format_change_content(change);

                    let hunk_start_line = global_line;
                    let hunk_end_line =
                        global_line + i64::try_from(hunk_content.lines().count()).unwrap_or(0);

                    // Create section for this hunk
                    sections.push(Section {
                        title: hunk_title,
                        level: 1,
                        line_start: hunk_start_line,
                        line_end: hunk_end_line,
                        column_start: 0,
                        column_end: 0,
                        byte_start: 0,
                        byte_end: 0,
                        file_path: file_path.clone(),
                        parent_index: None,
                        children_indices: Vec::new(),
                        section_content: Some(vec![hunk_content]),
                        chunk_type: None,
                        lhs_content: None,
                        rhs_content: None,
                    });

                    global_line = hunk_end_line + 1;
                }
            }
        } else if file.status == "created" || file.status == "deleted" {
            // For files with no chunks (created/deleted without detailed hunks),
            // create a proper hunk header

            // Try to read the file to get line count
            let line_count = if file.status == "created" {
                // For created files, try to read from filesystem
                std::fs::read_to_string(file_path)
                    .ok()
                    .map_or(0, |content| content.lines().count())
            } else {
                0 // Deleted files
            };

            let hunk_title = if file.status == "created" {
                format!("(1) @@ -0,0 +1,{line_count} @@")
            } else {
                format!("(1) @@ -1,{line_count} +0,0 @@")
            };

            let hunk_content = if file.status == "created" {
                std::fs::read_to_string(file_path)
                    .ok()
                    .unwrap_or_else(|| format!("File was {}", file.status))
            } else {
                format!("File was {}", file.status)
            };

            sections.push(Section {
                title: hunk_title,
                level: 1,
                line_start: global_line,
                line_end: global_line + i64::try_from(line_count).unwrap_or(0),
                column_start: 0,
                column_end: 0,
                byte_start: 0,
                byte_end: 0,
                file_path: file_path.clone(),
                parent_index: None,
                children_indices: Vec::new(),
                section_content: Some(hunk_content.lines().map(String::from).collect()),
                chunk_type: None,
                lhs_content: None,
                rhs_content: None,
            });

            global_line += i64::try_from(line_count).unwrap_or(0) + 1;
        }
    }

    Ok(sections)
}

/// Format a change as a proper git diff hunk header
fn format_hunk_header(change: &DifftLine, hunk_num: usize) -> String {
    let (lhs_line, rhs_line) = match (&change.lhs, &change.rhs) {
        (Some(lhs), Some(rhs)) => (lhs.line_number, rhs.line_number),
        (Some(lhs), None) => (lhs.line_number, 0),
        (None, Some(rhs)) => (0, rhs.line_number),
        _ => (0, 0),
    };

    // Determine chunk size (for now, single line changes)
    let lhs_count = i32::from(change.lhs.is_some());
    let rhs_count = i32::from(change.rhs.is_some());

    format!("({hunk_num}) @@ -{lhs_line},{lhs_count} +{rhs_line},{rhs_count} @@")
}

/// Format a single change for display
fn format_change_content(change: &DifftLine) -> String {
    let mut output = String::new();

    match (&change.lhs, &change.rhs) {
        (Some(lhs), Some(rhs)) => {
            // Modified line - show both sides
            write!(output, "-{}: ", lhs.line_number).unwrap();
            for ch in &lhs.changes {
                output.push_str(&ch.content);
            }
            output.push('\n');

            write!(output, "+{}: ", rhs.line_number).unwrap();
            for ch in &rhs.changes {
                output.push_str(&ch.content);
            }
            output.push('\n');
        }
        (Some(lhs), None) => {
            // Deleted line
            write!(output, "-{}: ", lhs.line_number).unwrap();
            for ch in &lhs.changes {
                output.push_str(&ch.content);
            }
            output.push('\n');
        }
        (None, Some(rhs)) => {
            // Added line
            write!(output, "+{}: ", rhs.line_number).unwrap();
            for ch in &rhs.changes {
                output.push_str(&ch.content);
            }
            output.push('\n');
        }
        (None, None) => {
            output.push_str(" \n");
        }
    }

    output
}

fn extract_chunk_text(side: &Value) -> Option<String> {
    side.get("changes")
        .and_then(|c| c.as_array())
        .map(|changes| {
            changes
                .iter()
                .filter_map(|change| change.get("content").and_then(|c| c.as_str()))
                .collect::<String>()
        })
}

fn extract_column_range(side: &Value) -> (i64, i64) {
    let changes = side.get("changes").and_then(|c| c.as_array());

    let start = changes
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("start"))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0);

    let end = changes
        .and_then(|arr| arr.last())
        .and_then(|last| last.get("end"))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0);

    (start, end)
}

/// Extract the difftastic hunks as sections (same as sections in a markdown etc)
///
/// # Errors
///
/// Returns an error if the JSON file cannot be read from disk.
pub fn extract_difftastic_sections(json_path: &Path) -> io::Result<Vec<Section>> {
    let content = fs::read_to_string(json_path)?;
    let lines: Vec<Value> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    let mut sections = Vec::new();

    for value in lines {
        let file_path = value
            .get("path")
            .and_then(|p| p.as_str())
            .unwrap_or("unknown");

        if let Some(chunks) = value.get("chunks").and_then(|c| c.as_array()) {
            for chunk_array in chunks {
                if let Some(chunk_list) = chunk_array.as_array() {
                    for chunk in chunk_list {
                        let lhs = chunk.get("lhs");
                        let rhs = chunk.get("rhs");

                        let chunk_type = match (lhs, rhs) {
                            (Some(_), None) => ChunkType::Deleted,
                            (None, Some(_)) => ChunkType::Added,
                            (Some(l), Some(r)) if l != r => ChunkType::Modified,
                            (Some(_), Some(_)) => ChunkType::Unchanged,
                            _ => continue,
                        };

                        let line_num = lhs
                            .or(rhs)
                            .and_then(|v| v.get("line_number"))
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0);

                        let (column_start, column_end) =
                            lhs.or(rhs).map_or((0, 0), extract_column_range);

                        let title = format!("Chunk @@ {file_path}:{line_num} @@");
                        let lhs_text = lhs.and_then(extract_chunk_text);
                        let rhs_text = rhs.and_then(extract_chunk_text);

                        sections.push(create_chunk_section(
                            file_path,
                            title,
                            line_num,
                            column_start,
                            column_end,
                            chunk_type,
                            lhs_text,
                            rhs_text,
                        ));
                    }
                }
            }
        }
    }

    Ok(sections)
}

#[cfg(test)]
#[path = "../tests/difftastic.rs"]
mod tests;
