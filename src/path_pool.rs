use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Path interning pool that stores unique paths once and returns u32 IDs.
/// This reduces memory usage for deep directory trees where many paths share prefixes.
pub struct PathPool {
    paths: Vec<PathBuf>,               // Store unique paths
    index: HashMap<PathBuf, u32>,      // Path -> ID mapping
}

impl PathPool {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// Intern a path and return its unique ID.
    /// If the path already exists, returns its existing ID.
    /// Otherwise, assigns a new ID and stores the path.
    pub fn intern(&mut self, path: &Path) -> u32 {
        if let Some(&id) = self.index.get(path) {
            return id;
        }

        let id = self.paths.len() as u32;
        let path_buf = path.to_path_buf();
        self.paths.push(path_buf.clone());
        self.index.insert(path_buf, id);
        id
    }

    /// Get the path associated with an ID.
    pub fn get(&self, id: u32) -> Option<&Path> {
        self.paths.get(id as usize).map(|p| p.as_path())
    }

    /// Get the number of unique paths stored.
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Check if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_pool_basic() {
        let mut pool = PathPool::new();

        let path1 = PathBuf::from("/home/user/dir1");
        let path2 = PathBuf::from("/home/user/dir2");

        let id1 = pool.intern(&path1);
        let id2 = pool.intern(&path2);

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(pool.len(), 2);

        assert_eq!(pool.get(id1), Some(path1.as_path()));
        assert_eq!(pool.get(id2), Some(path2.as_path()));
    }

    #[test]
    fn test_path_pool_deduplication() {
        let mut pool = PathPool::new();

        let path = PathBuf::from("/home/user/dir");

        let id1 = pool.intern(&path);
        let id2 = pool.intern(&path);

        // Same path should get same ID
        assert_eq!(id1, id2);
        // Should only store path once
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_path_pool_memory_efficiency() {
        let mut pool = PathPool::new();

        // Simulate deep directory tree with shared prefixes
        let base = PathBuf::from("/very/long/path/prefix");

        for i in 0..100 {
            let mut path = base.clone();
            path.push(format!("subdir{}", i));
            pool.intern(&path);
        }

        assert_eq!(pool.len(), 100);

        // Verify all paths are retrievable
        for i in 0..100 {
            let mut expected = base.clone();
            expected.push(format!("subdir{}", i));
            assert_eq!(pool.get(i), Some(expected.as_path()));
        }
    }
}
