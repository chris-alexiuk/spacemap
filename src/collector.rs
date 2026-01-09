use crate::bounded_heap::BoundedMinHeap;
use crate::categorize::Categorizer;
use crate::path_pool::PathPool;
use crate::types::{Bucket, DirEntry, FileEntry, FileMetadata};
use std::collections::HashMap;
use std::path::PathBuf;

/// A file with its size, ordered by size for use in BoundedMinHeap.
#[derive(Debug, Clone, Eq, PartialEq)]
struct FileWithSize {
    path: PathBuf,
    size: u64,
}

impl Ord for FileWithSize {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.size.cmp(&other.size)
    }
}

impl PartialOrd for FileWithSize {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A directory with its total size, ordered by size for use in BoundedMinHeap.
#[derive(Debug, Clone, Eq, PartialEq)]
struct DirWithSize {
    path: PathBuf,
    size: u64,
}

impl Ord for DirWithSize {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.size.cmp(&other.size)
    }
}

impl PartialOrd for DirWithSize {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Results from single-pass collection.
pub struct CollectionResults {
    pub buckets: Vec<Bucket>,
    pub top_files: Vec<FileEntry>,
    pub top_dirs: Vec<DirEntry>,
}

/// Single-pass collector that simultaneously:
/// 1. Categorizes and aggregates files into buckets
/// 2. Tracks top N largest files using a bounded heap
/// 3. Accumulates directory sizes and tracks top N largest directories
///
/// This replaces the previous approach of making three separate filesystem scans.
pub struct SinglePassCollector {
    categorizer: Box<dyn Categorizer>,

    // Category aggregation: category -> (total_bytes, file_count)
    category_stats: HashMap<String, (u64, u64)>,

    // Top files tracking using bounded heap
    top_files_heap: BoundedMinHeap<FileWithSize>,

    // Directory size accumulation: path_id -> total_bytes
    // Using u32 path IDs instead of PathBuf for memory efficiency
    // Note: We need to accumulate ALL directory sizes to be accurate,
    // but we use a bounded heap for the final top-N selection
    dir_accumulator: HashMap<u32, u64>,

    // Path interning pool: stores unique paths once, returns u32 IDs
    // This reduces memory usage for deep directory trees
    path_pool: PathPool,

    // Capacity for top-N tracking
    top_n: usize,

    // Whether to collect top files/dirs (only if verbose or JSON output)
    should_collect_tops: bool,
}

impl SinglePassCollector {
    /// Creates a new single-pass collector.
    ///
    /// # Arguments
    /// * `categorizer` - The categorization strategy (type, size, or age)
    /// * `top_n` - How many top files/directories to track
    /// * `should_collect_tops` - Whether to collect top files/dirs (skip if not verbose/JSON)
    pub fn new(categorizer: Box<dyn Categorizer>, top_n: usize, should_collect_tops: bool) -> Self {
        Self {
            categorizer,
            category_stats: HashMap::new(),
            top_files_heap: BoundedMinHeap::new(top_n),
            dir_accumulator: HashMap::new(),
            path_pool: PathPool::new(),
            top_n,
            should_collect_tops,
        }
    }

    /// Process a single file during the scan.
    ///
    /// This is called once for each file encountered during directory traversal.
    /// It performs all three collection tasks in a single pass:
    /// 1. Categorizes the file and updates category statistics
    /// 2. Potentially adds the file to the top-N heap
    /// 3. Accumulates the file's size to its parent directory
    pub fn process_file(&mut self, metadata: FileMetadata) {
        let size = metadata.size;

        // 1. Categorize and aggregate
        let category = self.categorizer.categorize(&metadata).into_owned();
        let entry = self.category_stats.entry(category).or_insert((0, 0));
        entry.0 += size;
        entry.1 += 1;

        if self.should_collect_tops {
            // 2. Track top files
            self.top_files_heap.push(FileWithSize {
                path: metadata.path.clone(),
                size,
            });

            // 3. Accumulate directory sizes using path interning
            if let Some(parent) = metadata.path.parent() {
                let path_id = self.path_pool.intern(parent);
                *self.dir_accumulator.entry(path_id).or_insert(0) += size;
            }
        }
    }

