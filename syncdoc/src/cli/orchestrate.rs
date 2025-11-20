use super::args::Args;
use super::worker::{sync, ProcessResult};
use std::path::PathBuf;
use std::thread::{available_parallelism, scope};
use syncdoc_migrate::DocsPathMode;

/// Run `sync` on all files in parallel and collect results.
pub(crate) fn sync_all(
    files: &[PathBuf],
    args: &Args,
    docs_root: &str,
    docs_mode: DocsPathMode,
) -> Vec<ProcessResult> {
    let num_threads = available_parallelism().map_or(1, |n| n.get());
    let oversubscribe = 4;
    let total_chunks = num_threads * oversubscribe;
    let chunk_size = files.len().div_ceil(total_chunks);

    scope(|s| {
        let handles: Vec<_> = files
            .chunks(chunk_size)
            .map(|chunk| {
                s.spawn(|| {
                    chunk
                        .iter()
                        .map(|file| sync(file, args, docs_root, docs_mode))
                        .collect::<Vec<_>>()
                })
            })
            .collect();

        // Flatten results from all threads
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap_or_default())
            .collect()
    })
}
