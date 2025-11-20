//! Aggregation and reporting for `syncdoc` CLI.

use super::args::Args;
use super::worker::ProcessResult;
use syncdoc_migrate::DocExtraction;

/// Aggregated results of a CLI run.
pub(crate) struct AggregatedResults {
    pub files_processed: usize,
    pub files_rewritten: usize,
    pub files_touched: usize,
    pub total_extractions: usize,
    pub parse_errors: Vec<String>,
    pub all_extractions: Vec<DocExtraction>,
}

impl AggregatedResults {
    /// Create an empty aggregation.
    pub(crate) fn new() -> Self {
        Self {
            files_processed: 0,
            files_rewritten: 0,
            files_touched: 0,
            total_extractions: 0,
            parse_errors: Vec::new(),
            all_extractions: Vec::new(),
        }
    }
}

/// Aggregate a vector of `ProcessResult` into counts and extractions.
pub(crate) fn aggregate_results(results: Vec<ProcessResult>) -> AggregatedResults {
    let mut agg = AggregatedResults::new();

    for result in results {
        match result {
            ProcessResult::Migrated {
                extractions,
                rewritten,
                touched,
            } => {
                agg.files_processed += 1;
                agg.total_extractions += extractions.len();
                agg.all_extractions.extend(extractions);
                if rewritten {
                    agg.files_rewritten += 1;
                }
                agg.files_touched += touched;
            }
            ProcessResult::Restored { dry_run } => {
                agg.files_processed += 1;
                if !dry_run {
                    agg.files_rewritten += 1;
                }
            }
            ProcessResult::NoChange => {
                agg.files_processed += 1;
            }
            ProcessResult::Error(e) => {
                agg.parse_errors.push(e);
            }
        }
    }

    agg
}

/// Print a CLI summary report based on aggregated results.
pub(crate) fn print_summary(agg: &AggregatedResults, args: &Args, dry_run: bool, verbose: bool) {
    // Write all extractions report (still delegates to syncdoc_migrate::write_extractions in main)
    if !agg.all_extractions.is_empty() && verbose {
        eprintln!();
        eprintln!("Write report:");
        // Actual writing is done outside, so this just prints a placeholder
        eprintln!("  Files written/skipped handled elsewhere");
    }

    eprintln!();
    if dry_run {
        eprintln!("=== Dry Run Summary ===");
        eprintln!("Would process {} file(s)", agg.files_processed);
        if args.restore {
            eprintln!("Would restore {} file(s)", agg.files_rewritten);
        } else {
            eprintln!("Would extract {} documentation(s)", agg.total_extractions);
            if args.touch {
                eprintln!("Would touch {} missing file(s)", agg.files_touched);
            }
            if args.strip_docs || args.annotate {
                eprintln!("Would rewrite {} file(s)", agg.files_rewritten);
            }
        }
    } else if args.restore {
        eprintln!("=== Restore Summary ===");
        eprintln!("Processed {} file(s)", agg.files_processed);
        eprintln!("Restored {} file(s)", agg.files_rewritten);
    } else {
        eprintln!("=== Migration Summary ===");
        eprintln!("Processed {} file(s)", agg.files_processed);
        eprintln!("Extracted {} documentation(s)", agg.total_extractions);
        if args.touch {
            eprintln!("Touched {} missing file(s)", agg.files_touched);
        }
        if args.strip_docs || args.annotate {
            eprintln!("Rewrote {} file(s)", agg.files_rewritten);
        }
    }

    if !agg.parse_errors.is_empty() {
        eprintln!();
        eprintln!("Parse errors: {}", agg.parse_errors.len());
        if !verbose {
            eprintln!("Run with --verbose to see details");
        } else {
            for error in &agg.parse_errors {
                eprintln!("  - {}", error);
            }
        }
    }

    if dry_run && !verbose {
        eprintln!();
        eprintln!("Dry run complete. Use -v to see detailed changes.");
    }
}
