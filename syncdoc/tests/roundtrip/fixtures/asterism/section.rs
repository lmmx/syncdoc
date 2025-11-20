//! Section representation for tree-sitter parsed documents.
//!
//! A section represents a hierarchical division of a document, typically
//! corresponding to a heading in markdown. Sections track their position
//! in the document tree through parent/child relationships and maintain
//! precise byte and line coordinates for content extraction and modification.

/// Hierarchical document division with precise coordinates for extraction and modification.
#[derive(Clone)]
pub struct Section {
    /// Section heading text without markup symbols.
    pub title: String,
    /// Nesting depth in the document hierarchy (1 for top-level).
    pub level: usize,
    /// First line of section content (after the heading).
    pub line_start: i64,
    /// Line where the next section begins or file ends.
    pub line_end: i64,
    /// Starting column of the section heading.
    pub column_start: i64,
    /// Ending column of the section heading.
    pub column_end: i64,
    /// Byte offset where section content begins.
    pub byte_start: usize,
    /// Byte offset where section content ends.
    pub byte_end: usize,
    /// Source file containing this section.
    pub file_path: String,
    /// Index of the containing section in the hierarchy.
    pub parent_index: Option<usize>,
    /// Indices of directly nested subsections.
    pub children_indices: Vec<usize>,
    /// Edited content for this section (if modified)
    pub section_content: Option<Vec<String>>,
    /// The chunk type (for diffs)
    pub chunk_type: Option<ChunkType>,
    /// The LHS (for diffs)
    pub lhs_content: Option<String>,
    /// The RHS (for diffs)
    pub rhs_content: Option<String>,
}

/// What sort of hunk (syntactic diff atomic unit) it is.
#[derive(Clone)]
pub enum ChunkType {
    /// Only RHS exists
    Added,
    /// Only LHS exists
    Deleted,
    /// Both LHS and RHS exist (and differ)
    Modified,
    /// Both LHS and RHS exist (and are the same, at least syntactically)
    Unchanged,
}

/// Types of nodes that can appear in the file tree view.
#[derive(Clone)]
pub enum NodeType {
    /// Directory node showing a path component
    Directory {
        /// Directory name (not full path)
        name: String,
        /// Full path for reference
        path: String,
    },
    /// File node (non-navigable, just shows filename)
    File {
        /// File name
        name: String,
        /// Full path for reference
        path: String,
    },
    /// Actual document section (navigable)
    Section(Section),
}

/// A node in the unified file + section tree.
#[derive(Clone)]
pub struct TreeNode {
    /// The type of node (directory, file, or section)
    pub node_type: NodeType,
    /// Nesting level in the tree (for indentation/box-drawing)
    pub tree_level: usize,
    /// Whether this node can be selected and edited
    pub navigable: bool,
    /// Index of the actual section if this is a Section node
    pub section_index: Option<usize>,
}

impl TreeNode {
    /// Create a directory node
    #[must_use]
    pub fn directory(name: String, path: String, tree_level: usize) -> Self {
        Self {
            node_type: NodeType::Directory { name, path },
            tree_level,
            navigable: false,
            section_index: None,
        }
    }

    /// Create a file node
    #[must_use]
    pub fn file(name: String, path: String, tree_level: usize) -> Self {
        Self {
            node_type: NodeType::File { name, path },
            tree_level,
            navigable: false,
            section_index: None,
        }
    }

    /// Create a section node
    #[must_use]
    pub fn section(section: Section, tree_level: usize, section_index: usize) -> Self {
        Self {
            node_type: NodeType::Section(section),
            tree_level,
            navigable: true,
            section_index: Some(section_index),
        }
    }
}
