#![cfg(test)]
#![allow(dead_code)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use syncdoc_migrate::{
    discover::parse_file,
    write::{extract_all_docs, find_expected_doc_paths, DocExtraction, WriteReport},
};
use tempfile::TempDir;

#[ctor::ctor]
fn init_debug() {
    syncdoc_migrate::debug::set_debug(true);
}

pub fn setup_test_file(source: &str, filename: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, source).unwrap();
    (temp_dir, file_path)
}

pub fn parse_and_extract(path: &Path, root: &str) -> (Vec<DocExtraction>, Vec<DocExtraction>) {
    let parsed = parse_file(path).unwrap();
    let extractions = extract_all_docs(&parsed, root);
    let expected = find_expected_doc_paths(&parsed, root);

    let existing: HashSet<_> = extractions.iter().map(|e| &e.markdown_path).collect();

    let missing: Vec<_> = expected
        .into_iter()
        .filter(|e| !existing.contains(&e.markdown_path) && !e.markdown_path.exists())
        .collect();

    (extractions, missing)
}

pub fn assert_report(report: &WriteReport, expected: usize) {
    assert_eq!(report.files_written, expected);
    assert_eq!(report.files_skipped, 0);
    assert!(report.errors.is_empty());
}

pub fn assert_file(path: impl AsRef<Path>, content: &str) {
    assert_eq!(fs::read_to_string(path).unwrap(), content);
}

pub fn assert_missing_path(missing: &[DocExtraction], path_suffix: &str) {
    assert!(
        missing
            .iter()
            .any(|e| e.markdown_path.to_str().unwrap().ends_with(path_suffix)),
        "Expected to find missing path ending with '{}'",
        path_suffix
    );
}
