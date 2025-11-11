use textum::{Boundary, BoundaryMode, Snippet, Target};
use ropey::Rope;
use std::fs;

fn get_docs_path(cargo_toml_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(cargo_toml_path)?;
    let rope = Rope::from_str(&content);

    // Try to find the section text - handle both cases: another section exists or EOF
    let section_text = if let Ok(resolution) = (Snippet::Between {
        start: Boundary::new(
            Target::Literal("[package.metadata.syncdoc]".to_string()),
            BoundaryMode::Exclude,
        ),
        end: Boundary::new(
            Target::Literal("[".to_string()),
            BoundaryMode::Exclude,
        ),
    }).resolve(&rope) {
        // Found another section
        rope.slice(resolution.start..resolution.end).to_string()
    } else {
        // No next section, go from header to EOF
        let snippet = Snippet::From(
            Boundary::new(
                Target::Literal("[package.metadata.syncdoc]".to_string()),
                BoundaryMode::Exclude,
            )
        );
        let resolution = snippet.resolve(&rope)
            .map_err(|e| format!("Failed to resolve snippet: {:?}", e))?;
        rope.slice(resolution.start..resolution.end).to_string()
    };

    // Parse the docs-path value
    for line in section_text.lines() {
        let line = line.trim();
        if line.starts_with("docs-path") {
            if let Some(value) = line.split('=').nth(1) {
                let cleaned = value.trim().trim_matches('"').to_string();
                return Ok(cleaned);
            }
        }
    }

    Err("docs-path not found in [package.metadata.syncdoc] section".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

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

        let result = get_docs_path(temp.path().to_str().unwrap()).unwrap();
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

        let result = get_docs_path(temp.path().to_str().unwrap()).unwrap();
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

        let result = get_docs_path(temp.path().to_str().unwrap()).unwrap();
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

        let result = get_docs_path(temp.path().to_str().unwrap()).unwrap();
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

        let result = get_docs_path(temp.path().to_str().unwrap());
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

        let result = get_docs_path(temp.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("docs-path not found"));
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

        let result = get_docs_path(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "api-docs");
    }
}
