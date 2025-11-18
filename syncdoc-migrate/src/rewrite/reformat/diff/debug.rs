#![cfg(debug_assertions)]
use super::hunk::DiffHunk;
use crate::syncdoc_debug;

/// Debug a list of hunks, showing before and after snippets.
pub fn debug_hunk_lines(before: &str, after: &str, hunks: &[DiffHunk]) {
    let before_lines: Vec<&str> = before.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();

    syncdoc_debug!("=== HUNK DEBUG ===");
    syncdoc_debug!("Before lines: {}", before_lines.len());
    syncdoc_debug!("After lines: {}", after_lines.len());
    syncdoc_debug!("Hunks: {}", hunks.len());

    for (i, hunk) in hunks.iter().enumerate() {
        syncdoc_debug!(
            "Hunk {}: before[{}..{}] -> after[{}..{}]",
            i,
            hunk.before_start,
            hunk.before_start + hunk.before_count,
            hunk.after_start,
            hunk.after_start + hunk.after_count
        );

        syncdoc_debug!("  Before snippet:");
        for j in hunk.before_start..hunk.before_start + hunk.before_count {
            if j < before_lines.len() {
                syncdoc_debug!("    [{}]: {:?}", j, before_lines[j]);
            }
        }

        syncdoc_debug!("  After snippet:");
        for j in hunk.after_start..hunk.after_start + hunk.after_count {
            if j < after_lines.len() {
                syncdoc_debug!("    [{}]: {:?}", j, after_lines[j]);
            }
        }
    }

    syncdoc_debug!("==================");
}
