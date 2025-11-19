//! Format trait and implementations for different document types.
//!
//! This module defines the `Format` trait which abstracts over different
//! document formats (markdown, org-mode, restructuredtext, etc.) by providing
//! tree-sitter queries specific to each format.

pub mod difftastic;
pub mod markdown;

/// Abstracts document type differences through tree-sitter queries.
///
/// Enables support for markdown and other structured formats by providing format-specific parsing
/// queries (tree-sitter uses SCM lisp queries).
pub trait Format {
    /// File extension for syntax highlighting (e.g., "md", "rs")
    fn file_extension(&self) -> &'static str;
    /// Returns the tree-sitter language parser for this format.
    fn language(&self) -> tree_sitter::Language;
    /// Tree-sitter query matching section boundaries in this format.
    fn section_query(&self) -> &str;
    /// Tree-sitter query extracting section titles in this format.
    fn title_query(&self) -> &str;
    /// Format a section heading for display with syntax highlighting
    fn format_section_display(&self, level: usize, title: &str) -> ratatui::text::Line<'static>;
}
