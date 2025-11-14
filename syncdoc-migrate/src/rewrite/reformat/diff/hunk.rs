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
    // Check lines being removed
    for i in 0..hunk.before_count {
        let idx = hunk.before_start + i;
        if idx < original_lines.len() {
            let line = original_lines[idx].trim();
            if line.starts_with("///")
                || line.starts_with("//!")
                || line.replace(" ", "").contains("#[doc")
                || line.replace(" ", "").contains("#![doc")
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
            if line.starts_with("///")
                || line.starts_with("//!")
                || line.replace(" ", "").contains("#[doc")
                || line.replace(" ", "").contains("#![doc")
                || line.replace(" ", "").contains("#[syncdoc::")
                || line.replace(" ", "").contains("#[omnidoc")
            {
                return true;
            }
        }
    }

    false
}

/// Splits a hunk if it contains both module-level and item-level doc changes
pub fn split_hunk_if_mixed(hunk: &DiffHunk, after_lines: &[&str]) -> Vec<DiffHunk> {
    let after_end = hunk.after_start + hunk.after_count;

    // Find if there's a module doc line followed by item doc line
    let mut module_doc_end = None;

    for i in hunk.after_start..after_end {
        if i >= after_lines.len() {
            break;
        }

        let line = after_lines[i].replace(" ", "");

        // If this is a module doc
        if line.starts_with("#![") || line.starts_with("#!{") {
            // Check if there's a non-blank, non-module-doc line after
            for j in (i + 1)..after_end {
                if j >= after_lines.len() {
                    break;
                }

                let next_line = after_lines[j];
                if next_line.trim().is_empty() {
                    continue; // Skip blank lines
                }

                let next_trimmed = next_line.replace(" ", "");
                // If we find an item-level attribute, split here
                if next_trimmed.starts_with("#[") {
                    module_doc_end = Some(i + 1); // Split after the module doc line
                    break;
                }

                break; // Found non-blank, non-attribute line
            }

            if module_doc_end.is_some() {
                break;
            }
        }
    }

    if let Some(split_point) = module_doc_end {
        let lines_in_first = split_point - hunk.after_start;

        vec![
            DiffHunk {
                before_start: hunk.before_start,
                before_count: lines_in_first,
                after_start: hunk.after_start,
                after_count: lines_in_first,
            },
            DiffHunk {
                before_start: hunk.before_start + lines_in_first,
                before_count: hunk.before_count - lines_in_first,
                after_start: split_point,
                after_count: hunk.after_count - lines_in_first,
            },
        ]
    } else {
        vec![hunk.clone()]
    }
}
