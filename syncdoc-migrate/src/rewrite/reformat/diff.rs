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

    // Handle inner module-level docs: #![doc = "//! text"]
    if let Some(start) = trimmed.find(r#"#![doc = "//!"#) {
        if let Some(end) = trimmed.rfind(r#""]"#) {
            let content_start = start + r#"#![doc = ""#.len();
            if content_start < end {
                let escaped_content = &trimmed[content_start..end];
                let indent = &line[..line.len() - line.trim_start().len()];

                // UNESCAPE the content
                let unescaped = unescape_doc_content(escaped_content);

                return if unescaped == "//! " {
                    format!("{}//!", indent)
                } else {
                    format!("{}{}", indent, unescaped)
                };
            }
        }
    }

    // Handle outer module-level docs: #[doc = "//! text"] (fallback)
    if let Some(start) = trimmed.find(r#"#[doc = "//!"#) {
        if let Some(end) = trimmed.rfind(r#""]"#) {
            let content_start = start + r#"#[doc = ""#.len();
            if content_start < end {
                let escaped_content = &trimmed[content_start..end];
                let indent = &line[..line.len() - line.trim_start().len()];

                let unescaped = unescape_doc_content(escaped_content);

                return if unescaped == "//! " {
                    format!("{}//!", indent)
                } else {
                    format!("{}{}", indent, unescaped)
                };
            }
        }
    }

    // Handle item-level docs: #[doc = "/// text"]
    if let Some(start) = trimmed.find(r#"#[doc = "///"#) {
        if let Some(end) = trimmed.rfind(r#""]"#) {
            let content_start = start + r#"#[doc = ""#.len();
            if content_start < end {
                let escaped_content = &trimmed[content_start..end];
                let indent = &line[..line.len() - line.trim_start().len()];

                let unescaped = unescape_doc_content(escaped_content);

                return if unescaped == "/// " {
                    format!("{}///", indent)
                } else {
                    format!("{}{}", indent, unescaped)
                };
            }
        }
    }

    line.to_string()
}

/// Unescapes Rust string content (the reverse of what the compiler does)
fn unescape_doc_content(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek() {
                Some(&'"') => {
                    chars.next();
                    result.push('"');
                }
                Some(&'\\') => {
                    chars.next();
                    result.push('\\');
                }
                Some(&'n') => {
                    chars.next();
                    result.push('\n');
                }
                Some(&'r') => {
                    chars.next();
                    result.push('\r');
                }
                Some(&'t') => {
                    chars.next();
                    result.push('\t');
                }
                _ => {
                    result.push(ch);
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn strip_all_doc_attr_bookends(code: &str) -> String {
    code.lines()
        .map(strip_doc_attr_bookends)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
#[path = "../../tests/diff.rs"]
mod diff_tests;
