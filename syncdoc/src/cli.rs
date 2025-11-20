//! syncdoc: Procedural macro attributes to inject documentation from external files
//!
//! Command-line interface for migrating Rust documentation to markdown files.
//!
//! `syncdoc` provides a migration tool that extracts documentation from Rust source files
//! and writes them to markdown files in a structured directory. It can optionally strip
//! the original doc comments and annotate items with `#[omnidoc]` attributes.
//!
//! The tool discovers all Rust files in a source directory, extracts their documentation,
//! and organizes the markdown files by module path and item name.
#![allow(clippy::multiple_crate_versions)]

/// Command-line interface for migrating documentation to markdown.
#[cfg(feature = "cli")]
pub mod cli {
    mod args;
    mod worker;

    use args::{print_usage, Args};
    use worker::{sync, ProcessResult};

    use std::io;
    use std::path::Path;
    use syncdoc_migrate::{
        discover_rust_files, get_or_create_docs_path, write_extractions, DocsPathMode,
    };

    /// Entry point for the `syncdoc` command-line interface.
    ///
    /// Migrates Rust documentation to markdown files.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if:
    /// - command-line argument parsing fails,
    /// - the source directory cannot be read,
    /// - files cannot be parsed,
    /// - or writing files fails.
    ///
    /// The process will also exit with a non-zero status if migration fails.
    pub fn main() -> io::Result<()> {
        let mut args: Args = facet_args::from_std_args()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("{e}")))?;

        // --migrate implies --cut + --add + --touch
        args.strip_docs = args.strip_docs || args.migrate;
        args.annotate = args.annotate || args.migrate;
        args.touch = args.touch || args.migrate;

        if args.help {
            print_usage();
            std::process::exit(0);
        }

        // Restore is a mutually exclusive operation with migrate/cut/add
        if args.restore && (args.migrate || args.strip_docs || args.annotate) {
            eprintln!("Error: --restore cannot be used with --migrate, --cut, or --add");
            std::process::exit(1);
        }

        let source_path = Path::new(&args.source);
        if !source_path.exists() {
            eprintln!("Error: Source path does not exist: {}", args.source);
            std::process::exit(1);
        }

        // Get docs root path and mode
        let (docs_root, docs_mode) = if args.inline_paths || args.docs.is_some() {
            // Explicit --inline-paths or --docs flag means inline mode
            let path = args.docs.as_deref().unwrap_or("docs");
            let docs_root = path.to_string();
            (docs_root, DocsPathMode::InlinePaths)
        } else {
            // Try to get from Cargo.toml, or use/create default
            match get_or_create_docs_path(source_path, args.dry_run) {
                Ok((path, mode)) => (path, mode),
                Err(e) => {
                    eprintln!("Warning: Failed to get docs path from Cargo.toml: {}", e);
                    eprintln!("Using default 'docs' directory with inline paths");
                    ("docs".to_string(), DocsPathMode::InlinePaths)
                }
            }
        };

        if args.verbose {
            eprintln!("Source directory: {}", source_path.display());
            eprintln!("Docs root: {}", docs_root);
            eprintln!("Docs mode: {:?}", docs_mode);
            eprintln!("Strip docs: {}", args.strip_docs);
            eprintln!("Annotate: {}", args.annotate);
            eprintln!("Restore: {}", args.restore);
            eprintln!();
        }

        // Discover Rust files
        let rust_files = discover_rust_files(source_path)?;

        if rust_files.is_empty() {
            if args.verbose {
                eprintln!("No Rust files found in source directory, nothing to process.");
            }
            return Ok(()); // early exit, nothing to do
        }

        if args.verbose {
            eprintln!("Found {} Rust file(s)", rust_files.len());
        }

        // Determine optimal chunk size with oversubscription for better load balancing
        let num_threads = std::thread::available_parallelism().map_or(1, |n| n.get());

        // Create 4x more chunks than threads to minimize straggler effects
        let oversubscribe = 4;
        let total_chunks = num_threads * oversubscribe;
        let chunk_size = rust_files.len().div_ceil(total_chunks);

        if args.verbose {
            eprintln!(
                "Processing with {} threads ({} chunks of ~{} files)",
                num_threads, total_chunks, chunk_size
            );
        }

        // Process files in parallel using thread::scope
        let results: Vec<ProcessResult> = std::thread::scope(|s| {
            let handles: Vec<_> = rust_files
                .chunks(chunk_size)
                .map(|chunk| {
                    s.spawn(|| {
                        chunk
                            .iter()
                            .map(|file_path| sync(file_path, &args, &docs_root, docs_mode))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();

            // Collect results from all threads
            handles
                .into_iter()
                .flat_map(|h| h.join().unwrap_or_default())
                .collect()
        });

        // Aggregate results
        let mut total_extractions = 0;
        let mut files_processed = 0;
        let mut files_rewritten = 0;
        let mut files_touched = 0;
        let mut parse_errors = Vec::new();
        let mut all_extractions = Vec::new();

        for result in results {
            match result {
                ProcessResult::Migrated {
                    extractions,
                    rewritten,
                    touched,
                } => {
                    files_processed += 1;
                    total_extractions += extractions.len();
                    all_extractions.extend(extractions);
                    if rewritten {
                        files_rewritten += 1;
                    }
                    files_touched += touched;
                }
                ProcessResult::Restored { dry_run, .. } => {
                    files_processed += 1;
                    if !dry_run {
                        files_rewritten += 1;
                    }
                }
                ProcessResult::NoChange => {
                    files_processed += 1;
                }
                ProcessResult::Error(e) => {
                    parse_errors.push(e);
                }
            }
        }

        // Write all extractions
        if !all_extractions.is_empty() {
            let write_report = write_extractions(&all_extractions, args.dry_run)?;

            if args.verbose {
                eprintln!();
                eprintln!("Write report:");
                eprintln!("  Files written: {}", write_report.files_written);
                eprintln!("  Files skipped: {}", write_report.files_skipped);
                if !write_report.errors.is_empty() {
                    eprintln!("  Errors:");
                    for error in &write_report.errors {
                        eprintln!("    - {}", error);
                    }
                }
            }
        }

        // Print summary
        eprintln!();
        if args.dry_run {
            eprintln!("=== Dry Run Summary ===");
            eprintln!("Would process {} file(s)", files_processed);
            if args.restore {
                eprintln!("Would restore {} file(s)", files_rewritten);
            } else {
                eprintln!("Would extract {} documentation(s)", total_extractions);
                if args.touch {
                    eprintln!("Would touch {} missing file(s)", files_touched);
                }
                if args.strip_docs || args.annotate {
                    eprintln!("Would rewrite {} file(s)", files_rewritten);
                }
            }
        } else if args.restore {
            eprintln!("=== Restore Summary ===");
            eprintln!("Processed {} file(s)", files_processed);
            eprintln!("Restored {} file(s)", files_rewritten);
        } else {
            eprintln!("=== Migration Summary ===");
            eprintln!("Processed {} file(s)", files_processed);
            eprintln!("Extracted {} documentation(s)", total_extractions);
            if args.touch {
                eprintln!("Touched {} missing file(s)", files_touched);
            }
            if args.strip_docs || args.annotate {
                eprintln!("Rewrote {} file(s)", files_rewritten);
            }
        }

        if !parse_errors.is_empty() {
            eprintln!();
            eprintln!("Parse errors: {}", parse_errors.len());
            if !args.verbose {
                eprintln!("Run with --verbose to see details");
            }
        }

        if args.dry_run && !args.verbose {
            eprintln!();
            eprintln!("Dry run complete. Use -v to see detailed changes.");
        }

        Ok(())
    }
}

/// Hint replacement CLI for when the cli module is used without building the cli feature.
#[cfg(not(feature = "cli"))]
pub mod cli {
    /// Provide a hint to the user that they did not build this crate with the cli feature.
    pub fn main() {
        eprintln!("Please build with the cli feature to run the CLI");
        eprintln!("Example: cargo install syncdoc --features cli");
        std::process::exit(1);
    }
}

pub use cli::main;
