use crate::checkpoint::ScanCheckpoint;
use crate::progress::ScanProgress;
use crate::types::{FileMetadata, Warning};
use std::path::Path;
use std::time::Instant;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub struct ScanStats {
    pub total_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub warnings: Vec<Warning>,
}

pub struct Scanner {
    follow_symlinks: bool,
    max_depth: Option<usize>,
    exclude_patterns: Vec<String>,
    need_modified: bool,
}

impl Scanner {
    pub fn new(follow_symlinks: bool, max_depth: Option<usize>, exclude_patterns: Vec<String>, need_modified: bool) -> Self {
        Self {
            follow_symlinks,
            max_depth,
            exclude_patterns,
            need_modified,
        }
    }

    pub fn scan<F>(
        &self,
        path: &Path,
        mut callback: F,
        progress: &ScanProgress,
        checkpoint: Option<(&mut ScanCheckpoint, &Path, u64)>,
    ) -> ScanStats
    where
        F: FnMut(FileMetadata),
    {
        let mut stats = ScanStats {
            total_bytes: 0,
            file_count: 0,
            dir_count: 0,
            warnings: Vec::new(),
        };

        let mut walker = WalkDir::new(path).follow_links(self.follow_symlinks);

        if let Some(depth) = self.max_depth {
            walker = walker.max_depth(depth);
        }

        // Early directory pruning: filter excluded directories BEFORE descending
        let walker = walker.into_iter().filter_entry(|entry| {
            !self.should_exclude(entry)
        });

        let mut last_checkpoint_time = Instant::now();
        let (mut checkpoint_ref, checkpoint_path, checkpoint_interval) = if let Some((ckpt, path, interval)) = checkpoint {
            (Some(ckpt), Some(path), interval)
        } else {
            (None, None, 0)
        };

        for entry_result in walker {
            match entry_result {
                Ok(entry) => {
                    if let Err(e) = self.process_entry(&entry, &mut stats, &mut callback) {
                        stats.warnings.push(Warning {
                            path: entry.path().display().to_string(),
                            error: e.to_string(),
                        });
                    }

                    // Update progress every 1000 files to avoid overhead
                    if stats.file_count % 1000 == 0 {
                        progress.update(
                            stats.file_count,
                            stats.total_bytes,
                            entry.path().to_str().unwrap_or(""),
                        );
                    }

                    // Checkpoint periodically
                    if let Some(ref mut ckpt) = checkpoint_ref {
                        if last_checkpoint_time.elapsed().as_secs() >= checkpoint_interval {
                            ckpt.update_from_stats(&stats);
                            if let Some(ckpt_path) = checkpoint_path {
                                let _ = ckpt.save(ckpt_path);
                            }
                            last_checkpoint_time = Instant::now();
                        }
                    }
                }
                Err(e) => {
                    stats.warnings.push(Warning {
                        path: e.path().map(|p| p.display().to_string()).unwrap_or_else(|| "unknown".to_string()),
                        error: e.to_string(),
                    });
                }
            }
        }

        progress.finish();
        stats
    }

    fn should_exclude(&self, entry: &DirEntry) -> bool {
        if self.exclude_patterns.is_empty() {
            return false;
        }

        let path_str = entry.path().to_string_lossy();

        for pattern in &self.exclude_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    fn process_entry<F>(
        &self,
        entry: &DirEntry,
        stats: &mut ScanStats,
        callback: &mut F,
    ) -> std::io::Result<()>
    where
        F: FnMut(FileMetadata),
    {
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            stats.dir_count += 1;
        } else if metadata.is_file() {
            let size = metadata.len();
            stats.total_bytes += size;
            stats.file_count += 1;

            let extension = entry.path()
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase());

            // Lazy metadata loading: only call modified() if needed for age mode
            let modified = if self.need_modified {
                metadata.modified().ok()
            } else {
                None
            };

            callback(FileMetadata {
                path: entry.path().to_path_buf(),
                size,
                extension,
                modified,
            });
        }

        Ok(())
    }
}
