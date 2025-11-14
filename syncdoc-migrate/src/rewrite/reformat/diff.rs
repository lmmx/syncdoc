//! Line-level diff computation and application using imara-diff

use imara_diff::{Algorithm, Diff, InternedInput};

/// Represents a change hunk in the diff
/// (Keep your API type, but it now reflects the built-in hunk format)
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

    let mut diff = Diff::compute(Algorithm::Myers, &input);
    diff.postprocess_lines(&input);

    diff.hunks()
        .map(|h| DiffHunk {
            before_start: h.before.start as usize,
            before_count: h.before.len() as usize,
            after_start: h.after.start as usize,
            after_count: h.after.len() as usize,
        })
        .collect()
}

/// Applies diff hunks to original source
pub fn apply_diff(original: &str, hunks: &[DiffHunk], formatted_after: &str) -> String {
    let original_lines: Vec<&str> = original.lines().collect();
    let after_lines: Vec<&str> = formatted_after.lines().collect();

    let mut result = Vec::new();
    let mut orig_idx = 0;

    for hunk in hunks {
        // Copy unchanged lines up to hunk start
        while orig_idx < hunk.before_start {
            result.push(original_lines[orig_idx]);
            orig_idx += 1;
        }

        // Skip removed lines in original
        orig_idx += hunk.before_count;

        // Add new lines from the formatted "after"
        let end = hunk.after_start + hunk.after_count;
        for i in hunk.after_start..end {
            if let Some(line) = after_lines.get(i) {
                result.push(line);
            }
        }
    }

    // Copy remaining unchanged lines
    while orig_idx < original_lines.len() {
        result.push(original_lines[orig_idx]);
        orig_idx += 1;
    }

    result.join("\n")
}

#[cfg(test)]
mod diff_tests;
