//! Bookending operation for reformatting macro attributes

const TRIGGER_STRINGS: &[&str] = &["syncdoc::module_doc!", "module_doc!"];

/// Applies bookend reformatting to lines containing trigger strings
pub fn reformat_bookended_lines(code: &str) -> String {
    code.lines()
        .map(|line| {
            if needs_bookending(line) {
                reformat_line(line).unwrap_or_else(|| line.to_string())
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Checks if line needs bookending (starts with #! and has trigger)
fn needs_bookending(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("#!") {
        return false;
    }

    let no_spaces = trimmed.replace(' ', "");
    TRIGGER_STRINGS
        .iter()
        .any(|trigger| no_spaces.contains(trigger))
}

/// Extracts content between #![ and ] for reformatting
fn extract_bookend_content(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let start = trimmed.find("#![")?;
    let content_start = start + 3;
    let end = trimmed.rfind(']')?;

    if content_start >= end {
        return None;
    }

    Some(trimmed[content_start..end].trim().to_string())
}

/// Wraps content in const expression for rustfmt
fn create_bookended_expr(content: &str) -> String {
    format!("const _: i32 = {{ {} }};", content)
}

/// Strips const wrapper after rustfmt
fn strip_bookends(formatted: &str) -> Option<String> {
    let trimmed = formatted.trim();
    let prefix = "const _: i32 = { ";
    let suffix = " };";

    if !trimmed.starts_with(prefix) || !trimmed.ends_with(suffix) {
        return None;
    }

    let content = &trimmed[prefix.len()..trimmed.len() - suffix.len()];
    Some(content.to_string())
}

/// Reconstructs #![ ... ] with reformatted content
fn reconstruct_inner_attr(content: &str) -> String {
    format!("#![{}]", content)
}

/// Reformats a single line via bookending
fn reformat_line(line: &str) -> Option<String> {
    let content = extract_bookend_content(line)?;
    let bookended = create_bookended_expr(&content);

    // Format via rustfmt
    let formatted = super::rustfmt(&bookended).ok()?;
    let stripped = strip_bookends(&formatted)?;

    Some(reconstruct_inner_attr(&stripped))
}

#[cfg(test)]
mod bookend_tests;
