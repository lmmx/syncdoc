use crate::path_utils::find_manifest_dir;
use ropey::Rope;
use std::fs;
use std::path::Path;
use textum::{Boundary, BoundaryMode, Snippet, Target};

/// Get a specified attribute from the current crate's Cargo.toml, relative to the source file
fn get_attribute_from_cargo_toml(
    cargo_toml_path: &Path,
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

/// Get the cfg-attr from the current crate's Cargo.toml
pub fn get_cfg_attr() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let manifest_dir = find_manifest_dir(Path::new("."))
        .ok_or("Could not find Cargo.toml (not in a Cargo project)")?;

    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    get_attribute_from_cargo_toml(&cargo_toml_path, "cfg-attr")
}

/// Get the docs-path from the current crate's Cargo.toml
pub fn get_docs_path(source_file: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let source_path = Path::new(source_file);
    let manifest_dir = find_manifest_dir(source_path)
        .ok_or("Could not find Cargo.toml (not in a Cargo project)")?;

    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let docs_path = get_attribute_from_cargo_toml(&cargo_toml_path, "docs-path")?
        .ok_or("docs-path not found")?;

    Ok(docs_path)
}

#[cfg(test)]
mod docs_path_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn get_docs_path_from_file(
        cargo_toml_path: &Path,
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

        let result = get_docs_path_from_file(temp.path()).unwrap();
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

        let result = get_docs_path_from_file(temp.path()).unwrap();
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

        let result = get_docs_path_from_file(temp.path()).unwrap();
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

        let result = get_docs_path_from_file(temp.path()).unwrap();
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

        let result = get_docs_path_from_file(temp.path());
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

        let result = get_docs_path_from_file(temp.path());
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

        let result = get_docs_path_from_file(temp.path()).unwrap();
        assert_eq!(result, "api-docs");
    }
}

#[cfg(test)]
mod cfg_attr_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn get_cfg_attr_from_file(
        cargo_toml_path: &Path,
    ) -> Result<String, Box<dyn std::error::Error>> {
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

        let result = get_cfg_attr_from_file(temp.path());
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

        let result = get_cfg_attr_from_file(temp.path()).unwrap();
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

        let result = get_cfg_attr_from_file(temp.path()).unwrap();
        assert_eq!(result, "a-custom-attr");
    }
}
