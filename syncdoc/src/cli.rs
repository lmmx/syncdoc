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
pub mod inner {
    use facet::Facet;
    use std::fs;
    use std::io;
    use std::path::Path;
    use syncdoc_migrate::{
        discover_rust_files, extract_all_docs, find_expected_doc_paths, get_or_create_docs_path,
        parse_file, restore_file, rewrite_file, write_extractions, DocsPathMode,
    };

    #[derive(Facet)]
    struct Args {
        /// Path to source directory to process
        #[facet(positional, default = "src".to_string())]
        source: String,

        /// Path to docs directory (default: 'docs' or from Cargo.toml if set)
        #[facet(named, short = 'd', long, default)]
        docs: Option<String>,

        /// Swap doc comments for #[omnidoc] attributes (implies cut and add)
        #[facet(named, short = 'm', long, default)]
        migrate: bool,

        /// Remove doc comments from source files
        #[facet(named, rename = "cut", short = 'c', long, default)]
        strip_docs: bool,

        /// Add #[omnidoc] attributes to items
        #[facet(named, rename = "add", short = 'a', long, default)]
        annotate: bool,

        /// Add #[omnidoc] attributes to items
        #[facet(named, short = 't', long, default)]
        touch: bool,

        /// Restore inline doc comments from markdown files (opposite of migrate)
        #[facet(named, short = 'r', long, default)]
        restore: bool,

        /// Preview changes without writing files
        #[facet(named, short = 'n', long, default)]
        dry_run: bool,

        /// Use inline path parameters instead of Cargo.toml config
        #[facet(named, long, default)]
        inline_paths: bool,

        /// Show verbose output
        #[facet(named, short = 'v', long, default)]
        verbose: bool,

        /// Show this help message
        #[facet(named, short = 'h', long, default)]
        help: bool,
    }

    fn print_usage() {
        println!("Usage: syncdoc [OPTIONS] <SOURCE>");
        println!();
        println!("Migrate Rust documentation to external markdown files.");
        println!();
        println!("Arguments:");
        println!("  <SOURCE>           Path to source directory to process (default: 'src')");
        println!();
        println!("Options:");
        println!(
            "  -d, --docs <dir>   Path to docs directory (default: 'docs' or from Cargo.toml if set)"
        );
        println!("  -m, --migrate      Swap doc comments for #[omnidoc] (cut + add + touch)");
        println!("  -c, --cut          Cut out doc comments from source files");
        println!("  -a, --add          Rewrite code with #[omnidoc] attributes");
        println!("  -t, --touch        Touch empty markdown files for any that don't exist");
        println!("      --inline-paths Use inline path= parameters instead of Cargo.toml");
        println!("  -r, --restore      Restore inline doc comments from markdown files");
        println!("  -n, --dry-run      Preview changes without writing files");
        println!("  -v, --verbose      Show verbose output");
        println!("  -h, --help         Show this help message");
        println!();
        println!("Examples:");
        println!("  # 'Sync' the docs dir with the docstrings in src/");
        println!("  syncdoc");
        println!();
        println!("  # Preview a full migration without running it");
        println!("  syncdoc --migrate --dry-run (or `-m -n` for short)");
        println!();
        println!("  # Full migration: cut docs, add attributes, and touch missing files");
        println!("  syncdoc --migrate (or `-m` for short, equal to `--cut --add --touch`)");
        println!();
        println!("  # Migrate with inline paths instead of Cargo.toml config");
        println!("  syncdoc --migrate --inline-paths");
        println!();
        println!("  # Restore documentation from markdown back to source");
        println!("  syncdoc --restore");
    }

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
            let path = args.docs.unwrap_or_else(|| "docs".to_string());
            (path, DocsPathMode::InlinePaths)
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

        if args.verbose {
            eprintln!("Found {} Rust file(s)", rust_files.len());
        }

        let mut total_extractions = 0;
        let mut files_processed = 0;
        let mut files_rewritten = 0;
        let mut files_touched = 0;
        let mut parse_errors = Vec::new();
        let mut all_extractions = Vec::new();

        // Process each file
        for file_path in &rust_files {
            if args.verbose {
                eprintln!("Processing: {}", file_path.display());
            }

            // Parse the file
            let parsed = match parse_file(file_path) {
                Ok(p) => p,
                Err(e) => {
                    let error_msg = format!("Failed to parse {}: {}", file_path.display(), e);
                    parse_errors.push(error_msg.clone());
                    if args.verbose {
                        eprintln!("  Warning: {}", error_msg);
                    }
                    continue;
                }
            };

            files_processed += 1;

            // RESTORE MODE: early exit from loop iteration
            if args.restore {
                if let Some(restored) = restore_file(&parsed, &docs_root) {
                    if args.dry_run {
                        if args.verbose {
                            eprintln!("  Would restore: {}", file_path.display());
                        }
                    } else {
                        fs::write(file_path, restored)?;
                        if args.verbose {
                            eprintln!("  Restored: {}", file_path.display());
                        }
                    }
                    files_rewritten += 1;
                }
                continue; // Skip the rest of the loop for this file
            }

            // MIGRATION MODE: existing code continues from here
            // Extract documentation
            let extractions = extract_all_docs(&parsed, &docs_root);
            total_extractions += extractions.len();

            if args.verbose && !extractions.is_empty() {
                eprintln!("  Found {} doc extraction(s)", extractions.len());
            }

            all_extractions.extend(extractions);

            // If touch mode and we're adding annotations, find expected paths
            if args.touch && args.annotate {
                let expected_paths = find_expected_doc_paths(&parsed, &docs_root);

                if args.verbose {
                    eprintln!("  Found {} expected doc path(s)", expected_paths.len());
                }

                // Filter to only those that don't already exist in all_extractions
                // First create a set of paths we already have content for
                let existing_paths: std::collections::HashSet<_> =
                    all_extractions.iter().map(|e| &e.markdown_path).collect();

                // Then filter expected paths to only those we don't have AND that don't exist on disk
                let missing_paths: Vec<_> = expected_paths
                    .into_iter()
                    .filter(|extraction| {
                        !existing_paths.contains(&extraction.markdown_path)
                            && !extraction.markdown_path.exists()
                    })
                    .collect();

                if !missing_paths.is_empty() {
                    if args.verbose {
                        eprintln!("  Will touch {} missing file(s)", missing_paths.len());
                    }
                    files_touched += missing_paths.len();
                    all_extractions.extend(missing_paths);
                }
            }

            // Rewrite source file if requested
            if args.strip_docs || args.annotate {
                if let Some(rewritten) = rewrite_file(
                    &parsed,
                    &docs_root,
                    docs_mode,
                    args.strip_docs,
                    args.annotate,
                ) {
                    if args.dry_run {
                        if args.verbose {
                            eprintln!("  Would rewrite: {}", file_path.display());
                        }
                    } else {
                        fs::write(file_path, rewritten)?;
                        if args.verbose {
                            eprintln!("  Rewrote: {}", file_path.display());
                        }
                    }
                    files_rewritten += 1;
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
pub mod inner {
    /// Provide a hint to the user that they did not build this crate with the cli feature.
    pub fn main() {
        eprintln!("Please build with the cli feature to run the CLI");
        eprintln!("Example: cargo install syncdoc --features cli");
        std::process::exit(1);
    }
}

pub use inner::main;
