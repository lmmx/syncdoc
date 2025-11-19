#![cfg(feature = "cli")]
use assert_cmd::cargo;
use braces::{brace_paths, BraceConfig};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Module configuration for test setup
#[derive(Debug, Clone)]
pub struct ModuleConfig {
    pub name: &'static str,
    pub submodules: &'static [&'static str],
}

impl ModuleConfig {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            submodules: &[],
        }
    }

    pub const fn with_submodules(name: &'static str, submodules: &'static [&'static str]) -> Self {
        Self { name, submodules }
    }
}

/// Convert paths to brace-compressed string for inline snapshots
pub fn to_braces(paths: &[&str]) -> String {
    let braces_config = BraceConfig::default();
    brace_paths(paths, &braces_config).expect("Brace error")
}

/// Set up a test project with specified modules from fixtures
pub fn setup_test_project(modules: &[ModuleConfig]) -> TempDir {
    let temp = TempDir::new().unwrap();

    // Create Cargo.toml
    fs::write(
        temp.path().join("Cargo.toml"),
        r#"[package]
name = "test_crate"
version = "0.1.0"
edition = "2021"

[package.metadata.syncdoc]
docs-path = "docs"
"#,
    )
    .unwrap();

    // Create src directory
    let src_dir = temp.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Generate lib.rs with module declarations
    let mut lib_content = String::new();
    for module in modules {
        if module.submodules.is_empty() {
            lib_content.push_str(&format!("pub mod {};\n", module.name));
        } else {
            lib_content.push_str(&format!("pub mod {} {{\n", module.name));
            for submodule in module.submodules {
                lib_content.push_str(&format!("    pub mod {};\n", submodule));
            }
            lib_content.push_str("}\n");
        }
    }
    fs::write(src_dir.join("lib.rs"), lib_content).unwrap();

    // Copy module files from fixtures
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("roundtrip")
        .join("fixtures")
        .join("asterism");

    for module in modules {
        if module.submodules.is_empty() {
            // Simple module: copy single file
            copy_fixture_file(&fixtures_dir, &src_dir, module.name);
        } else {
            // Parent module with submodules: create directory
            let module_dir = src_dir.join(module.name);
            fs::create_dir_all(&module_dir).unwrap();

            for submodule in module.submodules {
                copy_fixture_file(&fixtures_dir.join(module.name), &module_dir, submodule);
            }
        }
    }

    temp
}

fn copy_fixture_file(source_dir: &Path, dest_dir: &Path, module_name: &str) {
    let source = source_dir.join(format!("{}.rs", module_name));
    let dest = dest_dir.join(format!("{}.rs", module_name));

    if !source.exists() {
        panic!(
            "Fixture file not found: {}. Make sure to copy it from the asterism repo.",
            source.display()
        );
    }

    fs::copy(&source, &dest).unwrap();
}

/// Normalize debug output by replacing temp paths with a stable placeholder
pub fn normalize_debug_output(stderr: &str) -> String {
    use regex::Regex;

    // Replace temp directory paths like /tmp/.tmpXXXXXX with a stable placeholder
    let temp_path_regex = Regex::new(r"/tmp/\.tmp[A-Za-z0-9]+").unwrap();
    temp_path_regex
        .replace_all(stderr, "/tmp/.tmpXXXXXX")
        .to_string()
}

