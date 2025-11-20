//! Line-level diff computation and application using imara-diff

/// Represents a change hunk in the diff
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub before_start: usize,
    pub before_count: usize,
    pub after_start: usize,
    pub after_count: usize,
}

/// Checks if a hunk is related to documentation changes
pub fn is_doc_related_hunk(hunk: &DiffHunk, original_lines: &[&str], after_lines: &[&str]) -> bool {
    // Check lines being removed - ONLY actual doc comments/attributes
    for i in 0..hunk.before_count {
        let idx = hunk.before_start + i;
        if idx < original_lines.len() {
            let line = original_lines[idx].trim();
            let no_spaces = line.replace(" ", "");

            // Only match actual doc comments and doc-specific attributes
            if line.starts_with("///")
                || line.starts_with("//!")
                || no_spaces.starts_with("#[doc=")
                || no_spaces.starts_with("#![doc=")
            {
                return true;
            }
        }
    }

    // Check lines being added
    let after_end = hunk.after_start + hunk.after_count;
    for i in hunk.after_start..after_end {
        if i < after_lines.len() {
            let line = after_lines[i].trim();
            let no_spaces = line.replace(" ", "");

            // Only match doc-related additions
            if line.starts_with("///")
                || line.starts_with("//!")
                || no_spaces.starts_with("#[doc=")
                || no_spaces.starts_with("#![doc=")
                || no_spaces.starts_with("#[syncdoc::omnidoc")
                || no_spaces.starts_with("#[omnidoc")
                || (no_spaces.contains("module_doc!") && no_spaces.contains("#![doc="))
            {
                return true;
            }
        }
    }

    false
}

// /// Splits a hunk that contains both module-level (#!) and item-level (#[) documentation.
// ///
// /// Axiom: In restore operations, a module_doc!() macro (1 line) expands to N doc attributes,
// /// and an omnidoc attribute (1 line) expands to 1 doc attribute. A blank line between them
// /// in the original must be preserved in the output.
// fn split_mixed_restore_hunk(hunk: &DiffHunk, after_lines: &[&str]) -> Option<(DiffHunk, DiffHunk)> {
//     // Find the boundary: last module doc line before first item doc line
//     let mut last_module_doc_idx = None;
//
//     for i in hunk.after_start..(hunk.after_start + hunk.after_count) {
//         if i >= after_lines.len() {
//             break;
//         }
//
//         let line = after_lines[i].replace(" ", "");
//         if line.starts_with("#![doc") {
//             last_module_doc_idx = Some(i);
//         } else if line.starts_with("#[doc") {
//             // Found item doc - split point is after the last module doc we saw
//             if let Some(last_mod) = last_module_doc_idx {
//                 let split_after = last_mod + 1; // Split AFTER last module doc
//
//                 // In the before side: module_doc!() + blank line = first 2 lines
//                 // The omnidoc starts at line 2
//                 let before_split = 2;
//
//                 if before_split <= hunk.before_count {
//                     return Some((
//                         DiffHunk {
//                             before_start: hunk.before_start,
//                             before_count: before_split,
//                             after_start: hunk.after_start,
//                             after_count: split_after - hunk.after_start,
//                         },
//                         DiffHunk {
//                             before_start: hunk.before_start + before_split,
//                             before_count: hunk.before_count - before_split,
//                             after_start: split_after,
//                             after_count: hunk.after_count - (split_after - hunk.after_start),
//                         },
//                     ));
//                 }
//             }
//             break;
//         }
//     }
//
//     None
// }
//
// pub fn split_hunk_if_mixed(hunk: &DiffHunk, after_lines: &[&str]) -> Vec<DiffHunk> {
//     if hunk.after_count == 0 || hunk.before_count == 0 {
//         return vec![hunk.clone()];
//     }
//
//     if let Some((first, second)) = split_mixed_restore_hunk(hunk, after_lines) {
//         vec![first, second]
//     } else {
//         vec![hunk.clone()]
//     }
// }

