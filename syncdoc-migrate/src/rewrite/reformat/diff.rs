//! Line-level diff computation and application using imara-diff

use imara_diff::{Algorithm, Diff, InternedInput};

/// Represents a change hunk in the diff
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub before_start: usize,
    pub before_count: usize,
    pub after_start: usize,
    pub after_count: usize,
}

/// Computes line-level diff between before and after
pub fn compute_line_diff(before: &str, after: &str) -> Vec<DiffHunk> {
    let input = InternedInput::new(before, after);

    let mut diff = Diff::compute(Algorithm::Histogram, &input);
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

/// Checks if a hunk is related to documentation changes
fn is_doc_related_hunk(hunk: &DiffHunk, original_lines: &[&str], after_lines: &[&str]) -> bool {
    // Check lines being removed
    for i in 0..hunk.before_count {
        let idx = hunk.before_start + i;
        if idx < original_lines.len() {
            let line = original_lines[idx].trim();
            if line.starts_with("///")
                || line.starts_with("//!")
                || line.replace(" ", "").contains("#[doc")
                || line.replace(" ", "").contains("#![doc")
            {
                return true;
            }
        }
    }

    // Check lines being added
    let after_end = hunk.after_start + hunk.after_count;
    for i in hunk.after_start..after_end {
        if i < after_lines.len() {
            let line = after_lines[i].trim();
            if line.starts_with("///")
                || line.starts_with("//!")
                || line.replace(" ", "").contains("#[doc")
                || line.replace(" ", "").contains("#![doc")
                || line.replace(" ", "").contains("#[syncdoc::")
                || line.replace(" ", "").contains("#[omnidoc")
            {
                return true;
            }
        }
    }

    false
}

/// Splits a hunk if it contains both module-level and item-level doc changes
fn split_hunk_if_mixed(hunk: &DiffHunk, after_lines: &[&str]) -> Vec<DiffHunk> {
    let after_end = hunk.after_start + hunk.after_count;

    // Find if there's a module doc line followed by item doc line
    let mut module_doc_end = None;

    for i in hunk.after_start..after_end {
        if i >= after_lines.len() {
            break;
        }

        let line = after_lines[i].replace(" ", "");

        // If this is a module doc
        if line.starts_with("#![") || line.starts_with("#!{") {
            // Check if there's a non-blank, non-module-doc line after
            for j in (i + 1)..after_end {
                if j >= after_lines.len() {
                    break;
                }

                let next_line = after_lines[j];
                if next_line.trim().is_empty() {
                    continue; // Skip blank lines
                }

                let next_trimmed = next_line.replace(" ", "");
                // If we find an item-level attribute, split here
                if next_trimmed.starts_with("#[") {
                    module_doc_end = Some(i + 1); // Split after the module doc line
                    break;
                }

                break; // Found non-blank, non-attribute line
            }

            if module_doc_end.is_some() {
                break;
            }
        }
    }

    if let Some(split_point) = module_doc_end {
        let lines_in_first = split_point - hunk.after_start;

        vec![
            DiffHunk {
                before_start: hunk.before_start,
                before_count: lines_in_first,
                after_start: hunk.after_start,
                after_count: lines_in_first,
            },
            DiffHunk {
                before_start: hunk.before_start + lines_in_first,
                before_count: hunk.before_count - lines_in_first,
                after_start: split_point,
                after_count: hunk.after_count - lines_in_first,
            },
        ]
    } else {
        vec![hunk.clone()]
    }
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

    for (_hunk_num, hunk) in split_hunks.iter().enumerate() {
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
            for _ in 0..removed_blank_lines {
                result.push("");
            }
        }

        // PRESERVE REGULAR COMMENT-ONLY LINES that would be deleted
        // But NOT doc comments (/// or //!)
        for i in 0..hunk.before_count {
            let idx = hunk.before_start + i;
            if idx < original_lines.len() {
                let line = original_lines[idx];
                let trimmed = line.trim_start();

                // Check if this is a REGULAR comment line (not doc comment)
                // Must start with // but NOT /// or //!
                if trimmed.starts_with("//")
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
            for _ in 0..removed_blank_lines {
                result.push("");
            }
        }
    }

    // Copy remaining unchanged lines from original
    while orig_idx < original_lines.len() {
        result.push(original_lines[orig_idx]);
        orig_idx += 1;
    }

    result.join("\n")
}

#[cfg(test)]
mod diff_tests;
