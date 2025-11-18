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

    // IMPORTANT: Separate module-level hunks from item-level hunks
    // Module-level hunks should be processed first to ensure #![doc = module_doc!()]
    // appears before any other inner attributes
    let (mut module_hunks, mut item_hunks): (Vec<_>, Vec<_>) = split_hunks
        .into_iter()
        .partition(|h| is_module_level_hunk(h, &after_lines));

    // Sort module hunks by position to maintain order
    module_hunks.sort_by_key(|h| h.before_start);
    item_hunks.sort_by_key(|h| h.before_start);

    // Combine: module hunks first, then item hunks
    let ordered_hunks: Vec<_> = module_hunks.into_iter().chain(item_hunks).collect();

    #[cfg(debug_assertions)]
    debug_hunk_lines(original, formatted_after, &ordered_hunks);

    let mut result: Vec<&'a str> = Vec::new();
    let mut orig_idx = 0;

    for h in &ordered_hunks {
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
        } else {
            #[cfg(debug_assertions)]
            crate::syncdoc_debug!(
                "APPLYING relevant hunk at lines {}..{} (adds {} lines, removes {} lines)",
                h.before_start,
                h.before_start + h.before_count,
                h.after_count,
                h.before_count
            );
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

        // Add new lines from after (but skip non-doc attributes we've already preserved)
        for i in h.after_start..h.after_start + h.after_count {
            if i < after_lines.len() {
                let line = after_lines[i];

                // Skip non-doc attribute lines from transformed version
                // since we've already preserved them from the original
                if should_skip_from_transformed(line) {
                    continue;
                }

                result.push(line);
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
            if (trimmed.starts_with("#[") && !no_spaces.starts_with("#[doc") && !no_spaces.contains("omnidoc"))
                // Preserve any INNER attribute line that's NOT a doc attribute
                || (no_spaces.starts_with("#![") && !no_spaces.starts_with("#![doc"))
                // Also preserve regular comments (not doc comments)
                || (trimmed.starts_with("//") && !trimmed.starts_with("///") && !trimmed.starts_with("//!"))
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

/// Check if a line from the transformed version should be skipped
/// because it's a non-doc attribute that we've already preserved from original
fn should_skip_from_transformed(line: &str) -> bool {
    let trimmed = line.trim_start();

    // Remove ALL spaces to handle rustfmt'd attributes like "# [facet (...)]"
    let no_spaces = trimmed.replace(" ", "");

    // Skip OUTER non-doc attributes (already preserved from original)
    // Must check no_spaces version since rustfmt may add spaces: "# [facet" -> "#[facet"
    if no_spaces.starts_with("#[")
        && !no_spaces.starts_with("#[doc")
        && !no_spaces.contains("omnidoc")
        && !no_spaces.contains("syncdoc::omnidoc")
    {
        return true;
    }

    // Skip INNER non-doc attributes (already preserved from original)
    if no_spaces.starts_with("#![") && !no_spaces.starts_with("#![doc") {
        return true;
    }

    // Skip regular comments (already preserved from original)
    if trimmed.starts_with("//") && !trimmed.starts_with("///") && !trimmed.starts_with("//!") {
        return true;
    }

    false
}

/// Checks if a hunk contains module-level documentation (inner attributes)
fn is_module_level_hunk(hunk: &DiffHunk, after_lines: &[&str]) -> bool {
    let after_end = hunk.after_start + hunk.after_count;

    for i in hunk.after_start..after_end {
        if i < after_lines.len() {
            let line = after_lines[i].replace(" ", "");

            // Check for module-level doc attributes
            if line.starts_with("#![doc") || line.contains("module_doc!") {
                return true;
            }
        }
    }

    false
}
