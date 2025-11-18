mod apply;
mod debug;
mod hunk;

pub use apply::{apply_diff, apply_diff_restore};
pub use hunk::DiffHunk;

#[cfg(debug_assertions)]
use debug::debug_hunk_lines;
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
    debug_hunk_lines(before, after, &hunks);

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
