mod apply;
mod hunk;

use crate::syncdoc_debug;
pub use apply::{apply_diff, apply_diff_restore};
pub use hunk::DiffHunk;

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
        syncdoc_debug!("=== DIFF DEBUG ===");
        syncdoc_debug!("Before lines: {}", before.lines().count());
        syncdoc_debug!("After lines: {}", after.lines().count());
        syncdoc_debug!("Hunks: {}", hunks.len());

        let before_lines: Vec<&str> = before.lines().collect();
        let after_lines: Vec<&str> = after.lines().collect();

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

    hunks
}

/// Strips doc attribute bookends from a line if present
/// Converts `#[doc = "//! text"]` -> `//! text`
/// Converts `#[doc = "/// text"]` -> `/// text`
fn strip_doc_attr_bookends(line: &str) -> String {
    let trimmed = line.trim();
    if let Some(start) = trimmed.find(r#"#[doc = "//!"#) {
        if let Some(end) = trimmed.rfind(r#""]"#) {
            let content_start = start + r#"#[doc = ""#.len();
            if content_start < end {
                let content = &trimmed[content_start..end];
                let indent = &line[..line.len() - line.trim_start().len()];
                return if content == "//! " {
                    format!("{}//!", indent)
                } else {
                    format!("{}{}", indent, content)
                };
            }
        }
    }
    if let Some(start) = trimmed.find(r#"#[doc = "///"#) {
        if let Some(end) = trimmed.rfind(r#""]"#) {
            let content_start = start + r#"#[doc = ""#.len();
            if content_start < end {
                let content = &trimmed[content_start..end];
                let indent = &line[..line.len() - line.trim_start().len()];
                return if content == "/// " {
                    format!("{}///", indent)
                } else {
                    format!("{}{}", indent, content)
                };
            }
        }
    }
    line.to_string()
}

fn strip_all_doc_attr_bookends(code: &str) -> String {
    code.lines()
        .map(strip_doc_attr_bookends)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod diff_tests;
