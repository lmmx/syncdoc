#[cfg(debug_assertions)]
use super::debug::debug_hunk_lines;
use super::hunk::{self, is_doc_related_hunk, split_hunk_if_mixed, DiffHunk};
use super::strip_all_doc_attr_bookends;

/// Applies a set of diff hunks to the original source, filtering hunks by a relevance predicate,
/// and then post-processing the resulting string using the provided function.
fn apply_hunks<'a, RelevanceFn, PostProcessFn>(
    original: &'a str,
    hunks: &[DiffHunk],
    formatted_after: &'a str,
    is_relevant: RelevanceFn,
    post_process: PostProcessFn,
) -> String
where
    RelevanceFn: Fn(&DiffHunk, &[&'a str], &[&'a str]) -> bool,
    PostProcessFn: Fn(String) -> String,
{
    let original_lines: Vec<&'a str> = original.lines().collect();
    let after_lines: Vec<&'a str> = formatted_after.lines().collect();

    // Split mixed hunks first
    let mut split_hunks = Vec::new();
    for h in hunks {
        split_hunks.extend(split_hunk_if_mixed(h, &after_lines));
    }

    #[cfg(debug_assertions)]
    debug_hunk_lines(original, formatted_after, &split_hunks);

    let mut result: Vec<&'a str> = Vec::new();
    let mut orig_idx = 0;

    for h in &split_hunks {
        // ONLY apply relevant hunks (doc-related or restore-related)
        if !is_relevant(h, &original_lines, &after_lines) {
            #[cfg(debug_assertions)]
            crate::syncdoc_debug!(
                "Skipping irrelevant hunk at lines {}..{}",
                h.before_start,
                h.before_start + h.before_count
            );

            copy_original_lines(
                &original_lines,
                &mut result,
                &mut orig_idx,
                h.before_start,
                h.before_count,
            );
            continue;
        }

        // Copy unchanged lines from original up to hunk start
        copy_original_lines(
            &original_lines,
            &mut result,
            &mut orig_idx,
            h.before_start,
            0,
        );

        // Check if we're removing blank lines
        let removed_blank_lines =
            count_blank_lines(&original_lines, h.before_start, h.before_count);

        // Check if the new content is a module docstring (starts with #!)
        let is_module_doc = h.after_count > 0
            && h.after_start < after_lines.len()
            && after_lines[h.after_start]
                .replace(" ", "")
                .starts_with("#!");

        // For module docstrings, preserve blank lines AFTER
        // For everything else, preserve blank lines BEFORE
        if removed_blank_lines > 0 && !is_module_doc {
            result.extend(std::iter::repeat_n("", removed_blank_lines));
        }

        // PRESERVE ALL NON-DOC ATTRIBUTE LINES that would be deleted
        // This includes #[derive], #[cfg], #[facet], etc.
        preserve_non_doc_lines(&original_lines, &mut result, h.before_start, h.before_count);

        // Skip removed lines in original
        orig_idx += h.before_count;

        // Add new lines from after
        for i in h.after_start..h.after_start + h.after_count {
            if i < after_lines.len() {
                result.push(after_lines[i]);
            }
        }

        // For module docstrings, preserve blank lines AFTER
        if removed_blank_lines > 0 && is_module_doc {
            result.extend(std::iter::repeat_n("", removed_blank_lines));
        }
    }

    // Copy remaining unchanged lines from original
    copy_original_lines(
        &original_lines,
        &mut result,
        &mut orig_idx,
        original_lines.len(),
        0,
    );

    post_process(result.join("\n"))
}

fn copy_original_lines<'a>(
    original: &[&'a str],
    result: &mut Vec<&'a str>,
    orig_idx: &mut usize,
    up_to: usize,
    count: usize,
) {
    let end = up_to + count;
    while *orig_idx < end && *orig_idx < original.len() {
        result.push(original[*orig_idx]);
        *orig_idx += 1;
    }
}

fn count_blank_lines(original: &[&str], start: usize, count: usize) -> usize {
    (0..count)
        .filter(|i| {
            let idx = start + i;
            idx < original.len() && original[idx].trim().is_empty()
        })
        .count()
}

fn preserve_non_doc_lines<'a>(
    original: &[&'a str],
    result: &mut Vec<&'a str>,
    start: usize,
    count: usize,
) {
    for i in 0..count {
        let idx = start + i;
        if idx < original.len() {
            let line = original[idx];
            let trimmed = line.trim_start();
            let no_spaces = trimmed.replace(" ", "");

            // Preserve any OUTER attribute line that's NOT a doc attribute
            if trimmed.starts_with("#[")
                && !no_spaces.starts_with("#[doc")
                && !no_spaces.contains("omnidoc")
            {
                result.push(line);
            }
            // Preserve any INNER attribute line that's NOT a doc attribute
            else if no_spaces.starts_with("#![") && !no_spaces.starts_with("#![doc") {
                result.push(line);
            }
            // Also preserve regular comments (not doc comments)
            else if trimmed.starts_with("//")
                && !trimmed.starts_with("///")
                && !trimmed.starts_with("//!")
            {
                result.push(line);
            }
        }
    }
}

/// Applies only doc-related hunks to the original source.
pub fn apply_diff(original: &str, hunks: &[DiffHunk], formatted_after: &str) -> String {
    apply_hunks(original, hunks, formatted_after, is_doc_related_hunk, |s| s)
}

/// Applies hunks relevant for restore operations and strips doc attribute bookends.
pub fn apply_diff_restore(original: &str, hunks: &[DiffHunk], formatted_after: &str) -> String {
    apply_hunks(
        original,
        hunks,
        formatted_after,
        hunk::is_restore_related_hunk,
        |s| strip_all_doc_attr_bookends(&s),
    )
}
