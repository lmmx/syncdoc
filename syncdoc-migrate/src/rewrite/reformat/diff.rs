//! Line-level diff computation and application using imara-diff

mod hunk;

use crate::syncdoc_debug;
pub use hunk::{is_doc_related_hunk, split_hunk_if_mixed, DiffHunk}; // is_restore_related_hunk,

use imara_diff::{Algorithm, Diff, InternedInput};

/// Computes line-level diff between before and after
pub fn compute_line_diff(before: &str, after: &str) -> Vec<DiffHunk> {
    let input = InternedInput::new(before, after);

    let mut diff = Diff::compute(Algorithm::Myers, &input);
    diff.postprocess_lines(&input);

    let hunks: Vec<_> = diff
        .hunks()
        .map(|h| DiffHunk {
            before_start: h.before.start as usize,
            before_count: h.before.len(),
            after_start: h.after.start as usize,
            after_count: h.after.len(),
        })
        .collect();

    #[cfg(debug_assertions)]
    {
        eprintln!("=== DIFF DEBUG ===");
        eprintln!("Before lines: {}", before.lines().count());
        eprintln!("After lines: {}", after.lines().count());
        eprintln!("Hunks: {}", hunks.len());
        for (i, hunk) in hunks.iter().enumerate() {
            eprintln!(
                "Hunk {}: before[{}..{}] -> after[{}..{}]",
                i,
                hunk.before_start,
                hunk.before_start + hunk.before_count,
                hunk.after_start,
                hunk.after_start + hunk.after_count
            );
        }
        eprintln!("==================");
    }

    hunks
}

/// Applies diff hunks to original source
pub fn apply_diff(original: &str, hunks: &[DiffHunk], formatted_after: &str) -> String {
    let original_lines: Vec<&str> = original.lines().collect();
    let after_lines: Vec<&str> = formatted_after.lines().collect();

    // Split mixed hunks first
    let mut split_hunks = Vec::new();
    for hunk in hunks {
        split_hunks.extend(split_hunk_if_mixed(hunk, &after_lines));
    }

    #[cfg(debug_assertions)]
    {
        eprintln!("=== SPLIT HUNKS ===");
        eprintln!("Original hunks: {}", hunks.len());
        eprintln!("After splitting: {}", split_hunks.len());
        for (i, hunk) in split_hunks.iter().enumerate() {
            eprintln!(
                "Split hunk {}: before[{}..{}] -> after[{}..{}]",
                i,
                hunk.before_start,
                hunk.before_start + hunk.before_count,
                hunk.after_start,
                hunk.after_start + hunk.after_count
            );
        }
        eprintln!("===================");
    }

    let mut result = Vec::new();
    let mut orig_idx = 0;

    for hunk in split_hunks.iter() {
        // ONLY apply doc-related hunks
        if !is_doc_related_hunk(hunk, &original_lines, &after_lines) {
            #[cfg(debug_assertions)]
            eprintln!(
                "Skipping non-doc hunk at lines {}..{}",
                hunk.before_start,
                hunk.before_start + hunk.before_count
            );

            // Skip this hunk - copy original lines unchanged
            while orig_idx < hunk.before_start + hunk.before_count {
                if orig_idx < original_lines.len() {
                    result.push(original_lines[orig_idx]);
                }
                orig_idx += 1;
            }
            continue;
        }

        // Copy unchanged lines from original up to hunk start
        while orig_idx < hunk.before_start {
            if orig_idx < original_lines.len() {
                result.push(original_lines[orig_idx]);
            }
            orig_idx += 1;
        }

        // Check if we're removing blank lines
        let removed_blank_lines = (0..hunk.before_count)
            .filter(|i| {
                let idx = hunk.before_start + i;
                idx < original_lines.len() && original_lines[idx].trim().is_empty()
            })
            .count();

        // Check if the new content is a module docstring (starts with #!)
        let is_module_doc = hunk.after_count > 0
            && hunk.after_start < after_lines.len()
            && after_lines[hunk.after_start]
                .replace(" ", "")
                .starts_with("#!");

        // For module docstrings, preserve blank lines AFTER
        // For everything else, preserve blank lines BEFORE
        if removed_blank_lines > 0 && !is_module_doc {
            result.extend(std::iter::repeat_n("", removed_blank_lines))
        }

        // PRESERVE ALL NON-DOC ATTRIBUTE LINES that would be deleted
        // This includes #[derive], #[cfg], #[facet], etc.
        for i in 0..hunk.before_count {
            let idx = hunk.before_start + i;
            if idx < original_lines.len() {
                let line = original_lines[idx];
                let trimmed = line.trim_start();
                let no_spaces = trimmed.replace(" ", "");

                // Preserve any OUTER attribute line that's NOT a doc attribute
                if trimmed.starts_with("#[")
                    && !no_spaces.starts_with("#[doc")
                    && !no_spaces.contains("omnidoc")
                {
                    result.push(line);
                }
                // Preserve any INNER attribute line that's NOT a doc attribute
                else if no_spaces.starts_with("#![") && !no_spaces.starts_with("#![doc") {
                    result.push(line);
                }
                // Also preserve regular comments (not doc comments)
                else if trimmed.starts_with("//")
                    && !trimmed.starts_with("///")
                    && !trimmed.starts_with("//!")
                {
                    result.push(line);
                }
            }
        }

        // Skip removed lines in original
        orig_idx += hunk.before_count;

        // Add new lines from after
        let after_end = hunk.after_start + hunk.after_count;
        for i in hunk.after_start..after_end {
            if i < after_lines.len() {
                result.push(after_lines[i]);
            }
        }

        // For module docstrings, preserve blank lines AFTER
        if removed_blank_lines > 0 && is_module_doc {
            result.extend(std::iter::repeat_n("", removed_blank_lines))
        }
    }

    // Copy remaining unchanged lines from original
    while orig_idx < original_lines.len() {
        result.push(original_lines[orig_idx]);
        orig_idx += 1;
    }

    result.join("\n")
}

