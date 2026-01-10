use crate::types::FileMetadata;
use crate::config::SpacemapConfig;
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

pub trait Categorizer: Send + Sync {
    fn categorize(&self, metadata: &FileMetadata) -> Cow<'static, str>;
    fn get_label(&self, key: &str) -> String;
    fn clone_box(&self) -> Box<dyn Categorizer>;
}

pub struct TypeCategorizer {
    extension_map: HashMap<String, String>,
}

impl TypeCategorizer {
    pub fn new() -> Self {
        Self::with_config(None)
    }

    pub fn with_config(config: Option<&SpacemapConfig>) -> Self {
        let mut extension_map = Self::build_default_map();

        if let Some(cfg) = config {
            // Apply custom categories
            for category in &cfg.categories {
                for ext in &category.extensions {
                    extension_map.insert(ext.to_lowercase(), category.name.clone());
                }
            }

            // Apply remaps (override existing mappings)
            for remap in &cfg.remaps {
                for ext in &remap.extensions {
                    extension_map.insert(ext.to_lowercase(), remap.category.clone());
                }
            }
        }

        Self { extension_map }
    }

    fn build_default_map() -> HashMap<String, String> {
        let mut map = HashMap::new();

        // Images
        for ext in ["jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "ico", "heic", "heif"] {
            map.insert(ext.to_string(), "Images".to_string());
        }

        // Videos
        for ext in ["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg"] {
            map.insert(ext.to_string(), "Videos".to_string());
        }

        // Audio
        for ext in ["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus"] {
            map.insert(ext.to_string(), "Audio".to_string());
        }

        // Documents
        for ext in ["pdf", "doc", "docx", "txt", "odt", "rtf", "tex", "md"] {
            map.insert(ext.to_string(), "Documents".to_string());
        }

        // Spreadsheets
        for ext in ["xls", "xlsx", "csv", "ods"] {
            map.insert(ext.to_string(), "Spreadsheets".to_string());
        }

        // Presentations
        for ext in ["ppt", "pptx", "odp"] {
            map.insert(ext.to_string(), "Presentations".to_string());
        }

        // Archives
        for ext in ["zip", "tar", "gz", "bz2", "7z", "rar", "xz", "zst", "tgz"] {
            map.insert(ext.to_string(), "Archives".to_string());
        }

        // Code
        for ext in ["rs", "py", "js", "ts", "java", "c", "cpp", "h", "hpp", "go", "rb", "php", "swift", "kt"] {
            map.insert(ext.to_string(), "Code".to_string());
        }

        // Config
        for ext in ["json", "xml", "yaml", "yml", "toml", "ini", "conf", "cfg"] {
            map.insert(ext.to_string(), "Config".to_string());
        }

        // Binaries
        for ext in ["exe", "dll", "so", "dylib", "bin", "app", "deb", "rpm"] {
            map.insert(ext.to_string(), "Binaries".to_string());
        }

        // Disk Images
        for ext in ["iso", "img", "dmg", "vdi", "vmdk"] {
            map.insert(ext.to_string(), "Disk Images".to_string());
        }

        // Databases
        for ext in ["db", "sqlite", "sql", "mdb"] {
            map.insert(ext.to_string(), "Databases".to_string());
        }

        // Logs
        map.insert("log".to_string(), "Logs".to_string());

        // Fonts
        for ext in ["ttf", "otf", "woff", "woff2"] {
            map.insert(ext.to_string(), "Fonts".to_string());
        }

        map
    }
}

impl Categorizer for TypeCategorizer {
    fn clone_box(&self) -> Box<dyn Categorizer> {
        Box::new(TypeCategorizer {
            extension_map: self.extension_map.clone(),
        })
    }

    fn categorize(&self, metadata: &FileMetadata) -> Cow<'static, str> {
        let category = metadata
            .extension
            .as_ref()
            .and_then(|ext| self.extension_map.get(ext))
            .map(|s| s.as_str())
            .unwrap_or("Other");
        Cow::Owned(category.to_string())
    }

    fn get_label(&self, key: &str) -> String {
        key.to_string()
    }
}

