use std::path::{Path, PathBuf};

/// Find the Cargo manifest directory by walking up from a given path
pub fn find_manifest_dir(start_path: &Path) -> Option<PathBuf> {
    let mut current = start_path;

    loop {
        if current.join("Cargo.toml").exists() {
            return Some(current.to_path_buf());
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
