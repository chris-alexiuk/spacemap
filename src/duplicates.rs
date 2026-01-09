use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub size: u64,
    pub hash: String,
    pub paths: Vec<String>,
    pub wasted_space: u64,
}

pub struct DuplicateFinder {
    size_groups: HashMap<u64, Vec<PathBuf>>,
}

impl DuplicateFinder {
    pub fn new() -> Self {
        Self {
            size_groups: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, path: PathBuf, size: u64) {
        self.size_groups.entry(size).or_default().push(path);
    }

    pub fn find_duplicates(&self) -> Vec<DuplicateGroup> {
        let mut duplicates = Vec::new();

        for (size, paths) in &self.size_groups {
            if paths.len() < 2 {
                continue; // No duplicates possible
            }

            // Progressive hashing: first 4KB only
            let mut quick_hashes: HashMap<String, Vec<PathBuf>> = HashMap::new();
            for path in paths {
                if let Ok(hash) = Self::hash_file_partial(path, 4096) {
                    quick_hashes.entry(hash).or_default().push(path.clone());
                }
            }

            // For groups with matching quick hashes, do full hash
            for (_, candidates) in quick_hashes {
                if candidates.len() < 2 {
                    continue;
                }

                let mut full_hashes: HashMap<String, Vec<PathBuf>> = HashMap::new();
                for path in &candidates {
                    if let Ok(hash) = Self::hash_file_full(path) {
                        full_hashes.entry(hash).or_default().push(path.clone());
                    }
                }

                for (hash, dup_paths) in full_hashes {
                    if dup_paths.len() > 1 {
                        duplicates.push(DuplicateGroup {
                            size: *size,
                            hash: hash.clone(),
                            paths: dup_paths.iter().map(|p| p.display().to_string()).collect(),
                            wasted_space: *size * (dup_paths.len() as u64 - 1),
                        });
                    }
                }
            }
        }

        duplicates.sort_by(|a, b| b.wasted_space.cmp(&a.wasted_space));
        duplicates
    }

    fn hash_file_partial(path: &PathBuf, bytes: usize) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut buffer = vec![0u8; bytes];
        let n = file.read(&mut buffer)?;
        buffer.truncate(n);

        let hash = blake3::hash(&buffer);
        Ok(hash.to_hex().to_string())
    }

    fn hash_file_full(path: &PathBuf) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = blake3::Hasher::new();
        io::copy(&mut file, &mut hasher)?;
        Ok(hasher.finalize().to_hex().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_duplicate_finder_no_duplicates() {
        let mut finder = DuplicateFinder::new();

        let mut file1 = NamedTempFile::new().unwrap();
        file1.write_all(b"content1").unwrap();

        let mut file2 = NamedTempFile::new().unwrap();
        file2.write_all(b"content2").unwrap();

        finder.add_file(file1.path().to_path_buf(), 8);
        finder.add_file(file2.path().to_path_buf(), 8);

        let duplicates = finder.find_duplicates();
        assert_eq!(duplicates.len(), 0);
    }

    #[test]
    fn test_duplicate_finder_with_duplicates() {
        let mut finder = DuplicateFinder::new();

        let mut file1 = NamedTempFile::new().unwrap();
        file1.write_all(b"same content").unwrap();
        file1.flush().unwrap();

        let mut file2 = NamedTempFile::new().unwrap();
        file2.write_all(b"same content").unwrap();
        file2.flush().unwrap();

        finder.add_file(file1.path().to_path_buf(), 12);
        finder.add_file(file2.path().to_path_buf(), 12);

        let duplicates = finder.find_duplicates();
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].paths.len(), 2);
        assert_eq!(duplicates[0].wasted_space, 12);
    }
}
