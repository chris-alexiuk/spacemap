use crate::progress::ScanProgress;
use crate::scanner::ScanStats;
use crate::types::{FileMetadata, Warning};
use jwalk::WalkDir;
use parking_lot::Mutex;
use rayon::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Parallel directory scanner using jwalk and rayon.
/// Uses a shared collector protected by parking_lot::Mutex for simplicity.
pub struct ParallelScanner {
    num_threads: usize,
    follow_symlinks: bool,
    max_depth: Option<usize>,
    exclude_patterns: Vec<String>,
    need_modified: bool,
}

impl ParallelScanner {
    pub fn new(
        num_threads: usize,
        follow_symlinks: bool,
        max_depth: Option<usize>,
        exclude_patterns: Vec<String>,
        need_modified: bool,
    ) -> Self {
        // Auto-detect thread count if 0
        let num_threads = if num_threads == 0 {
            rayon::current_num_threads()
        } else {
            num_threads
        };

        Self {
            num_threads,
            follow_symlinks,
            max_depth,
            exclude_patterns,
            need_modified,
        }
    }

    pub fn scan(
        &self,
        path: &Path,
        progress: &ScanProgress,
    ) -> (ScanStats, Vec<FileMetadata>) {
        // Shared statistics
        let total_bytes = Arc::new(AtomicU64::new(0));
        let file_count = Arc::new(AtomicU64::new(0));
        let dir_count = Arc::new(AtomicU64::new(0));
        let warnings = Arc::new(Mutex::new(Vec::new()));

        // Configure jwalk for parallel walking
        let mut walker = WalkDir::new(path)
            .follow_links(self.follow_symlinks)
            .skip_hidden(false)  // CRITICAL: Match walkdir behavior - scan hidden files!
            .parallelism(jwalk::Parallelism::RayonNewPool(self.num_threads));

        if let Some(depth) = self.max_depth {
            walker = walker.max_depth(depth);
        }

        // Clone for closure
        let exclude_patterns = self.exclude_patterns.clone();
        let need_modified = self.need_modified;

        // Process entries in parallel and collect FileMetadata
        let files: Vec<FileMetadata> = walker
            .into_iter()
            .par_bridge()
            .filter_map(|entry_result| {
                let result = match entry_result {
                    Ok(entry) => {
                        // Check exclusion patterns
                        if !exclude_patterns.is_empty() {
                            let path_str = entry.path().to_string_lossy().to_string();
                            if exclude_patterns.iter().any(|pattern| path_str.contains(pattern)) {
                                return None;
                            }
                        }

                        // Process the entry
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_dir() {
                                dir_count.fetch_add(1, Ordering::Relaxed);
                                None
                            } else if metadata.is_file() {
                                let size = metadata.len();
                                total_bytes.fetch_add(size, Ordering::Relaxed);
                                let count = file_count.fetch_add(1, Ordering::Relaxed) + 1;

                                let extension = entry
                                    .path()
                                    .extension()
                                    .and_then(|e| e.to_str())
                                    .map(|s| s.to_lowercase());

                                // Lazy metadata loading
                                let modified = if need_modified {
                                    metadata.modified().ok()
                                } else {
                                    None
                                };

                                // Update progress periodically
                                if count % 1000 == 0 {
                                    progress.update(
                                        count,
                                        total_bytes.load(Ordering::Relaxed),
                                        entry.path().to_str().unwrap_or(""),
                                    );
                                }

                                Some(FileMetadata {
                                    path: entry.path(),
                                    size,
                                    extension,
                                    modified,
                                })
                            } else {
                                None
                            }
                        } else {
                            warnings.lock().push(Warning {
                                path: entry.path().display().to_string(),
                                error: "Failed to read metadata".to_string(),
                            });
                            None
                        }
                    }
                    Err(e) => {
                        warnings.lock().push(Warning {
                            path: e
                                .path()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "unknown".to_string()),
                            error: e.to_string(),
                        });
                        None
                    }
                };
                result
            })
            .collect();

        progress.finish();

        let stats = ScanStats {
            total_bytes: total_bytes.load(Ordering::Relaxed),
            file_count: file_count.load(Ordering::Relaxed),
            dir_count: dir_count.load(Ordering::Relaxed),
            warnings: Arc::try_unwrap(warnings)
                .unwrap_or_else(|_| panic!("Failed to unwrap warnings"))
                .into_inner(),
        };

        (stats, files)
    }
}


