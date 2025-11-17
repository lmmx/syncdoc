use crate::path_utils::find_manifest_dir;
use crate::syncdoc_debug;
use ropey::Rope;
use std::fs;
use std::path::{Path, PathBuf};
use textum::{Boundary, BoundaryMode, Snippet, Target};

/// Get a specified attribute from the current crate's Cargo.toml, relative to the source file
fn get_attribute_from_cargo_toml(
    cargo_toml_path: &str,
    attribute: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(cargo_toml_path)?;
    let rope = Rope::from_str(&content);

    // Try to find the section text
    let section_text = if let Ok(resolution) = (Snippet::Between {
        start: Boundary::new(
            Target::Literal("[package.metadata.syncdoc]".to_string()),
            BoundaryMode::Exclude,
        ),
        end: Boundary::new(Target::Literal("[".to_string()), BoundaryMode::Exclude),
    })
    .resolve(&rope)
    {
        rope.slice(resolution.start..resolution.end).to_string()
    } else {
        let snippet = Snippet::From(Boundary::new(
            Target::Literal("[package.metadata.syncdoc]".to_string()),
            BoundaryMode::Exclude,
        ));
        match snippet.resolve(&rope) {
            Ok(resolution) => rope.slice(resolution.start..resolution.end).to_string(),
            Err(_) => return Ok(None), // No syncdoc section, return None
        }
    };

    // Parse the specified attribute's value
    for line in section_text.lines() {
        let line = line.trim();
        if line.starts_with(attribute) {
            if let Some(value) = line.split('=').nth(1) {
                let cleaned = value.trim().trim_matches('"').to_string();
                return Ok(Some(cleaned));
            }
        }
    }

    Ok(None) // Attribute not found, return None
}

/// Resolve a source file path to an absolute path, handling both absolute and relative paths
fn resolve_source_path(source_file: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let source_path = Path::new(source_file);

    // Make it absolute if it's not already
    let source_path = if source_path.is_absolute() {
        source_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(source_path)
    };

    Ok(source_path)
}

/// Get the cfg-attr from the current crate's Cargo.toml, relative to the source file
pub fn get_cfg_attr(source_file: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let source_path = resolve_source_path(source_file)?;
    let source_dir = source_path
        .parent()
        .ok_or("Source file has no parent directory")?;

    let manifest_dir = find_manifest_dir(source_dir).ok_or("Could not find Cargo.toml")?;

    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    get_attribute_from_cargo_toml(cargo_toml_path.to_str().unwrap(), "cfg-attr")
}

/// Get the docs-path from the current crate's Cargo.toml, relative to the source file
pub fn get_docs_path(source_file: &str) -> Result<String, Box<dyn std::error::Error>> {
    syncdoc_debug!("get_docs_path called:");
    syncdoc_debug!("  source_file: {}", source_file);

    let source_path = resolve_source_path(source_file)?;

    let source_dir = source_path
        .parent()
        .ok_or("Source file has no parent directory")?;

    let manifest_dir = find_manifest_dir(source_dir).ok_or("Could not find Cargo.toml")?;
    syncdoc_debug!("  manifest_dir: {}", manifest_dir.display());

    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let docs_path = get_attribute_from_cargo_toml(cargo_toml_path.to_str().unwrap(), "docs-path")?
        .ok_or("docs-path not found")?;
    syncdoc_debug!("  docs_path from toml: {}", docs_path);

    let manifest_path = manifest_dir.canonicalize()?;
    syncdoc_debug!("  manifest_path (canonical): {}", manifest_path.display());

    let source_dir_canonical = source_dir.canonicalize()?;
    syncdoc_debug!(
        "  source_dir (canonical): {}",
        source_dir_canonical.display()
    );

    // Security check: ensure source_dir is within manifest_dir
    if !source_dir_canonical.starts_with(&manifest_path) {
        return Err("Source file is outside the manifest directory (security violation)".into());
    }

    // Calculate number of ".." needed to go from source_dir to manifest_dir
    let relative_path = source_dir_canonical
        .strip_prefix(&manifest_path)
        .map_err(|_| "Failed to strip prefix")?;
    syncdoc_debug!("  relative_path (stripped): {}", relative_path.display());

    let depth = relative_path.components().count();
    syncdoc_debug!("  depth: {}", depth);

    let mut result = PathBuf::new();

    for _ in 0..depth {
        result.push("..");
    }

    result.push(&docs_path);
    let result_str = result.to_string_lossy().to_string();
    syncdoc_debug!("  final result: {}", result_str);
    Ok(result_str)
}

