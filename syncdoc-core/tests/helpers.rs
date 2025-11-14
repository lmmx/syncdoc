#![cfg(test)]
#[path = "helpers/docs.rs"]
pub mod docs;
#[path = "helpers/formatting.rs"]
pub mod formatting;
#[path = "helpers/parsing.rs"]
pub mod parsing;

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

pub struct TestCrate {
    _temp_dir: TempDir,
    root: PathBuf,
}

impl TestCrate {
    pub fn new(name: &str) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Should have parent")
            .to_path_buf();

        let cargo_toml = format!(
            r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
syncdoc = {{ path = "{}" }}

[package.metadata.syncdoc]
docs-path = "docs"
"#,
            name,
            workspace_root.join("syncdoc").display()
        );

        fs::write(root.join("Cargo.toml"), cargo_toml).unwrap();
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir_all(root.join("docs/lib")).unwrap();

        Self {
            _temp_dir: temp_dir,
            root,
        }
    }

    pub fn write_lib(&self, code: &str) {
        let existing_types = parsing::extract_existing_types(code);
        let dummy_types = formatting::create_dummy_types_str(code, &existing_types);
        let code_with_types = formatting::inject_types_into_modules(code, &existing_types);

        let full_content = format!(
            r#"#![doc = include_str!("../docs/lib.md")]

    {}

    use syncdoc::omnidoc;

    #[omnidoc(path = "docs")]
    {}"#,
            dummy_types, code_with_types
        );
        fs::write(self.root.join("src/lib.rs"), full_content).unwrap();
    }

    pub fn write_doc(&self, relative_path: &str, content: &str) {
        let full_path = self.root.join("docs").join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full_path, content).unwrap();
    }

    pub fn auto_create_docs(&self, code: &str) {
        docs::auto_create_docs(self, code);
    }

    pub fn cargo_check(&self) -> (bool, String) {
        let output = Command::new("cargo")
            .args(&["check", "--quiet"])
            .current_dir(&self.root)
            .output()
            .expect("Failed to run cargo check");

        let success = output.status.success();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        (success, stderr)
    }

    pub fn get_expanded_lib(&self) -> Option<String> {
        let lib_path = self.root.join("src/lib.rs");
        let content = fs::read_to_string(&lib_path).ok()?;
        formatting::format_with_rustfmt(&content)
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }
}
