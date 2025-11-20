//! Format-preserving code rewriting using rustfmt and line-level diffs

pub(crate) mod bookend;
pub(crate) mod diff;

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
        crate::syncdoc_debug!("\n=== REFORMAT START ===");
        crate::syncdoc_debug!("Original length: {}", original.len());
        crate::syncdoc_debug!("Transformed length: {}", transformed.len());
    }

    // 1. Format both strings with rustfmt
    let formatted_original = rustfmt(original)?;
    let formatted_transformed = rustfmt(transformed)?;

    #[cfg(debug_assertions)]
    {
        crate::syncdoc_debug!("Formatted original length: {}", formatted_original.len());
        crate::syncdoc_debug!(
            "Formatted transformed length: {}",
            formatted_transformed.len()
        );
        crate::syncdoc_debug!("\n--- Formatted Original ---");
        crate::syncdoc_debug!("{}", formatted_original);
        crate::syncdoc_debug!("\n--- Formatted Transformed ---");
        crate::syncdoc_debug!("{}", formatted_transformed);
    }

    // 2. Compute line-level diff
    let diff_hunks = compute_line_diff(&formatted_original, &formatted_transformed);

    #[cfg(debug_assertions)]
    {
        crate::syncdoc_debug!("DEBUG restore: Found {} hunks", diff_hunks.len());
        for (i, h) in diff_hunks.iter().enumerate() {
            crate::syncdoc_debug!(
                "  Hunk {}: before[{}..{}] after[{}..{}]",
                i,
                h.before_start,
                h.before_start + h.before_count,
                h.after_start,
                h.after_start + h.after_count
            );
        }
    }

    // 3. Apply diff to FORMATTED original (not raw original)
    // This ensures line numbers match
    let diff_result = apply_diff(&formatted_original, &diff_hunks, &formatted_transformed);

    #[cfg(debug_assertions)]
    {
        crate::syncdoc_debug!("Original != Result: {}", original != diff_result);
        if original == diff_result {
            crate::syncdoc_debug!("WARNING: Restore produced no changes for this file!");
        }
    }

    // 4. Reformat bookended lines in transformed code
    let mut result = reformat_bookended_lines(&diff_result);

    #[cfg(debug_assertions)]
    crate::syncdoc_debug!("After bookending: {}", transformed.len());

    // Ensure EOF newline
    if !result.ends_with('\n') {
        result.push('\n');
    }

    #[cfg(debug_assertions)]
    {
        crate::syncdoc_debug!("\n--- Final Result ---");
        crate::syncdoc_debug!("{}", result);
        crate::syncdoc_debug!("=== REFORMAT END ===\n");
    }

    Ok(result)
}

/// Formats Rust code using rustfmt
pub(crate) fn rustfmt(code: &str) -> Result<String, String> {
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

/// Rewrites code for restore operations while preserving original formatting
///
/// This is the inverse of the forward migration - it converts omnidoc attributes
/// back to inline doc comments while preserving formatting.
pub fn rewrite_preserving_format_restore(
    original: &str,
    transformed: &str,
) -> Result<String, String> {
    #[cfg(debug_assertions)]
    {
        crate::syncdoc_debug!("\n=== RESTORE REFORMAT START ===");
        crate::syncdoc_debug!("Original length: {}", original.len());
        crate::syncdoc_debug!("Transformed length: {}", transformed.len());
    }

    let formatted_original = rustfmt(original).map_err(|e| {
        eprintln!("ERROR: rustfmt failed on original: {}", e);
        e
    })?;
    let formatted_transformed = rustfmt(transformed).map_err(|e| {
        eprintln!("ERROR: rustfmt failed on transformed: {}", e);
        eprintln!("Transformed code was:");
        eprintln!("{}", transformed);
        e
    })?;

    #[cfg(debug_assertions)]
    {
        crate::syncdoc_debug!("Formatted original length: {}", formatted_original.len());
        crate::syncdoc_debug!(
            "Formatted transformed length: {}",
            formatted_transformed.len()
        );
    }

    let diff_hunks = compute_line_diff(&formatted_original, &formatted_transformed);

    #[cfg(debug_assertions)]
    {
        crate::syncdoc_debug!("DEBUG restore: Found {} hunks", diff_hunks.len());
        for (i, h) in diff_hunks.iter().enumerate() {
            crate::syncdoc_debug!(
                "  Hunk {}: before[{}..{}] after[{}..{}]",
                i,
                h.before_start,
                h.before_start + h.before_count,
                h.after_start,
                h.after_start + h.after_count
            );
        }
    }

    let diff_result =
        diff::apply_diff_restore(&formatted_original, &diff_hunks, &formatted_transformed);

    let mut result = reformat_bookended_lines(&diff_result);

    if !result.ends_with('\n') {
        result.push('\n');
    }

    #[cfg(debug_assertions)]
    {
        crate::syncdoc_debug!("=== RESTORE REFORMAT END ===\n");
    }

    Ok(result)
}
