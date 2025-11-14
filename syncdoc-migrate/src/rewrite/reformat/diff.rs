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

/// Applies diff hunks to original source
pub fn apply_diff(original: &str, hunks: &[DiffHunk], formatted_after: &str) -> String {
    let original_lines: Vec<&str> = original.lines().collect();
    let after_lines: Vec<&str> = formatted_after.lines().collect();

    let mut result = Vec::new();
    let mut orig_idx = 0;

    for (_hunk_num, hunk) in hunks.iter().enumerate() {
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