/// Strips doc attribute bookends from a line if present
/// Converts `#[doc = "//! text"]` -> `//! text`
/// Converts `#[doc = "/// text"]` -> `/// text`
fn strip_doc_attr_bookends(line: &str) -> String {
    let trimmed = line.trim();

    // Check for #[doc = "//! ..."] pattern
    if let Some(start) = trimmed.find(r#"#[doc = "//!"#) {
        if let Some(end) = trimmed.rfind(r#""]"#) {
            let content_start = start + r#"#[doc = ""#.len();
            if content_start < end {
                let content = &trimmed[content_start..end];
                let indent = &line[..line.len() - line.trim_start().len()];
                // If content is just a space after //!, output just //!
                if content == "//! " {
                    return format!("{}//!", indent);
                }
                return format!("{}{}", indent, content);
            }
        }
    }

    // Check for #[doc = "/// ..."] pattern
    if let Some(start) = trimmed.find(r#"#[doc = "///"#) {
        if let Some(end) = trimmed.rfind(r#""]"#) {
            let content_start = start + r#"#[doc = ""#.len();
            if content_start < end {
                let content = &trimmed[content_start..end];
                let indent = &line[..line.len() - line.trim_start().len()];
                // If content is just a space after ///, output just ///
                if content == "/// " {
                    return format!("{}///", indent);
                }
                return format!("{}{}", indent, content);
            }
        }
    }

    line.to_string()
}

