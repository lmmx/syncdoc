//! Format-preserving code rewriting using rustfmt and line-level diffs

mod bookend;
mod diff;

/// Rewrites code while preserving original formatting where possible
///
/// This function applies transformations (strip/annotate) and then uses
/// rustfmt to normalize both the original and transformed code. It computes
/// a line-level diff and applies only the changed lines to preserve the
/// original formatting for unchanged code.
pub fn rewrite_preserving_format(original: &str, transformed: &str) -> Result<String, String> {
    // 1. Reformat bookended lines in transformed code
    let transformed = bookend::reformat_bookended_lines(transformed);

    // 2. Format both strings with rustfmt
    let formatted_original = rustfmt(&original)?;
    let formatted_transformed = rustfmt(&transformed)?;

    // 3. Compute line-level diff
    let diff_hunks = diff::compute_line_diff(&formatted_original, &formatted_transformed);

    // 4. Apply diff to original source
    Ok(diff::apply_diff(
        &original,
        &diff_hunks,
        &formatted_transformed,
    ))
}

/// Formats Rust code using rustfmt
fn rustfmt(code: &str) -> Result<String, String> {
    use duct::cmd;

    cmd!("rustfmt", "--emit=stdout")
        .stdin_bytes(code.as_bytes())
        .stdout_capture()
        .stderr_null()
        .run()
        .map_err(|e| format!("rustfmt failed: {}", e))
        .and_then(|output| {
            String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8: {}", e))
        })
}

#[cfg(test)]
mod reformat_tests;
