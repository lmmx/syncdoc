#![cfg(test)]
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestFixture {
    /// Not read, used for side effect (tempdir creation)
    pub _temp_dir: TempDir,
    pub docs_dir: PathBuf,
}

impl TestFixture {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let docs_dir = temp_dir.path().join("docs");
        fs::create_dir_all(&docs_dir).expect("Failed to create docs dir");

        Self {
            _temp_dir: temp_dir,
            docs_dir,
        }
    }

    pub fn create_doc_file(&self, relative_path: &str) -> PathBuf {
        let full_path = self.docs_dir.join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dirs");
        }
        fs::write(&full_path, "# Documentation\n").expect("Failed to write doc file");
        full_path
    }

    pub fn docs_path(&self) -> String {
        self.docs_dir.to_string_lossy().to_string()
    }
}