fn strip_all_doc_attr_bookends(code: &str) -> String {
    code.lines()
        .map(|line| strip_doc_attr_bookends(line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Applies diff hunks to original source for restore operations
pub fn apply_diff_restore(original: &str, hunks: &[DiffHunk], formatted_after: &str) -> String {
    let original_lines: Vec<&str> = original.lines().collect();
    let after_lines: Vec<&str> = formatted_after.lines().collect();

    let mut split_hunks = Vec::new();
    for hunk in hunks {
        split_hunks.extend(split_hunk_if_mixed(hunk, &after_lines));
    }

    #[cfg(debug_assertions)]
    {
        eprintln!("=== RESTORE SPLIT HUNKS ===");
        eprintln!("Original hunks: {}", hunks.len());
        eprintln!("After splitting: {}", split_hunks.len());
        for (i, hunk) in split_hunks.iter().enumerate() {
            eprintln!(
                "Split hunk {}: before[{}..{}] -> after[{}..{}]",
                i,
                hunk.before_start,
                hunk.before_start + hunk.before_count,
                hunk.after_start,
                hunk.after_start + hunk.after_count
            );
        }
        eprintln!("===========================");
    }

    let mut result = Vec::new();
    let mut orig_idx = 0;

    for hunk in split_hunks.iter() {
        let is_restore = hunk::is_restore_related_hunk(hunk, &original_lines, &after_lines);

        syncdoc_debug!("\n=== HUNK DEBUG ===");
        syncdoc_debug!(
            "Hunk: before[{}..{}] -> after[{}..{}]",
            hunk.before_start,
            hunk.before_start + hunk.before_count,
            hunk.after_start,
            hunk.after_start + hunk.after_count
        );
        syncdoc_debug!("Is restore related: {}", is_restore);
        syncdoc_debug!("\nBEFORE lines:");
        for i in 0..hunk.before_count {
            let idx = hunk.before_start + i;
            if idx < original_lines.len() {
                syncdoc_debug!("  [{}]: {:?}", idx, original_lines[idx]);
            }
        }
        syncdoc_debug!("\nAFTER lines:");
        for i in hunk.after_start..hunk.after_start + hunk.after_count {
            if i < after_lines.len() {
                syncdoc_debug!("  [{}]: {:?}", i, after_lines[i]);
            }
        }
        syncdoc_debug!("==================\n");

        if !is_restore {
            syncdoc_debug!(
                "Skipping non-restore hunk at lines {}..{}",
                hunk.before_start,
                hunk.before_start + hunk.before_count
            );

            while orig_idx < hunk.before_start + hunk.before_count {
                if orig_idx < original_lines.len() {
                    result.push(original_lines[orig_idx]);
                }
                orig_idx += 1;
            }
            continue;
        }

        while orig_idx < hunk.before_start {
            if orig_idx < original_lines.len() {
                result.push(original_lines[orig_idx]);
            }
            orig_idx += 1;
        }

        let removed_blank_lines = (0..hunk.before_count)
            .filter(|i| {
                let idx = hunk.before_start + i;
                idx < original_lines.len() && original_lines[idx].trim().is_empty()
            })
            .count();

        let is_module_doc = hunk.after_count > 0
            && hunk.after_start < after_lines.len()
            && after_lines[hunk.after_start].trim().starts_with("//!");

        if removed_blank_lines > 0 && !is_module_doc {
            result.extend(std::iter::repeat_n("", removed_blank_lines))
        }

        for i in 0..hunk.before_count {
            let idx = hunk.before_start + i;
            if idx < original_lines.len() {
                let line = original_lines[idx];
                let trimmed = line.trim_start();

                if trimmed.starts_with("//")
                    && !trimmed.starts_with("///")
                    && !trimmed.starts_with("//!")
                {
                    result.push(line);
                }
            }
        }

        orig_idx += hunk.before_count;

        let after_end = hunk.after_start + hunk.after_count;
        for i in hunk.after_start..after_end {
            if i < after_lines.len() {
                result.push(after_lines[i]);
            }
        }

        if removed_blank_lines > 0 && is_module_doc {
            result.extend(std::iter::repeat_n("", removed_blank_lines))
        }
    }

    while orig_idx < original_lines.len() {
        result.push(original_lines[orig_idx]);
        orig_idx += 1;
    }

    let joined = result.join("\n");
    strip_all_doc_attr_bookends(&joined)
}

#[cfg(test)]
mod diff_tests;
