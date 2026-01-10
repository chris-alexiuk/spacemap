use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::types::Bucket;

/// Main configuration structure for spacemap
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpacemapConfig {
    /// Custom category definitions with extensions
    #[serde(default)]
    pub categories: Vec<CustomCategory>,

    /// Extension remaps to reassign extensions to different categories
    #[serde(default)]
    pub remaps: Vec<ExtensionRemap>,

    /// Category-level color overrides
    #[serde(default)]
    pub category_colors: HashMap<String, String>,

    /// Per-extension color overrides (highest priority)
    #[serde(default)]
    pub extension_colors: HashMap<String, String>,

    /// Display settings
    #[serde(default)]
    pub display: DisplayConfig,
}

/// Custom category with name, extensions, and optional color
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomCategory {
    pub name: String,
    pub extensions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Remap extensions to a different category
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtensionRemap {
    pub extensions: Vec<String>,
    pub category: String,
}

/// Display configuration options
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DisplayConfig {
    /// Whether to use percentage-based coloring as fallback (default: true)
    #[serde(default = "default_true")]
    pub use_percentage_colors: bool,
}

fn default_true() -> bool {
    true
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            use_percentage_colors: true,
        }
    }
}

impl SpacemapConfig {
    /// Load config from custom path or default XDG location
    pub fn load(custom_path: Option<&PathBuf>) -> Result<Self, ConfigError> {
        let path = if let Some(p) = custom_path {
            p.clone()
        } else {
            match Self::default_config_path() {
                Ok(p) => p,
                Err(_) => return Ok(Self::default()),
            }
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&path)
            .map_err(|e| ConfigError::Io(path.clone(), e))?;

        toml::from_str(&contents).map_err(|e| ConfigError::Parse(path.clone(), e))
    }

    /// Get default config path: ~/.config/spacemap/config.toml
    pub fn default_config_path() -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;

        Ok(config_dir.join("spacemap").join("config.toml"))
    }
}

impl Default for SpacemapConfig {
    fn default() -> Self {
        Self {
            categories: Vec::new(),
            remaps: Vec::new(),
            category_colors: HashMap::new(),
            extension_colors: HashMap::new(),
            display: DisplayConfig::default(),
        }
    }
}

/// Color resolution with priority: extension > category > percentage
pub struct ColorResolver {
    config: SpacemapConfig,
}

impl ColorResolver {
    pub fn new(config: SpacemapConfig) -> Self {
        Self { config }
    }

    /// Resolve color for a bucket based on priority system
    /// Priority: extension_colors > category_colors > percentage-based
    pub fn resolve_bucket_color(
        &self,
        bucket: &Bucket,
        extension: Option<&str>,
    ) -> Option<String> {
        // Priority 1: Extension-specific color
        if let Some(ext) = extension {
            if let Some(color) = self.config.extension_colors.get(ext) {
                return Some(color.clone());
            }
        }

        // Priority 2: Category-level color
        if let Some(color) = self.config.category_colors.get(&bucket.key) {
            return Some(color.clone());
        }

        // Check if category has custom color from category definition
        for cat in &self.config.categories {
            if cat.name == bucket.key {
                if let Some(ref color) = cat.color {
                    return Some(color.clone());
                }
            }
        }

        // Priority 3: Percentage-based (if enabled)
        if self.config.display.use_percentage_colors {
            return Some(Self::percentage_to_color(bucket.percent));
        }

        None
    }

    fn percentage_to_color(percent: f64) -> String {
        if percent > 50.0 {
            "red".to_string()
        } else if percent > 20.0 {
            "yellow".to_string()
        } else {
            "white".to_string()
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    NoConfigDir,
    Io(PathBuf, std::io::Error),
    Parse(PathBuf, toml::de::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NoConfigDir => write!(f, "Could not determine config directory"),
            ConfigError::Io(path, e) => {
                write!(f, "Failed to read config at {}: {}", path.display(), e)
            }
            ConfigError::Parse(path, e) => {
                write!(f, "Failed to parse config at {}: {}", path.display(), e)
            }
        }
    }
}

impl std::error::Error for ConfigError {}
