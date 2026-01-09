use crate::types::FileMetadata;
use std::borrow::Cow;
use std::time::{Duration, SystemTime};

pub trait Categorizer: Send + Sync {
    fn categorize(&self, metadata: &FileMetadata) -> Cow<'static, str>;
    fn get_label(&self, key: &str) -> String;
    fn clone_box(&self) -> Box<dyn Categorizer>;
}

pub struct TypeCategorizer;

impl TypeCategorizer {
    pub fn new() -> Self {
        Self
    }

    fn map_extension_to_category(ext: &str) -> &'static str {
        match ext {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "heic" | "heif" => "Images",
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" => "Videos",
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" => "Audio",
            "pdf" | "doc" | "docx" | "txt" | "odt" | "rtf" | "tex" | "md" => "Documents",
            "xls" | "xlsx" | "csv" | "ods" => "Spreadsheets",
            "ppt" | "pptx" | "odp" => "Presentations",
            "zip" | "tar" | "gz" | "bz2" | "7z" | "rar" | "xz" | "zst" | "tgz" => "Archives",
            "rs" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" | "hpp" | "go" | "rb" | "php" | "swift" | "kt" => "Code",
            "json" | "xml" | "yaml" | "yml" | "toml" | "ini" | "conf" | "cfg" => "Config",
            "exe" | "dll" | "so" | "dylib" | "bin" | "app" | "deb" | "rpm" => "Binaries",
            "iso" | "img" | "dmg" | "vdi" | "vmdk" => "Disk Images",
            "db" | "sqlite" | "sql" | "mdb" => "Databases",
            "log" => "Logs",
            "ttf" | "otf" | "woff" | "woff2" => "Fonts",
            _ => "Other",
        }
    }
}

impl Categorizer for TypeCategorizer {
    fn clone_box(&self) -> Box<dyn Categorizer> {
        Box::new(TypeCategorizer)
    }

    fn categorize(&self, metadata: &FileMetadata) -> Cow<'static, str> {
        let category = metadata
            .extension
            .as_ref()
            .map(|ext| Self::map_extension_to_category(ext))
            .unwrap_or("Other");
        Cow::Borrowed(category)
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