pub struct SizeCategorizer {
    buckets: Vec<(u64, String)>,
}

impl SizeCategorizer {
    pub fn new(custom_buckets: Option<Vec<u64>>) -> Self {
        let buckets = if let Some(bounds) = custom_buckets {
            Self::create_buckets(bounds)
        } else {
            vec![
                (0, "0-1 KiB".to_string()),
                (1024, "1-10 KiB".to_string()),
                (10 * 1024, "10-100 KiB".to_string()),
                (100 * 1024, "100 KiB-1 MiB".to_string()),
                (1024 * 1024, "1-10 MiB".to_string()),
                (10 * 1024 * 1024, "10-100 MiB".to_string()),
                (100 * 1024 * 1024, "100 MiB-1 GiB".to_string()),
                (1024 * 1024 * 1024, "1+ GiB".to_string()),
            ]
        };

        Self { buckets }
    }

    fn create_buckets(mut bounds: Vec<u64>) -> Vec<(u64, String)> {
        bounds.sort_unstable();
        bounds.dedup();

        bounds
            .iter()
            .map(|&size| {
                let label = format!("{}+ bytes", size);
                (size, label)
            })
            .collect()
    }

    fn find_bucket(&self, size: u64) -> &str {
        for i in (0..self.buckets.len()).rev() {
            if size >= self.buckets[i].0 {
                return &self.buckets[i].1;
            }
        }
        &self.buckets[0].1
    }
}

impl Categorizer for SizeCategorizer {
    fn clone_box(&self) -> Box<dyn Categorizer> {
        Box::new(SizeCategorizer {
            buckets: self.buckets.clone(),
        })
    }

    fn categorize(&self, metadata: &FileMetadata) -> Cow<'static, str> {
        Cow::Owned(self.find_bucket(metadata.size).to_string())
    }

    fn get_label(&self, key: &str) -> String {
        key.to_string()
    }
}

pub struct AgeCategorizer {
    buckets: Vec<(u64, String)>,
}

impl AgeCategorizer {
    pub fn new(custom_buckets: Option<Vec<u64>>) -> Self {
        let buckets = if let Some(bounds) = custom_buckets {
            Self::create_buckets(bounds)
        } else {
            vec![
                (0, "0-7 days".to_string()),
                (7, "7-30 days".to_string()),
                (30, "30-90 days".to_string()),
                (90, "90-365 days".to_string()),
                (365, "1+ years".to_string()),
            ]
        };

        Self { buckets }
    }

    fn create_buckets(mut bounds: Vec<u64>) -> Vec<(u64, String)> {
        bounds.sort_unstable();
        bounds.dedup();

        bounds
            .iter()
            .map(|&days| {
                let label = format!("{}+ days", days);
                (days, label)
            })
            .collect()
    }

    fn find_bucket(&self, days: u64) -> &str {
        for i in (0..self.buckets.len()).rev() {
            if days >= self.buckets[i].0 {
                return &self.buckets[i].1;
            }
        }
        &self.buckets[0].1
    }

    fn days_since_modified(modified: SystemTime) -> u64 {
        SystemTime::now()
            .duration_since(modified)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
            / 86400
    }
}

impl Categorizer for AgeCategorizer {
    fn clone_box(&self) -> Box<dyn Categorizer> {
        Box::new(AgeCategorizer {
            buckets: self.buckets.clone(),
        })
    }

    fn categorize(&self, metadata: &FileMetadata) -> Cow<'static, str> {
        if let Some(modified) = metadata.modified {
            let days = Self::days_since_modified(modified);
            Cow::Owned(self.find_bucket(days).to_string())
        } else {
            Cow::Borrowed("Unknown")
        }
    }

    fn get_label(&self, key: &str) -> String {
        key.to_string()
    }
}