/// Splits a hunk if it contains both module-level and item-level doc changes
pub fn split_hunk_if_mixed(hunk: &DiffHunk, after_lines: &[&str]) -> Vec<DiffHunk> {
    #[cfg(debug_assertions)]
    {
        crate::syncdoc_debug!("\n=== SPLIT_HUNK_IF_MIXED ===");
        crate::syncdoc_debug!(
            "Hunk: before[{}..{}] after[{}..{}]",
            hunk.before_start,
            hunk.before_start + hunk.before_count,
            hunk.after_start,
            hunk.after_start + hunk.after_count
        );
        crate::syncdoc_debug!(
            "after_count={}, before_count={}",
            hunk.after_count,
            hunk.before_count
        );
    }

    // Don't try to split deletion-only or insertion-only hunks
    if hunk.after_count == 0 || hunk.before_count == 0 {
        #[cfg(debug_assertions)]
        crate::syncdoc_debug!("SKIP: Deletion or insertion only hunk");
        return vec![hunk.clone()];
    }

    let after_end = hunk.after_start + hunk.after_count;

    #[cfg(debug_assertions)]
    crate::syncdoc_debug!("Checking lines {} to {}", hunk.after_start, after_end);

    // Find if there's a module doc line followed by item doc line
    let mut module_doc_end = None;

    for i in hunk.after_start..after_end {
        if i >= after_lines.len() {
            break;
        }

        let line = after_lines[i].replace(" ", "");

        #[cfg(debug_assertions)]
        crate::syncdoc_debug!("  Line {}: {:?}", i, after_lines[i]);

        // If this is a module doc
        if line.starts_with("#![") || line.starts_with("#!{") {
            #[cfg(debug_assertions)]
            crate::syncdoc_debug!("    -> Found module doc line");

            // Check if there's a non-blank, non-module-doc line after
            for j in (i + 1)..after_end {
                if j >= after_lines.len() {
                    break;
                }

                let next_line = after_lines[j];

                #[cfg(debug_assertions)]
                crate::syncdoc_debug!("    Checking next line {}: {:?}", j, next_line);

                if next_line.trim().is_empty() {
                    #[cfg(debug_assertions)]
                    crate::syncdoc_debug!("      -> Blank, skipping");
                    continue; // Skip blank lines
                }

                let next_trimmed = next_line.replace(" ", "");

                #[cfg(debug_assertions)]
                crate::syncdoc_debug!("      -> next_trimmed: {:?}", next_trimmed);

                // If we find an item-level attribute, split here
                if next_trimmed.starts_with("#[") {
                    #[cfg(debug_assertions)]
                    crate::syncdoc_debug!("      -> FOUND ITEM DOC! Setting split point to {}", j);

                    module_doc_end = Some(j); // Split AT the item doc line, not after module doc
                    break;
                }

                break; // Found non-blank, non-attribute line
            }

            if module_doc_end.is_some() {
                break;
            }
        }
    }

    #[cfg(debug_assertions)]
    crate::syncdoc_debug!("module_doc_end = {:?}", module_doc_end);

    if let Some(split_point) = module_doc_end {
        let after_lines_in_first = split_point - hunk.after_start;

        #[cfg(debug_assertions)]
        crate::syncdoc_debug!(
            "Attempting to split at line {} (after_lines_in_first={})",
            split_point,
            after_lines_in_first
        );

        // Determine the split point for the "before" side of the hunk.
        //
        // In restore operations, the "before" side contains compressed macro syntax
        // (e.g., `#![doc = module_doc!()]` + blank line + `#[omnidoc]`) while the
        // "after" side contains expanded doc attributes. The expansion is asymmetric:
        // one module_doc!() macro becomes N module doc attributes.
        //
        // We split the "before" side to separate module-level changes from item-level
        // changes. The module doc macro and its trailing blank line (lines 0-1) go in
        // the first hunk; the omnidoc attribute (line 2+) goes in the second hunk.
        //
        // This count must not exceed the actual lines available in the hunk. If it
        // would, the hunk structure doesn't match our assumptions about syncdoc's
        // generated code, so we abort the split to avoid creating invalid hunks.

        // Note: we can rely on this hardcoded line 2 because we control the migration we restore
        // from. If syncdoc is used differently, this will break the expectation (...TODO)

        // Before has: module_doc!() at line 0, blank at line 1, omnidoc at line 2
        // So split before at line 2 (after module_doc + blank)
        let before_lines_in_first = if hunk.before_count >= 2 { 2 } else { 1 };

        #[cfg(debug_assertions)]
        crate::syncdoc_debug!(
            "before_lines_in_first={}, before_count={}",
            before_lines_in_first,
            hunk.before_count
        );

        // SAFETY CHECK: Ensure we have enough lines in before
        if before_lines_in_first > hunk.before_count {
            #[cfg(debug_assertions)]
            crate::syncdoc_debug!(
                "CANNOT SPLIT: before_lines_in_first ({}) > before_count ({})",
                before_lines_in_first,
                hunk.before_count
            );

            return vec![hunk.clone()];
        }

        #[cfg(debug_assertions)]
        crate::syncdoc_debug!("SPLITTING into 2 hunks");

        vec![
            DiffHunk {
                before_start: hunk.before_start,
                before_count: before_lines_in_first,
                after_start: hunk.after_start,
                after_count: after_lines_in_first,
            },
            DiffHunk {
                before_start: hunk.before_start + before_lines_in_first,
                before_count: hunk.before_count - before_lines_in_first,
                after_start: split_point,
                after_count: hunk.after_count - after_lines_in_first,
            },
        ]
    } else {
        #[cfg(debug_assertions)]
        crate::syncdoc_debug!("No split point found, returning original hunk");

        vec![hunk.clone()]
    }
}

/// Checks if a hunk is related to restore operations (removing omnidoc, adding doc comments)
pub fn is_restore_related_hunk(
    hunk: &DiffHunk,
    original_lines: &[&str],
    after_lines: &[&str],
) -> bool {
    let removes_omnidoc = (0..hunk.before_count).any(|i| {
        let idx = hunk.before_start + i;
        idx < original_lines.len() && {
            let line = original_lines[idx].replace(" ", "");
            line.contains("#[omnidoc")
                || line.contains("#[syncdoc::omnidoc")
                || line.contains("#![doc=syncdoc::module_doc!")
        }
    });

    let adds_docs = (hunk.after_start..hunk.after_start + hunk.after_count).any(|i| {
        i < after_lines.len() && {
            let line = after_lines[i].trim();
            // Check for both direct doc comments AND doc attributes from restore
            line.starts_with("///")
                || line.starts_with("//!")
                || line.contains(r#"#[doc = "///"#)
                || line.contains(r#"#[doc = "//!"#)
        }
    });

    removes_omnidoc || adds_docs
}