    /// Finalize collection and produce results.
    ///
    /// This converts the accumulated data into the final output format:
    /// - Creates buckets from category statistics
    /// - Extracts top files from the bounded heap
    /// - Selects top directories from the accumulator using a bounded heap
    pub fn finalize(self, total_bytes: u64) -> CollectionResults {
        // Create buckets from category statistics
        let mut buckets: Vec<Bucket> = self
            .category_stats
            .into_iter()
            .map(|(key, (bytes, count))| {
                let percent = if total_bytes > 0 {
                    (bytes as f64 / total_bytes as f64) * 100.0
                } else {
                    0.0
                };

                Bucket {
                    label: self.categorizer.get_label(&key),
                    key,
                    bytes,
                    percent,
                    file_count: count,
                }
            })
            .collect();

        buckets.sort_by(|a, b| b.bytes.cmp(&a.bytes));

        // Extract top files
        let top_files = if self.should_collect_tops {
            self.top_files_heap
                .into_sorted_vec()
                .into_iter()
                .map(|f| FileEntry {
                    path: f.path.display().to_string(),
                    bytes: f.size,
                })
                .collect()
        } else {
            Vec::new()
        };

        // Extract top directories using bounded heap
        // Dereference path IDs back to paths using the path pool
        let top_dirs = if self.should_collect_tops {
            let mut dir_heap = BoundedMinHeap::new(self.top_n);
            for (path_id, size) in self.dir_accumulator {
                // Dereference path ID to actual path
                if let Some(path) = self.path_pool.get(path_id) {
                    dir_heap.push(DirWithSize {
                        path: path.to_path_buf(),
                        size
                    });
                }
            }

            dir_heap
                .into_sorted_vec()
                .into_iter()
                .map(|d| DirEntry {
                    path: d.path.display().to_string(),
                    bytes: d.size,
                })
                .collect()
        } else {
            Vec::new()
        };

        CollectionResults {
            buckets,
            top_files,
            top_dirs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::categorize::TypeCategorizer;

    fn create_test_metadata(path: &str, size: u64, ext: Option<&str>) -> FileMetadata {
        FileMetadata {
            path: PathBuf::from(path),
            size,
            extension: ext.map(String::from),
            modified: None,
        }
    }

    #[test]
    fn test_single_pass_collector_basic() {
        let categorizer = Box::new(TypeCategorizer::new());
        let mut collector = SinglePassCollector::new(categorizer, 3, true);

        collector.process_file(create_test_metadata("/test/file1.rs", 100, Some("rs")));
        collector.process_file(create_test_metadata("/test/file2.py", 200, Some("py")));
        collector.process_file(create_test_metadata("/test/file3.rs", 150, Some("rs")));

        let results = collector.finalize(450);

        // Should have categorized into "Code"
        assert_eq!(results.buckets.len(), 1);
        assert_eq!(results.buckets[0].key, "Code");
        assert_eq!(results.buckets[0].bytes, 450);
        assert_eq!(results.buckets[0].file_count, 3);

        // Top files should be in descending order
        assert_eq!(results.top_files.len(), 3);
        assert_eq!(results.top_files[0].bytes, 200);
        assert_eq!(results.top_files[1].bytes, 150);
        assert_eq!(results.top_files[2].bytes, 100);
    }

    #[test]
    fn test_single_pass_collector_bounded_top_files() {
        let categorizer = Box::new(TypeCategorizer::new());
        let mut collector = SinglePassCollector::new(categorizer, 2, true);

        // Add 5 files, but only top 2 should be kept
        collector.process_file(create_test_metadata("/test/file1.txt", 100, Some("txt")));
        collector.process_file(create_test_metadata("/test/file2.txt", 500, Some("txt")));
        collector.process_file(create_test_metadata("/test/file3.txt", 200, Some("txt")));
        collector.process_file(create_test_metadata("/test/file4.txt", 50, Some("txt")));
        collector.process_file(create_test_metadata("/test/file5.txt", 300, Some("txt")));

        let results = collector.finalize(1150);

        // Should only have top 2 files
        assert_eq!(results.top_files.len(), 2);
        assert_eq!(results.top_files[0].bytes, 500);
        assert_eq!(results.top_files[1].bytes, 300);
    }

    #[test]
    fn test_single_pass_collector_skip_tops() {
        let categorizer = Box::new(TypeCategorizer::new());
        let mut collector = SinglePassCollector::new(categorizer, 10, false);

        collector.process_file(create_test_metadata("/test/file1.txt", 100, Some("txt")));
        collector.process_file(create_test_metadata("/test/file2.txt", 200, Some("txt")));

        let results = collector.finalize(300);

        // Buckets should still be created
        assert_eq!(results.buckets.len(), 1);
        assert_eq!(results.buckets[0].file_count, 2);

        // But top files/dirs should be empty
        assert_eq!(results.top_files.len(), 0);
        assert_eq!(results.top_dirs.len(), 0);
    }
}
