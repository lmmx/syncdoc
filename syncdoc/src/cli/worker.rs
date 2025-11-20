use super::args::Args;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use syncdoc_migrate::{
    extract_all_docs, find_expected_doc_paths, parse_file as syncdoc_parse_file, restore_file,
    rewrite_file, DocExtract, DocsPathMode, ParsedFile,
};

/// Enum to represent the result of processing a single file
#[derive(Debug)]
pub enum ProcessResult {
    Migrated {
        extracts: Vec<DocExtract>,
        rewritten: bool,
        touched: usize,
    },
    Restored {
        dry_run: bool,
    },
    NoChange,
    Error(String),
}

/// Helper macro for verbose logging, expecting the last argument(s) in braces
macro_rules! vlog {
    ($args:expr, { $($arg:tt)* }) => {
        if $args.verbose {
            eprintln!($($arg)*);
        }
    };
}

/// Helper macro for conditional verbose logging, expecting the last argument(s) in braces
macro_rules! vlog_if {
    ($args:expr, $cond:expr, { $($arg:tt)* }) => {
        if $args.verbose && $cond {
            eprintln!($($arg)*);
        }
    };
}

/// === Step 2: Parse the file ===
fn parse_file_with_error_handling(
    file_path: &Path,
    args: &Args,
) -> Result<ParsedFile, ProcessResult> {
    match syncdoc_parse_file(file_path) {
        Ok(p) => Ok(p),
        Err(e) => {
            let error_msg = format!("Failed to parse {}: {}", file_path.display(), e);
            vlog!(args, {"  Warning: {}", error_msg});
            Err(ProcessResult::Error(error_msg))
        }
    }
}

/// === Step 3: RESTORE MODE ===
/// If restore is requested, we early exit after restoring (equivalent to `continue`)
fn handle_restore(
    file_path: &Path,
    parsed: &ParsedFile,
    args: &Args,
    docs_root: &str,
) -> Option<ProcessResult> {
    if !args.restore {
        return None;
    }

    if let Some(restored) = restore_file(parsed, docs_root) {
        if args.dry_run {
            vlog!(args, {"  Would restore: {}", file_path.display()});
            Some(ProcessResult::Restored { dry_run: true })
        } else {
            match fs::write(file_path, restored) {
                Ok(_) => {
                    vlog!(args, {"  Restored: {}", file_path.display()});
                    Some(ProcessResult::Restored { dry_run: false })
                }
                Err(e) => Some(ProcessResult::Error(format!(
                    "Failed to write {}: {}",
                    file_path.display(),
                    e
                ))),
            }
        }
    } else {
        // No restoration needed
        Some(ProcessResult::NoChange)
    }
}

/// === Step 5: Touch missing files if needed ===
/// === Step 6: Rewrite source file if requested ===
fn handle_touch_and_rewrite(
    file_path: &Path,
    parsed: &ParsedFile,
    args: &Args,
    docs_root: &str,
    docs_mode: DocsPathMode,
    all_extracts: &mut Vec<DocExtract>,
) -> Result<(bool, usize), ProcessResult> {
    let mut touched_count = 0;

    // === Step 5: Touch missing files if needed ===
    if args.touch && args.annotate {
        // Determine expected doc paths
        let expected_paths = find_expected_doc_paths(parsed, docs_root);

        vlog!(args, {"  Found {} expected doc path(s)", expected_paths.len()});

        // Filter paths: exclude those already in extracts and those already on disk
        let existing_paths: HashSet<_> = all_extracts.iter().map(|e| &e.markdown_path).collect();

        let missing: Vec<_> = expected_paths
            .into_iter()
            .filter(|extract| {
                !existing_paths.contains(&extract.markdown_path) && !extract.markdown_path.exists()
            })
            .collect();

        vlog_if!(args, !missing.is_empty(), {"  Will touch {} missing file(s)", missing.len()});

        touched_count = missing.len();
        all_extracts.extend(missing);
    }

    // === Step 6: Rewrite source file if requested ===
    let rewritten = if args.strip_docs || args.annotate {
        if let Some(rewritten) =
            rewrite_file(parsed, docs_root, docs_mode, args.strip_docs, args.annotate)
        {
            if args.dry_run {
                vlog!(args, {"  Would rewrite: {}", file_path.display()});
                true
            } else {
                match fs::write(file_path, rewritten) {
                    Ok(_) => {
                        vlog!(args, {"  Rewrote: {}", file_path.display()});
                        true
                    }
                    Err(e) => {
                        return Err(ProcessResult::Error(format!(
                            "Failed to write {}: {}",
                            file_path.display(),
                            e
                        )));
                    }
                }
            }
        } else {
            false
        }
    } else {
        false
    };

    Ok((rewritten, touched_count))
}

/// Process a single file (called by worker threads)
///
/// Preserves all steps: parsing, RESTORE mode, MIGRATION mode, touching missing files,
/// and rewriting the source file if requested. Comments explicitly mirror the sequential version.
pub fn sync(
    file_path: &Path,
    args: &Args,
    docs_root: &str,
    docs_mode: DocsPathMode,
) -> ProcessResult {
    vlog!(args, {"Processing: {}", file_path.display()});
    let parsed = match parse_file_with_error_handling(file_path, args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    // RESTORE MODE: Put documentation back into docstrings from markdown
    if let Some(result) = handle_restore(file_path, &parsed, args, docs_root) {
        return result; // early exit
    }
    // MIGRATION MODE: Extract documentation from the parsed file
    let mut extracts = extract_all_docs(&parsed, docs_root);

    vlog_if!(args, !extracts.is_empty(), {"  Extracted {} doc(s)", extracts.len()});

    // === Step 5 & 6: Touch missing files and rewrite source file ===
    let (rewritten, touched) = match handle_touch_and_rewrite(
        file_path,
        &parsed,
        args,
        docs_root,
        docs_mode,
        &mut extracts,
    ) {
        Ok(result) => result,
        Err(e) => return e,
    };

    // Return full migration result
    ProcessResult::Migrated {
        extracts,
        rewritten,
        touched,
    }
}
