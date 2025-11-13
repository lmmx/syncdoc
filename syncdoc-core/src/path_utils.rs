use std::path::{Path, PathBuf};

/// Find the Cargo manifest directory by walking up from a given path
/// First tries CARGO_MANIFEST_DIR env var, then walks up the filesystem
pub fn find_manifest_dir(start_path: &Path) -> Option<PathBuf> {
    // Try env var first (more reliable during macro expansion)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        return Some(PathBuf::from(manifest_dir));
    }

    // Fall back to walking up the filesystem
    let mut current = start_path;
    let root = if cfg!(windows) {
        // On Windows, stop at drive root (e.g., C:\)
        current.ancestors().last()
    } else {
        // On Unix, stop at /
        Some(Path::new("/"))
    };

    loop {
        if current.join("Cargo.toml").exists() {
            return Some(current.to_path_buf());
        }

        if Some(current) == root {
            // Reached filesystem root without finding Cargo.toml
            return None;
        }

        current = current.parent()?;
    }
}

/// Convert a doc path to be relative to the Cargo manifest directory
/// from the perspective of the call site file
pub fn make_manifest_relative_path(doc_path: &str, call_site_file: &Path) -> String {
    // Find the manifest directory
    let manifest_dir = match find_manifest_dir(call_site_file) {
        Some(dir) => dir,
        None => {
            // Fallback: return path as-is if we can't find manifest
            return doc_path.to_string();
        }
    };

    // Get the directory containing the call site file
    let call_site_dir = call_site_file.parent().unwrap_or_else(|| Path::new("."));

    // Compute relative path from call site to manifest dir
    let rel_to_manifest =
        path_relative_from(&manifest_dir, call_site_dir).unwrap_or_else(|| manifest_dir.clone());

    // Combine with the doc path
    let full_path = rel_to_manifest.join(doc_path);

    // Convert to string, using forward slashes for cross-platform compatibility
    full_path.to_str().unwrap_or(doc_path).replace('\\', "/")
}

/// Computes a relative path from `base` to `path`, returning a path with `../` components
/// if necessary.
///
/// This function is vendored from the old Rust standard library implementation
/// (pre-1.0, removed in RFC 474) and is distributed under the same terms as the
/// Rust project (MIT/Apache-2.0 dual license).
///
/// Unlike `Path::strip_prefix`, this function can handle cases where `path` is not
/// a descendant of `base`, making it suitable for finding relative paths between
/// arbitrary directories (e.g., between sibling directories in a workspace).
fn path_relative_from(path: &Path, base: &Path) -> Option<PathBuf> {
    use std::path::Component;

    if path.is_absolute() != base.is_absolute() {
        if path.is_absolute() {
            Some(PathBuf::from(path))
        } else {
            None
        }
    } else {
        let mut ita = path.components();
        let mut itb = base.components();
        let mut comps: Vec<Component> = vec![];
        loop {
            match (ita.next(), itb.next()) {
                (None, None) => break,
                (Some(a), None) => {
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
                (None, _) => comps.push(Component::ParentDir),
                (Some(a), Some(b)) if comps.is_empty() && a == b => {}
                (Some(a), Some(_b)) => {
                    comps.push(Component::ParentDir);
                    for _ in itb {
                        comps.push(Component::ParentDir);
                    }
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
            }
        }
        Some(comps.iter().map(|c| c.as_os_str()).collect())
    }
}

/// Extract module path from source file relative to src/
/// e.g., src/main.rs -> "main", src/foo/mod.rs -> "foo", src/a/b/c.rs -> "a/b/c"
pub fn extract_module_path(source_file: &str) -> String {
    let source_path = Path::new(source_file);

    if let Some(manifest_dir) = find_manifest_dir(source_path) {
        if let Ok(rel) = source_path.strip_prefix(&manifest_dir) {
            let rel_str = rel.to_string_lossy();
            let without_src = rel_str
                .strip_prefix("src/")
                .or(rel_str.strip_prefix("src\\"))
                .unwrap_or(&rel_str);

            if without_src == "main.rs" || without_src == "lib.rs" {
                return without_src.trim_end_matches(".rs").to_string();
            } else if without_src.ends_with("/mod.rs") || without_src.ends_with("\\mod.rs") {
                return without_src
                    .trim_end_matches("/mod.rs")
                    .trim_end_matches("\\mod.rs")
                    .replace('\\', "/");
            } else if without_src.ends_with(".rs") {
                return without_src.trim_end_matches(".rs").replace('\\', "/");
            }
        }
    }

    String::new()
}

pub fn apply_module_path(base_path: String) -> String {
    let call_site = proc_macro2::Span::call_site();
    if let Some(source_path) = call_site.local_file() {
        let source_file = source_path.to_string_lossy().to_string();
        let module_path = extract_module_path(&source_file);
        if module_path.is_empty() {
            base_path
        } else {
            format!("{}/{}", base_path, module_path)
        }
    } else {
        base_path
    }
}
