#![cfg(test)]
#[path = "helpers/docs.rs"]
pub mod docs;
#[path = "helpers/formatting.rs"]
pub mod formatting;
#[path = "helpers/regex.rs"]
pub mod regex;

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[ctor::ctor]
fn init_debug() {
    syncdoc_core::debug::init_from_env();
}

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

        fs::write(
            root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
                [dependencies]\nsyncdoc = {{ path = \"{}\" }}\n\n\
                [package.metadata.syncdoc]\ndocs-path = \"docs\"\n",
                name,
                workspace_root.join("syncdoc").display()
            ),
        )
        .unwrap();
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir_all(root.join("docs/lib")).unwrap();

        Self {
            _temp_dir: temp_dir,
            root,
        }
    }

    pub fn write_lib(&self, code: &str) {
        let (dummy_types, existing_types) = formatting::create_dummy_types_str(code);
        let code_with_types = formatting::inject_types_into_modules(code, &existing_types);

        fs::write(
            self.root.join("src/lib.rs"),
            format!(
                "#![doc = include_str!(\"../docs/lib.md\")]\n\n{}\n\nuse syncdoc::omnidoc;\n\n\
                #[omnidoc(path = \"docs\")]\n{}",
                dummy_types, code_with_types
            ),
        )
        .unwrap();
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
            .args(["check", "--quiet"])
            .current_dir(&self.root)
            .output()
            .expect("Failed to run cargo check");

        (
            output.status.success(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )
    }

    pub fn get_expanded_lib(&self) -> Option<String> {
        let content = fs::read_to_string(self.root.join("src/lib.rs")).ok()?;
        formatting::format_with_rustfmt(&content)
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }
}