#[cfg(test)]
mod docs_path_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn get_docs_path_from_file(
        cargo_toml_path: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let docs_path = get_attribute_from_cargo_toml(cargo_toml_path, "docs-path")?
            .ok_or("docs-path not found")?;
        Ok(docs_path)
    }

    #[test]
    fn test_docs_path_with_following_section() {
        let content = r#"
[package]
name = "myproject"

[package.metadata.syncdoc]
docs-path = "docs"

[dependencies]
serde = "1.0"
"#;
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", content).unwrap();
        temp.flush().unwrap();

        let result = get_docs_path_from_file(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "docs");
    }

    #[test]
    fn test_docs_path_at_eof() {
        let content = r#"
[package]
name = "myproject"

[package.metadata.syncdoc]
docs-path = "documentation"
"#;
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", content).unwrap();
        temp.flush().unwrap();

        let result = get_docs_path_from_file(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "documentation");
    }

    #[test]
    fn test_docs_path_with_extra_whitespace() {
        let content = r#"
[package.metadata.syncdoc]
  docs-path  =  "my-docs"
"#;
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", content).unwrap();
        temp.flush().unwrap();

        let result = get_docs_path_from_file(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "my-docs");
    }

    #[test]
    fn test_docs_path_without_quotes() {
        let content = r#"
[package.metadata.syncdoc]
docs-path = docs
"#;
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", content).unwrap();
        temp.flush().unwrap();

        let result = get_docs_path_from_file(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "docs");
    }

    #[test]
    fn test_missing_syncdoc_section() {
        let content = r#"
[package]
name = "myproject"
"#;
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", content).unwrap();
        temp.flush().unwrap();

        let result = get_docs_path_from_file(temp.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_docs_path_field() {
        let content = r#"
[package.metadata.syncdoc]
other-field = "value"
"#;
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", content).unwrap();
        temp.flush().unwrap();

        let result = get_docs_path_from_file(temp.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("docs-path not found"));
    }

    #[test]
    fn test_docs_path_with_multiple_fields() {
        let content = r#"
[package.metadata.syncdoc]
enable = true
docs-path = "api-docs"
output-format = "markdown"

[dependencies]
"#;
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", content).unwrap();
        temp.flush().unwrap();

        let result = get_docs_path_from_file(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "api-docs");
    }
}

#[cfg(test)]
mod relative_path_tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_get_docs_path_with_relative_source_file() {
        // Create a temporary directory structure that mimics a Rust project
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create Cargo.toml with syncdoc config
        let cargo_toml_path = project_root.join("Cargo.toml");
        let mut cargo_toml = fs::File::create(&cargo_toml_path).unwrap();
        write!(
            cargo_toml,
            r#"
[package]
name = "test-project"

[package.metadata.syncdoc]
docs-path = "docs"
"#
        )
        .unwrap();
        cargo_toml.flush().unwrap();

        // Create src directory
        let src_dir = project_root.join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create a dummy source file
        let lib_rs = src_dir.join("lib.rs");
        fs::File::create(&lib_rs).unwrap();

        // Change to the project directory so relative paths work
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(project_root).unwrap();

        // Test with a RELATIVE path (this is what proc macros give us)
        let result = get_docs_path("src/lib.rs");

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // This should succeed with the fix, fail without it
        assert!(
            result.is_ok(),
            "Should handle relative source file paths. Error: {:?}",
            result.err()
        );

        let docs_path = result.unwrap();
        assert_eq!(docs_path, "../docs");
    }

    #[test]
    fn test_get_docs_path_with_nested_relative_source_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let cargo_toml_path = project_root.join("Cargo.toml");
        let mut cargo_toml = fs::File::create(&cargo_toml_path).unwrap();
        write!(
            cargo_toml,
            r#"
[package]
name = "test-project"

[package.metadata.syncdoc]
docs-path = "documentation"
"#
        )
        .unwrap();
        cargo_toml.flush().unwrap();

        let src_dir = project_root.join("src");
        fs::create_dir(&src_dir).unwrap();

        let nested_dir = src_dir.join("nested");
        fs::create_dir(&nested_dir).unwrap();

        let nested_file = nested_dir.join("module.rs");
        fs::File::create(&nested_file).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(project_root).unwrap();

        // Test with nested relative path
        let result = get_docs_path("src/nested/module.rs");

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok(), "Should handle nested relative paths");
        let docs_path = result.unwrap();
        assert_eq!(docs_path, "../../documentation");
    }
}

#[cfg(test)]
mod cfg_attr_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn get_cfg_attr_from_file(cargo_toml_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let cfg_attr = get_attribute_from_cargo_toml(cargo_toml_path, "cfg-attr")?
            .ok_or("cfg-attr not found")?;
        Ok(cfg_attr)
    }

    #[test]
    fn test_cfg_attr_not_set() {
        let content = r#"
[package]
name = "myproject"

[package.metadata.syncdoc]
"#;
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", content).unwrap();
        temp.flush().unwrap();

        let result = get_cfg_attr_from_file(temp.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cfg-attr not found"));
    }

    #[test]
    fn test_cfg_attr_set_as_doc() {
        let content = r#"
[package]
name = "myproject"

[package.metadata.syncdoc]
cfg-attr = "doc"
"#;
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", content).unwrap();
        temp.flush().unwrap();

        let result = get_cfg_attr_from_file(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "doc");
    }

    #[test]
    fn test_cfg_attr_set_as_custom() {
        let content = r#"
[package]
name = "myproject"

[package.metadata.syncdoc]
cfg-attr = "a-custom-attr"
"#;
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", content).unwrap();
        temp.flush().unwrap();

        let result = get_cfg_attr_from_file(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "a-custom-attr");
    }
}
