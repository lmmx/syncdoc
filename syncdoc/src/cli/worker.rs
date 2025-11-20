use super::args::Args;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use syncdoc_migrate::{
    extract_all_docs, find_expected_doc_paths, parse_file, restore_file, rewrite_file,
    DocExtraction, DocsPathMode,
};

/// Enum to represent the result of processing a single file
#[derive(Debug)]
pub enum ProcessResult {
    Migrated {
        extractions: Vec<DocExtraction>,
        rewritten: bool,
        touched: usize,
    },
    Restored {
        dry_run: bool,
    },
    NoChange,
    Error(String),
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
    // === Step 1: Verbose info about file being processed ===
    if args.verbose {
        eprintln!("Processing: {}", file_path.display());
    }

    // === Step 2: Parse the file ===
    let parsed = match parse_file(file_path) {
        Ok(p) => p,
        Err(e) => {
            let error_msg = format!("Failed to parse {}: {}", file_path.display(), e);
            if args.verbose {
                eprintln!("  Warning: {}", error_msg);
            }
            return ProcessResult::Error(error_msg);
        }
    };

    // === Step 3: RESTORE MODE ===
    // If restore is requested, we early exit after restoring (equivalent to `continue`)
    if args.restore {
        if let Some(restored) = restore_file(&parsed, docs_root) {
            if args.dry_run {
                if args.verbose {
                    eprintln!("  Would restore: {}", file_path.display());
                }
                return ProcessResult::Restored { dry_run: true };
            } else {
                match fs::write(file_path, restored) {
                    Ok(_) => {
                        if args.verbose {
                            eprintln!("  Restored: {}", file_path.display());
                        }
                        return ProcessResult::Restored { dry_run: false };
                    }
                    Err(e) => {
                        return ProcessResult::Error(format!(
                            "Failed to write {}: {}",
                            file_path.display(),
                            e
                        ));
                    }
                }
            }
        } else {
            // No restoration needed
            return ProcessResult::NoChange;
        }
    }

    // === Step 4: MIGRATION MODE ===
    // Extract documentation from the parsed file
    let extractions = extract_all_docs(&parsed, docs_root);

    if args.verbose && !extractions.is_empty() {
        eprintln!("  Found {} doc extraction(s)", extractions.len());
    }

    let mut all_extractions = extractions;

    // === Step 5: Touch missing files if needed ===
    if args.touch && args.annotate {
        // Determine expected doc paths
        let expected_paths = find_expected_doc_paths(&parsed, docs_root);

        if args.verbose {
            eprintln!("  Found {} expected doc path(s)", expected_paths.len());
        }

        // Filter paths: exclude those already in extractions and those already on disk
        let existing_paths: HashSet<_> = all_extractions.iter().map(|e| &e.markdown_path).collect();

        let missing_paths: Vec<_> = expected_paths
            .into_iter()
            .filter(|extraction| {
                !existing_paths.contains(&extraction.markdown_path)
                    && !extraction.markdown_path.exists()
            })
            .collect();

        if !missing_paths.is_empty() && args.verbose {
            eprintln!("  Will touch {} missing file(s)", missing_paths.len());
        }

        let touched_count = missing_paths.len();
        all_extractions.extend(missing_paths);

        // === Step 6: Rewrite source file if requested ===
        let rewritten = if args.strip_docs || args.annotate {
            if let Some(rewritten) = rewrite_file(
                &parsed,
                docs_root,
                docs_mode,
                args.strip_docs,
                args.annotate,
            ) {
                if args.dry_run {
                    if args.verbose {
                        eprintln!("  Would rewrite: {}", file_path.display());
                    }
                    true
                } else {
                    match fs::write(file_path, rewritten) {
                        Ok(_) => {
                            if args.verbose {
                                eprintln!("  Rewrote: {}", file_path.display());
                            }
                            true
                        }
                        Err(e) => {
                            return ProcessResult::Error(format!(
                                "Failed to write {}: {}",
                                file_path.display(),
                                e
                            ));
                        }
                    }
                }
            } else {
                false
            }
        } else {
            false
        };

        // Return full migration result
        return ProcessResult::Migrated {
            extractions: all_extractions,
            rewritten,
            touched: touched_count,
        };
    }

    // If touch mode was not requested, return migrated result with 0 touched files
    ProcessResult::Migrated {
        extractions: all_extractions,
        rewritten: false,
        touched: 0,
    }
}
