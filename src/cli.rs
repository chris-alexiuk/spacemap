use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "spacemap")]
#[command(about = "A beautiful CLI disk space analyzer", long_about = None)]
pub struct Cli {
    /// Path to scan (defaults to current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Categorization mode: type, size, or age
    #[arg(long, value_name = "MODE", default_value = "type")]
    pub by: String,

    /// Show verbose output with drill-down sections
    #[arg(long, short)]
    pub verbose: bool,

    /// Number of top items to show in verbose sections
    #[arg(long, default_value = "10")]
    pub top: usize,

    /// Maximum depth for directory recursion
    #[arg(long, value_name = "N")]
    pub max_depth: Option<usize>,

    /// Glob or regex patterns to exclude
    #[arg(long, value_name = "PATTERN")]
    pub exclude: Vec<String>,

    /// Follow symbolic links (disabled by default)
    #[arg(long)]
    pub follow_symlinks: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Output JSON to stdout
    #[arg(long)]
    pub json: bool,

    /// Write JSON output to file
    #[arg(long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Custom size bucket boundaries (comma-separated bytes)
    #[arg(long, value_name = "SPEC")]
    pub size_buckets: Option<String>,

    /// Custom age bucket boundaries (comma-separated days)
    #[arg(long, value_name = "SPEC")]
    pub age_buckets: Option<String>,
}

impl Cli {
    pub fn validate(&self) -> Result<(), String> {
        match self.by.as_str() {
            "type" | "size" | "age" => Ok(()),
            _ => Err(format!("Invalid --by mode: {}. Must be type, size, or age", self.by)),
        }
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone().unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn should_output_json(&self) -> bool {
        self.json || self.output.is_some()
    }
}
