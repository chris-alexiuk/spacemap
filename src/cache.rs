use crate::types::ScanResults;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub dir_hash: String,
    pub last_scan: SystemTime,
    pub results: ScanResults,
}

pub struct ScanCache {
    cache_dir: PathBuf,
    entries: HashMap<PathBuf, CacheEntry>,
}

impl ScanCache {
    pub fn new(cache_dir: Option<PathBuf>) -> io::Result<Self> {
        let cache_dir = cache_dir.unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".cache/spacemap")
        });

        fs::create_dir_all(&cache_dir)?;

        let mut cache = Self {
            cache_dir,
            entries: HashMap::new(),
        };

        cache.load_entries()?;
        Ok(cache)
    }

    fn load_entries(&mut self) -> io::Result<()> {
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("cache") {
                    if let Ok(cache_entry) = Self::read_cache_file(&entry.path()) {
                        let scan_path = PathBuf::from(&cache_entry.results.scanned_path);
                        self.entries.insert(scan_path, cache_entry);
                    }
                }
            }
        }
        Ok(())
    }

    fn read_cache_file(path: &Path) -> io::Result<CacheEntry> {
        let contents = fs::read(path)?;
        bincode::deserialize(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn write_cache_file(path: &Path, entry: &CacheEntry) -> io::Result<()> {
        let contents = bincode::serialize(entry)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, contents)
    }

    pub fn get(&self, path: &Path) -> Option<&CacheEntry> {
        let entry = self.entries.get(path)?;

        // Validate cache: check if directory has been modified
        if let Ok(current_hash) = Self::compute_dir_hash(path) {
            if entry.dir_hash == current_hash {
                return Some(entry);
            }
        }

        None // Cache invalid
    }

    pub fn put(&mut self, path: PathBuf, results: ScanResults) -> io::Result<()> {
        let dir_hash = Self::compute_dir_hash(&path)?;

        let entry = CacheEntry {
            dir_hash,
            last_scan: SystemTime::now(),
            results,
        };

        // Write to disk
        let cache_file = self.cache_file_path(&path);
        Self::write_cache_file(&cache_file, &entry)?;

        // Update in-memory cache
        self.entries.insert(path, entry);

        Ok(())
    }

    fn cache_file_path(&self, path: &Path) -> PathBuf {
        // Create a safe filename from the path
        let path_str = path.display().to_string();
        let hash = blake3::hash(path_str.as_bytes());
        let filename = format!("{}.cache", hash.to_hex());
        self.cache_dir.join(filename)
    }

    fn compute_dir_hash(path: &Path) -> io::Result<String> {
        let metadata = fs::metadata(path)?;

        // Hash based on: mtime + size
        let mut hasher = blake3::Hasher::new();

        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                hasher.update(&duration.as_secs().to_le_bytes());
            }
        }

        hasher.update(&metadata.len().to_le_bytes());

        // Also count immediate children (for quicker invalidation)
        if let Ok(entries) = fs::read_dir(path) {
            let count = entries.count();
            hasher.update(&(count as u64).to_le_bytes());
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        fs::remove_dir_all(&self.cache_dir)?;
        fs::create_dir_all(&self.cache_dir)?;
        self.entries.clear();
        Ok(())
    }
}
