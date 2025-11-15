//! Format-preserving code rewriting using rustfmt and line-level diffs

mod bookend;
mod diff;

use bookend::reformat_bookended_lines;
use diff::{apply_diff, compute_line_diff};
use duct::cmd;

/// Rewrites code while preserving original formatting where possible
///
/// This function applies transformations (strip/annotate) and then uses
/// rustfmt to normalize both the original and transformed code. It computes
/// a line-level diff and applies only the changed lines to preserve the
/// original formatting for unchanged code.
pub fn rewrite_preserving_format(original: &str, transformed: &str) -> Result<String, String> {
    #[cfg(debug_assertions)]
    {
        eprintln!("\n=== REFORMAT START ===");
        eprintln!("Original length: {}", original.len());
        eprintln!("Transformed length: {}", transformed.len());
    }

    // 1. Format both strings with rustfmt
    let formatted_original = rustfmt(original)?;
    let formatted_transformed = rustfmt(transformed)?;

    #[cfg(debug_assertions)]
    {
        eprintln!("Formatted original length: {}", formatted_original.len());
        eprintln!(
            "Formatted transformed length: {}",
            formatted_transformed.len()
        );
        eprintln!("\n--- Formatted Original ---");
        eprintln!("{}", formatted_original);
        eprintln!("\n--- Formatted Transformed ---");
        eprintln!("{}", formatted_transformed);
    }

    // 2. Compute line-level diff
    let diff_hunks = compute_line_diff(&formatted_original, &formatted_transformed);

    // 3. Apply diff to FORMATTED original (not raw original)
    // This ensures line numbers match
    let diff_result = apply_diff(&formatted_original, &diff_hunks, &formatted_transformed);

    // 4. Reformat bookended lines in transformed code
    let mut result = reformat_bookended_lines(&diff_result);

    #[cfg(debug_assertions)]
    eprintln!("After bookending: {}", transformed.len());

    // Ensure EOF newline
    if !result.ends_with('\n') {
        result.push('\n');
    }

    #[cfg(debug_assertions)]
    {
        eprintln!("\n--- Final Result ---");
        eprintln!("{}", result);
        eprintln!("=== REFORMAT END ===\n");
    }

    Ok(result)
}

/// Formats Rust code using rustfmt
fn rustfmt(code: &str) -> Result<String, String> {
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
