use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResults {
    pub scanned_path: String,
    pub mode: String,
    pub totals: Totals,
    pub disk_usage: Option<DiskUsage>,
    pub buckets: Vec<Bucket>,
    pub top_files: Vec<FileEntry>,
    pub top_dirs: Vec<DirEntry>,
    pub warnings: Vec<Warning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicates: Option<Vec<DuplicateGroup>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub size: u64,
    pub hash: String,
    pub paths: Vec<String>,
    pub wasted_space: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub used_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Totals {
    pub total_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub skipped_paths: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub key: String,
    pub label: String,
    pub bytes: u64,
    pub percent: f64,
    pub file_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub representative_extension: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub extension: Option<String>,
    pub modified: Option<std::time::SystemTime>,
}
