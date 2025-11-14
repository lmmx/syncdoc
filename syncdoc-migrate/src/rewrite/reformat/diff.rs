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

    #[cfg(debug_assertions)]
    {
        eprintln!("=== APPLY DEBUG ===");
        eprintln!("Original has {} lines", original_lines.len());
        eprintln!("After has {} lines", after_lines.len());
        eprintln!("Applying {} hunks", hunks.len());
    }

    let mut result = Vec::new();
    let mut orig_idx = 0;

    for (_hunk_num, hunk) in hunks.iter().enumerate() {
        #[cfg(debug_assertions)]
        eprintln!("Processing hunk {}: orig_idx={}", _hunk_num, orig_idx);

        // Copy unchanged lines from original up to hunk start
        while orig_idx < hunk.before_start {
            if orig_idx < original_lines.len() {
                result.push(original_lines[orig_idx]);
                #[cfg(debug_assertions)]
                eprintln!(
                    "  Copy original[{}]: {:?}",
                    orig_idx, original_lines[orig_idx]
                );
            }
            orig_idx += 1;
        }

        // Skip removed lines in original
        #[cfg(debug_assertions)]
        eprintln!("  Skipping {} lines from original", hunk.before_count);
        orig_idx += hunk.before_count;

        // Add new lines from after
        let after_end = hunk.after_start + hunk.after_count;
        #[cfg(debug_assertions)]
        eprintln!(
            "  Adding lines from after[{}..{}]",
            hunk.after_start, after_end
        );

        for i in hunk.after_start..after_end {
            if i < after_lines.len() {
                result.push(after_lines[i]);
                #[cfg(debug_assertions)]
                eprintln!("    Add after[{}]: {:?}", i, after_lines[i]);
            }
        }
    }

    // Copy remaining unchanged lines from original
    #[cfg(debug_assertions)]
    eprintln!("Copying remaining lines from orig_idx={}", orig_idx);

    while orig_idx < original_lines.len() {
        result.push(original_lines[orig_idx]);
        #[cfg(debug_assertions)]
        eprintln!(
            "  Copy original[{}]: {:?}",
            orig_idx, original_lines[orig_idx]
        );
        orig_idx += 1;
    }

    #[cfg(debug_assertions)]
    {
        eprintln!("Result has {} lines", result.len());
        eprintln!("==================");
    }

    result.join("\n")
}

#[cfg(test)]
mod diff_tests;
