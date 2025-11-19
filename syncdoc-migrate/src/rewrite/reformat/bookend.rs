//! Bookending operation for reformatting macro attributes

use crate::syncdoc_debug;

const TRIGGER_STRINGS: &[&str] = &["syncdoc::module_doc!", "module_doc!"];

/// Applies bookend reformatting to lines containing trigger strings
pub fn reformat_bookended_lines(code: &str) -> String {
    syncdoc_debug!("\n=== BOOKEND DEBUG START ===");
    syncdoc_debug!("Input code length: {}", code.len());

    let result = code
        .lines()
        .enumerate()
        .map(|(idx, line)| {
            syncdoc_debug!("\nLine {}: {:?}", idx, line);

            if needs_bookending(line) {
                syncdoc_debug!("  -> NEEDS BOOKENDING");
                match reformat_line(line) {
                    Some(reformatted) => {
                        syncdoc_debug!("  -> Reformatted to: {:?}", reformatted);
                        reformatted
                    }
                    None => {
                        syncdoc_debug!("  -> FAILED TO REFORMAT");
                        line.to_string()
                    }
                }
            } else {
                syncdoc_debug!("  -> No bookending needed");
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    syncdoc_debug!("\nFinal result length: {}", result.len());
    syncdoc_debug!("=== BOOKEND DEBUG END ===\n");

    result
}

/// Checks if line needs bookending (starts with #! and has trigger)
pub(crate) fn needs_bookending(line: &str) -> bool {
    let trimmed = line.trim_start();
    syncdoc_debug!("    Checking needs_bookending:");
    syncdoc_debug!("      trimmed: {:?}", trimmed);

    let no_spaces = trimmed.replace(' ', "");
    syncdoc_debug!("      no_spaces: {:?}", no_spaces);

    if !no_spaces.starts_with("#!") {
        syncdoc_debug!("      -> Does NOT start with #!");
        return false;
    }
    syncdoc_debug!("      -> Starts with #!");

    let has_trigger = TRIGGER_STRINGS.iter().any(|trigger| {
        let contains = no_spaces.contains(trigger);
        syncdoc_debug!("      -> Checking trigger {:?}: {}", trigger, contains);
        contains
    });

    syncdoc_debug!("      -> Has trigger: {}", has_trigger);
    has_trigger
}

/// Extracts content between #![ and ] for reformatting
pub(crate) fn extract_bookend_content(line: &str) -> Option<String> {
    syncdoc_debug!("    extract_bookend_content:");
    let trimmed = line.trim_start();
    syncdoc_debug!("      trimmed: {:?}", trimmed);

    // Remove all spaces to find the pattern reliably
    let no_spaces = trimmed.replace(' ', "");
    syncdoc_debug!("      no_spaces: {:?}", no_spaces);

    let start = no_spaces.find("#![")?;
    syncdoc_debug!("      Found #![ at position: {}", start);

    let content_start = start + 3;
    let end = no_spaces.rfind(']')?;
    syncdoc_debug!("      Found ] at position: {}", end);

    if content_start >= end {
        syncdoc_debug!("      ERROR: content_start >= end");
        return None;
    }

    let content = no_spaces[content_start..end].trim().to_string();
    syncdoc_debug!("      Extracted content: {:?}", content);
    Some(content)
}

/// Reformats a single line via bookending
pub(crate) fn reformat_line(line: &str) -> Option<String> {
    syncdoc_debug!("    reformat_line for: {:?}", line);

    let content = extract_bookend_content(line)?;
    syncdoc_debug!("    Got content: {:?}", content);

    let bookended = create_bookended_expr(&content);
    syncdoc_debug!("    Created bookended expr: {:?}", bookended);

    // Format via rustfmt
    let formatted = super::rustfmt(&bookended).ok()?;
    syncdoc_debug!("    Rustfmt output: {:?}", formatted);

    let stripped = strip_bookends(&formatted)?;
    syncdoc_debug!("    Stripped bookends: {:?}", stripped);

    let result = reconstruct_inner_attr(&stripped);
    syncdoc_debug!("    Final result: {:?}", result);

    Some(result)
}

/// Wraps content in const expression for rustfmt
pub(crate) fn create_bookended_expr(content: &str) -> String {
    format!("const _: i32 = {{ {} }};", content)
}

/// Strips const wrapper after rustfmt
pub(crate) fn strip_bookends(formatted: &str) -> Option<String> {
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
pub(crate) fn reconstruct_inner_attr(content: &str) -> String {
    format!("#![{}]", content)
}
