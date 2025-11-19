//! The edit plan manages document modifications using textum patches.
//!
//! This module defines the transformation that work in the TUI manifests as actual edits on disk.
//! asterism uses textum for generic line-based patching that works with any text format.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use textum::{Boundary, BoundaryMode, Patch, PatchSet, Snippet, Target};

/// Serialisable collection of file modifications for atomic application.
#[derive(Serialize, Deserialize, Clone)]
pub struct EditPlan {
    /// Individual section replacements grouped for batch processing.
    pub edits: Vec<Edit>,
}

/// Precise coordinates and content for replacing a section in a file.
#[derive(Serialize, Deserialize, Clone)]
pub struct Edit {
    /// Target file path for this modification.
    pub file_name: String,
    /// First line of the section to replace (inclusive).
    pub line_start: i64,
    /// Final line of the section to replace (exclusive).
    pub line_end: i64,
    /// Starting column of the section header.
    pub column_start: i64,
    /// Ending column of the section header.
    pub column_end: i64,
    /// New content replacing the section body.
    pub section_content: String,
    /// Section title for tracking and debugging edits.
    pub item_name: String,
}

impl EditPlan {
    /// Apply all edits in the plan using textum patches.
    ///
    /// Groups edits by file and uses textum's `PatchSet` to apply all changes
    /// atomically per file. Each edit targets a line range and replaces the
    /// content between those lines with the new section content.
    ///
    /// # Errors
    ///
    /// Returns an error if file operations, patching, or line number conversion fails.
    pub fn apply(&mut self) -> io::Result<()> {
        let mut file_groups: HashMap<String, Vec<&Edit>> = HashMap::new();

        for edit in &self.edits {
            file_groups
                .entry(edit.file_name.clone())
                .or_default()
                .push(edit);
        }

        for (file_name, edits) in file_groups {
            let mut patchset = PatchSet::new();

            for edit in edits {
                let line_start: usize = edit.line_start.try_into().map_err(|_| {
                    io::Error::other(format!("Invalid line_start: {}", edit.line_start))
                })?;
                let line_end: usize = edit.line_end.try_into().map_err(|_| {
                    io::Error::other(format!("Invalid line_end: {}", edit.line_end))
                })?;

                let start = Boundary::new(Target::Line(line_start), BoundaryMode::Include);
                let end = Boundary::new(Target::Line(line_end), BoundaryMode::Exclude);
                let snippet = Snippet::Between { start, end };

                let replacement = format!("\n{}\n\n", edit.section_content.trim());

                let patch = Patch {
                    file: file_name.clone(),
                    snippet,
                    replacement,
                };

                patchset.add(patch);
            }

            let results = patchset
                .apply_to_files()
                .map_err(|e| io::Error::other(e.to_string()))?;

            if let Some(new_content) = results.get(&file_name) {
                std::fs::write(&file_name, new_content)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/edit_plan.rs"]
mod tests;
