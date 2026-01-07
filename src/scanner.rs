use crate::types::{FileMetadata, Warning};
use std::collections::{BinaryHeap, HashMap};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub struct ScanStats {
    pub total_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct FileWithSize {
    path: PathBuf,
    size: u64,
}

impl Ord for FileWithSize {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.size.cmp(&self.size)
    }
}

impl PartialOrd for FileWithSize {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Scanner {
    follow_symlinks: bool,
    max_depth: Option<usize>,
    exclude_patterns: Vec<String>,
}

impl Scanner {
    pub fn new(follow_symlinks: bool, max_depth: Option<usize>, exclude_patterns: Vec<String>) -> Self {
        Self {
            follow_symlinks,
            max_depth,
            exclude_patterns,
        }
    }

    pub fn scan<F>(&self, path: &Path, mut callback: F) -> ScanStats
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

        for entry_result in walker {
            match entry_result {
                Ok(entry) => {
                    if self.should_exclude(&entry) {
                        continue;
                    }

                    if let Err(e) = self.process_entry(&entry, &mut stats, &mut callback) {
                        stats.warnings.push(Warning {
                            path: entry.path().display().to_string(),
                            error: e.to_string(),
                        });
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

            let modified = metadata.modified().ok();

            callback(FileMetadata {
                path: entry.path().to_path_buf(),
                size,
                extension,
                modified,
            });
        }

        Ok(())
    }

    pub fn collect_top_files(&self, path: &Path, n: usize) -> Vec<(PathBuf, u64)> {
        let mut heap = BinaryHeap::new();

        self.scan(path, |meta| {
            heap.push(FileWithSize {
                path: meta.path.clone(),
                size: meta.size,
            });
        });

        heap.into_sorted_vec()
            .into_iter()
            .take(n)
            .map(|f| (f.path, f.size))
            .collect()
    }

    pub fn collect_top_dirs(&self, path: &Path, n: usize) -> Vec<(PathBuf, u64)> {
        let mut dir_sizes: HashMap<PathBuf, u64> = HashMap::new();

        self.scan(path, |meta| {
            if let Some(parent) = meta.path.parent() {
                *dir_sizes.entry(parent.to_path_buf()).or_insert(0) += meta.size;
            }
        });

        let heap: BinaryHeap<_> = dir_sizes
            .into_iter()
            .map(|(path, size)| FileWithSize { path, size })
            .collect();

        heap.into_sorted_vec()
            .into_iter()
            .take(n)
            .map(|f| (f.path, f.size))
            .collect()
    }
}