/// Run syncdoc migration
pub fn run_migrate(project_dir: &Path, env_vars: HashMap<&str, &str>) -> String {
    let mut cmd = Command::new(cargo::cargo_bin!("syncdoc"));
    cmd.current_dir(project_dir).args(["--migrate"]);

    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    let output = cmd.output().unwrap();

    assert!(
        output.status.success(),
        "Migration failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8(output.stderr).unwrap();
    normalize_debug_output(&stderr)
}

/// Run syncdoc restore
pub fn run_restore(project_dir: &Path, env_vars: HashMap<&str, &str>) -> String {
    let mut cmd = Command::new(cargo::cargo_bin!("syncdoc"));
    cmd.current_dir(project_dir).args(["--restore"]);

    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    let output = cmd.output().unwrap();

    assert!(
        output.status.success(),
        "Restore failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8(output.stderr).unwrap();
    normalize_debug_output(&stderr)
}

/// Run full round-trip: migrate then restore
pub fn run_roundtrip(project_dir: &Path) -> RoundtripResult {
    let env_vars = HashMap::from([("SYNCDOC_DEBUG", "1")]);

    // Save original source
    let original_source = read_all_rs_files(&project_dir.join("src"));

    // Run migration
    let migrate_stderr = run_migrate(project_dir, env_vars.clone());

    // Capture state after migration
    let migrated_source = read_all_rs_files(&project_dir.join("src"));
    let docs_files = read_all_md_files(&project_dir.join("docs"));

    // Run restore
    let restore_stderr = run_restore(project_dir, env_vars);

    // Capture restored source
    let restored_source = read_all_rs_files(&project_dir.join("src"));

    RoundtripResult {
        original_source,
        migrated_source,
        docs_files,
        restored_source,
        migrate_stderr,
        restore_stderr,
    }
}

/// Results from a complete round-trip test
#[derive(Debug)]
pub struct RoundtripResult {
    pub original_source: HashMap<PathBuf, String>,
    pub migrated_source: HashMap<PathBuf, String>,
    pub docs_files: HashMap<PathBuf, String>,
    pub restored_source: HashMap<PathBuf, String>,
    pub migrate_stderr: String,
    pub restore_stderr: String,
}

impl RoundtripResult {
    /// Check if source was perfectly restored
    pub fn is_perfectly_restored(&self) -> bool {
        self.original_source == self.restored_source
    }

    /// Get diff for a specific file
    pub fn get_file_diff(&self, rel_path: &str) -> Option<FileDiff> {
        let path = PathBuf::from(rel_path);

        let original = self.original_source.get(&path)?;
        let restored = self.restored_source.get(&path)?;

        Some(FileDiff {
            path: path.clone(),
            original: original.clone(),
            restored: restored.clone(),
            matches: original == restored,
        })
    }

    /// Get all files that differ
    pub fn get_differing_files(&self) -> Vec<PathBuf> {
        self.original_source
            .keys()
            .filter(|path| self.original_source.get(*path) != self.restored_source.get(*path))
            .cloned()
            .collect()
    }

    /// Get brace-compressed list of all source files
    pub fn get_source_files_brace(&self) -> String {
        let mut paths: Vec<_> = self
            .original_source
            .keys()
            .map(|p| p.to_str().unwrap())
            .collect();
        paths.sort();
        to_braces(&paths)
    }

    /// Get brace-compressed list of all docs files
    pub fn get_docs_files_brace(&self) -> String {
        let mut paths: Vec<_> = self
            .docs_files
            .keys()
            .map(|p| p.to_str().unwrap())
            .collect();
        paths.sort();
        to_braces(&paths)
    }

    /// Snapshot all source files to external snapshots with prefix in subdirectory
    pub fn snapshot_source_files(&self, test_name: &str) {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(format!("snapshots/{}", test_name));
        settings.bind(|| {
            for (path, content) in &self.original_source {
                let snapshot_name =
                    format!("original_{}", path.to_str().unwrap().replace('/', "_"));
                insta::assert_snapshot!(snapshot_name, content);
            }

            for (path, content) in &self.migrated_source {
                let snapshot_name =
                    format!("migrated_{}", path.to_str().unwrap().replace('/', "_"));
                insta::assert_snapshot!(snapshot_name, content);
            }

            for (path, content) in &self.restored_source {
                let snapshot_name =
                    format!("restored_{}", path.to_str().unwrap().replace('/', "_"));
                insta::assert_snapshot!(snapshot_name, content);
            }
        });
    }

    /// Snapshot all docs files to external snapshots with prefix in subdirectory
    pub fn snapshot_docs_files(&self, test_name: &str) {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(format!("snapshots/{}", test_name));
        settings.bind(|| {
            for (path, content) in &self.docs_files {
                let snapshot_name = format!("docs_{}", path.to_str().unwrap().replace('/', "_"));
                insta::assert_snapshot!(snapshot_name, content);
            }
        });
    }
}

#[derive(Debug)]
pub struct FileDiff {
    pub path: PathBuf,
    pub original: String,
    pub restored: String,
    pub matches: bool,
}

fn read_all_rs_files(dir: &Path) -> HashMap<PathBuf, String> {
    let mut files = HashMap::new();

    if !dir.exists() {
        return files;
    }

    read_rs_files_recursive(dir, dir, &mut files);
    files
}

fn read_rs_files_recursive(root: &Path, current: &Path, files: &mut HashMap<PathBuf, String>) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "rs") {
                if let Ok(relative) = path.strip_prefix(root) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        files.insert(relative.to_path_buf(), content);
                    }
                }
            } else if path.is_dir() {
                read_rs_files_recursive(root, &path, files);
            }
        }
    }
}

fn read_all_md_files(dir: &Path) -> HashMap<PathBuf, String> {
    let mut files = HashMap::new();

    if !dir.exists() {
        return files;
    }

    read_md_files_recursive(dir, dir, &mut files);
    files
}

fn read_md_files_recursive(root: &Path, current: &Path, files: &mut HashMap<PathBuf, String>) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "md") {
                if let Ok(relative) = path.strip_prefix(root) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        files.insert(relative.to_path_buf(), content);
                    }
                }
            } else if path.is_dir() {
                read_md_files_recursive(root, &path, files);
            }
        }
    }
}
