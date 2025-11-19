//! Document discovery and section extraction using tree-sitter.
//!
//! This module handles finding markdown files in the filesystem and parsing
//! them with tree-sitter queries to extract section hierarchies.

use crate::formats::Format;
use crate::section::Section;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

/// Find documents matching the given extensions.
///
/// If paths is empty, scans the current directory recursively.
/// Skips common build/dependency directories.
///
/// # Errors
///
/// Returns an error if directory traversal fails.
pub fn find_documents(paths: Vec<PathBuf>, extensions: &[String]) -> io::Result<Vec<PathBuf>> {
    if paths.is_empty() {
        find_in_directory(Path::new("."), extensions)
    } else {
        let mut results = Vec::new();
        for path in paths {
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions
                        .iter()
                        .any(|e| e == ext.to_string_lossy().as_ref())
                    {
                        results.push(path);
                    }
                }
            } else if path.is_dir() {
                results.extend(find_in_directory(&path, extensions)?);
            }
        }
        Ok(results)
    }
}

fn find_in_directory(dir: &Path, extensions: &[String]) -> io::Result<Vec<PathBuf>> {
    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if ["target", "dist", ".git", "node_modules"].contains(&name_str.as_ref()) {
                        continue;
                    }
                }
                results.extend(find_in_directory(&path, extensions)?);
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions
                        .iter()
                        .any(|e| e == ext.to_string_lossy().as_ref())
                    {
                        results.push(path);
                    }
                }
            }
        }
    }

    Ok(results)
}

/// Extract sections from a document using tree-sitter.
///
/// Parses the file with the given format and extracts all sections,
/// building parent/child relationships based on heading levels.
///
/// # Errors
///
/// Returns an error if file reading or parsing fails.
pub fn extract_sections<F: Format>(file_path: &Path, format: &F) -> io::Result<Vec<Section>> {
    let content = fs::read_to_string(file_path)?;

    let mut parser = Parser::new();
    parser
        .set_language(&format.language())
        .map_err(|e| io::Error::other(format!("Language error: {e}")))?;

    let tree = parser
        .parse(&content, None)
        .ok_or_else(|| io::Error::other("Parse failed"))?;

    let section_query = Query::new(&format.language(), format.section_query())
        .map_err(|e| io::Error::other(format!("Query error: {e}")))?;

    let title_query = Query::new(&format.language(), format.title_query())
        .map_err(|e| io::Error::other(format!("Query error: {e}")))?;

    // Collect all heading nodes by traversing the entire tree
    let mut headings: Vec<tree_sitter::Node> = Vec::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&section_query, tree.root_node(), content.as_bytes());

    while let Some(m) = matches.next() {
        if let Some(c) = m.captures.first() {
            headings.push(c.node);
        }
    }

    let mut sections = Vec::new();

    for (i, heading) in headings.iter().enumerate() {
        // Determine level from the heading marker child node
        let mut level = 1;
        let mut heading_cursor = heading.walk();
        if heading_cursor.goto_first_child() {
            loop {
                let node = heading_cursor.node();
                let kind = node.kind();
                // Match atx_h1_marker, atx_h2_marker, etc.
                if kind.starts_with("atx_h") && kind.ends_with("_marker") {
                    if let Some(level_char) = kind.chars().nth(5) {
                        level = level_char.to_digit(10).unwrap_or(1) as usize;
                    }
                    break;
                }
                if !heading_cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        // Extract title using query
        let mut title_cursor = QueryCursor::new();
        let mut title = String::from("Untitled");
        let mut title_matches = title_cursor.matches(&title_query, *heading, content.as_bytes());
        while let Some(m) = title_matches.next() {
            if let Some(c) = m
                .captures
                .iter()
                .find(|c| title_query.capture_names()[c.index as usize] == "title")
            {
                title = content[c.node.byte_range()].trim().to_string();
                break;
            }
        }

        // Calculate byte range (content only, after heading line)
        let byte_start = heading.end_byte();
        let byte_end = headings
            .get(i + 1)
            .map_or(content.len(), tree_sitter::Node::start_byte);

        // Calculate line coordinates
        // Around line 100-110 in extract_sections
        let line_start = i64::try_from(heading.end_position().row).unwrap_or(0);
        let line_end = headings.get(i + 1).map_or(
            i64::try_from(content.lines().count()).unwrap_or(0),
            |next| i64::try_from(next.start_position().row).unwrap_or(0),
        );

        let column_start = i64::try_from(heading.start_position().column).unwrap_or(0);
        let column_end = i64::try_from(heading.end_position().column).unwrap_or(0);

        sections.push(Section {
            title,
            level,
            line_start,
            line_end,
            column_start,
            column_end,
            byte_start,
            byte_end,
            file_path: file_path.to_string_lossy().to_string(),
            parent_index: None,
            children_indices: Vec::new(),
            section_content: None,
            chunk_type: None,
            lhs_content: None,
            rhs_content: None,
        });
    }

    // Build parent/child relationships
    build_hierarchy(&mut sections);

    Ok(sections)
}

fn build_hierarchy(sections: &mut [Section]) {
    let mut stack: Vec<(usize, usize)> = Vec::new(); // (index, level)

    for i in 0..sections.len() {
        let current_level = sections[i].level;

        // Pop stack until we find parent level
        while let Some(&(_, parent_level)) = stack.last() {
            if parent_level < current_level {
                break;
            }
            stack.pop();
        }

        // Set parent relationship
        if let Some(&(parent_idx, _)) = stack.last() {
            sections[i].parent_index = Some(parent_idx);
            sections[parent_idx].children_indices.push(i);
        }

        stack.push((i, current_level));
    }
}

#[cfg(test)]
#[path = "tests/input.rs"]
mod tests;
