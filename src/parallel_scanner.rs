use crate::categorize::Categorizer;
use crate::collector::SinglePassCollector;
use crate::progress::ScanProgress;
use crate::scanner::ScanStats;
use crate::types::{FileMetadata, Warning};
use jwalk::WalkDir;
use parking_lot::Mutex;
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;

/// Thread-local state: collector + statistics
struct ThreadState {
    collector: SinglePassCollector,
    total_bytes: u64,
    file_count: u64,
    dir_count: u64,
}

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

    pub fn scan<F>(
        &self,
        path: &Path,
        categorizer: Box<dyn Categorizer>,
        top_n: usize,
        should_collect_tops: bool,
        progress: &ScanProgress,
        callback: F,
    ) -> (ScanStats, SinglePassCollector)
    where
        F: Fn(&FileMetadata) + Send + Sync,
    {
        // Shared warnings (rare, so Mutex is fine)
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

        // Use rayon's fold to create thread-local state (collector + stats)
        let final_state = walker
            .into_iter()
            .par_bridge()
            .fold(
                || ThreadState {
                    collector: SinglePassCollector::new(categorizer.clone_box(), top_n, should_collect_tops),
                    total_bytes: 0,
                    file_count: 0,
                    dir_count: 0,
                },
                |mut state, entry_result| {
                    let file_meta = match entry_result {
                        Ok(entry) => {
                            // Check exclusion patterns
                            if !exclude_patterns.is_empty() {
                                let path_str = entry.path().to_string_lossy().to_string();
                                if exclude_patterns.iter().any(|pattern| path_str.contains(pattern)) {
                                    return state;
                                }
                            }

                            // Process the entry
                            if let Ok(metadata) = entry.metadata() {
                                if metadata.is_dir() {
                                    state.dir_count += 1;
                                } else if metadata.is_file() {
                                    let size = metadata.len();
                                    state.total_bytes += size;
                                    state.file_count += 1;

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

                                    let file_meta = FileMetadata {
                                        path: entry.path(),
                                        size,
                                        extension,
                                        modified,
                                    };

                                    // Process directly into thread-local collector
                                    state.collector.process_file(file_meta.clone());
                                    callback(&file_meta);
                                }
                            } else {
                                warnings.lock().push(Warning {
                                    path: entry.path().display().to_string(),
                                    error: "Failed to read metadata".to_string(),
                                });
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
                        }
                    };
                    state
                },
            )
            .reduce(
                || ThreadState {
                    collector: SinglePassCollector::new(categorizer.clone_box(), top_n, should_collect_tops),
                    total_bytes: 0,
                    file_count: 0,
                    dir_count: 0,
                },
                |mut acc, other| {
                    // Merge collectors
                    acc.collector.merge(other.collector);
                    // Sum statistics
                    acc.total_bytes += other.total_bytes;
                    acc.file_count += other.file_count;
                    acc.dir_count += other.dir_count;
                    acc
                },
            );

        progress.finish();

        let stats = ScanStats {
            total_bytes: final_state.total_bytes,
            file_count: final_state.file_count,
            dir_count: final_state.dir_count,
            warnings: Arc::try_unwrap(warnings)
                .unwrap_or_else(|_| panic!("Failed to unwrap warnings"))
                .into_inner(),
        };

        (stats, final_state.collector)
    }
}


