use facet::Facet;

#[derive(Facet)]
pub struct Args {
    /// Path to source directory to process
    #[facet(positional, default = "src".to_string())]
    pub source: String,

    /// Path to docs directory (default: 'docs' or from Cargo.toml if set)
    #[facet(named, short = 'd', long, default)]
    pub docs: Option<String>,

    /// Swap doc comments for #[omnidoc] attributes (implies cut and add)
    #[facet(named, short = 'm', long, default)]
    pub migrate: bool,

    /// Remove doc comments from source files
    #[facet(named, rename = "cut", short = 'c', long, default)]
    pub strip_docs: bool,

    /// Add #[omnidoc] attributes to items
    #[facet(named, rename = "add", short = 'a', long, default)]
    pub annotate: bool,

    /// Add #[omnidoc] attributes to items
    #[facet(named, short = 't', long, default)]
    pub touch: bool,

    /// Restore inline doc comments from markdown files (opposite of migrate)
    #[facet(named, short = 'r', long, default)]
    pub restore: bool,

    /// Preview changes without writing files
    #[facet(named, short = 'n', long, default)]
    pub dry_run: bool,

    /// Use inline path parameters instead of Cargo.toml config
    #[facet(named, long, default)]
    pub inline_paths: bool,

    /// Show verbose output
    #[facet(named, short = 'v', long, default)]
    pub verbose: bool,

    /// Show this help message
    #[facet(named, short = 'h', long, default)]
    pub help: bool,
}

pub fn print_usage() {
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
