use crate::scanner::ScanStats;
use crate::types::ScanResults;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCheckpoint {
    pub version: u8,
    pub started_at: SystemTime,
    pub last_checkpoint: SystemTime,
    pub scanned_path: PathBuf,
    pub stats: CheckpointStats,
    pub partial_results: Option<ScanResults>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStats {
    pub total_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
}

impl ScanCheckpoint {
    pub fn new(scanned_path: PathBuf) -> Self {
        let now = SystemTime::now();
        Self {
            version: 1,
            started_at: now,
            last_checkpoint: now,
            scanned_path,
            stats: CheckpointStats {
                total_bytes: 0,
                file_count: 0,
                dir_count: 0,
            },
            partial_results: None,
        }
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let contents = fs::read(path)?;
        bincode::deserialize(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let contents = bincode::serialize(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, contents)
    }

    pub fn update_from_stats(&mut self, stats: &ScanStats) {
        self.stats.total_bytes = stats.total_bytes;
        self.stats.file_count = stats.file_count;
        self.stats.dir_count = stats.dir_count;
        self.last_checkpoint = SystemTime::now();
    }

    pub fn should_checkpoint(&self, interval_seconds: u64) -> bool {
        if let Ok(elapsed) = self.last_checkpoint.elapsed() {
            elapsed.as_secs() >= interval_seconds
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_checkpoint_save_load() {
        let checkpoint = ScanCheckpoint::new(PathBuf::from("/test/path"));

        let temp_file = NamedTempFile::new().unwrap();
        checkpoint.save(temp_file.path()).unwrap();

        let loaded = ScanCheckpoint::load(temp_file.path()).unwrap();
        assert_eq!(loaded.scanned_path, PathBuf::from("/test/path"));
        assert_eq!(loaded.version, 1);
    }
}
