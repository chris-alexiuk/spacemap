use crate::config::{ColorResolver, SpacemapConfig};
use crate::types::{Bucket, DirEntry, FileEntry, ScanResults, Warning};
use colored::*;
use humansize::{format_size, BINARY};
use std::io;

pub struct TerminalRenderer {
    use_color: bool,
    verbose: bool,
    color_resolver: Option<ColorResolver>,
}

impl TerminalRenderer {
    pub fn new(use_color: bool, verbose: bool) -> Self {
        Self::with_config(use_color, verbose, None)
    }

    pub fn with_config(use_color: bool, verbose: bool, config: Option<&SpacemapConfig>) -> Self {
        let color_resolver = config.map(|c| ColorResolver::new(c.clone()));
        Self {
            use_color,
            verbose,
            color_resolver,
        }
    }

    pub fn render(&self, results: &ScanResults) {
        println!();
        self.print_header(results);
        println!();
        self.print_buckets(&results.buckets);

        if self.verbose {
            if !results.top_files.is_empty() {
                println!();
                self.print_top_files(&results.top_files);
            }

            if !results.top_dirs.is_empty() {
                println!();
                self.print_top_dirs(&results.top_dirs);
            }

            if let Some(ref duplicates) = results.duplicates {
                if !duplicates.is_empty() {
                    println!();
                    self.print_duplicates(duplicates);
                }
            }

            if !results.warnings.is_empty() {
                println!();
                self.print_warnings(&results.warnings);
            }
        }
        println!();
    }

    fn print_header(&self, results: &ScanResults) {
        // Title
        println!(
            "  {}",
            self.style(&format!("Storage Analysis: {}", results.scanned_path), "cyan", true)
        );
        println!("  {}", self.style(&"─".repeat(56), "bright_black", false));

        // Disk info
        if let Some(disk) = &results.disk_usage {
            let scan_percent = if disk.total_space > 0 {
                (results.totals.total_bytes as f64 / disk.total_space as f64) * 100.0
            } else {
                0.0
            };

            println!(
                "  {}  {} {} {} ({:.1}% used)",
                self.style("DISK", "blue", true),
                self.style(&format_size(disk.used_space, BINARY), "yellow", true),
                self.style("/", "white", false),
                self.style(&format_size(disk.total_space, BINARY), "white", false),
                disk.used_percent
            );

            println!(
                "  {}  {} ({:.2}% of disk)",
                self.style("SCAN", "green", true),
                self.style(&format_size(results.totals.total_bytes, BINARY), "green", false),
                scan_percent
            );
        }

        // Stats row
        println!(
            "  {}  {}    {}  {}    {}  {}",
            self.style("MODE", "magenta", true),
            self.style(&results.mode, "magenta", false),
            self.style("FILES", "cyan", true),
            self.style(&format!("{}", results.totals.file_count), "cyan", false),
            self.style("DIRS", "cyan", true),
            self.style(&format!("{}", results.totals.dir_count), "cyan", false),
        );
    }

    fn print_buckets(&self, buckets: &[Bucket]) {
        if buckets.is_empty() {
            println!("  No files found.");
            return;
        }

        // Fixed column widths
        let name_w = 14;
        let size_w = 12;
        let pct_w = 8;
        let files_w = 10;
        let bar_w = 20;

        // Header - format as single string to preserve spacing
        let header = format!(
            "  {:<name_w$}{:>size_w$}{:>pct_w$}{:>files_w$}  {}",
            "CATEGORY",
            "SIZE",
            "PCT",
            "FILES",
            "DISTRIBUTION",
            name_w = name_w,
            size_w = size_w,
            pct_w = pct_w,
            files_w = files_w,
        );
        println!("{}", self.style(&header, "white", true));

        // Separator
        let sep = format!(
            "  {}{}{}{}  {}",
            "─".repeat(name_w),
            "─".repeat(size_w),
            "─".repeat(pct_w),
            "─".repeat(files_w),
            "─".repeat(bar_w),
        );
        println!("{}", self.style(&sep, "bright_black", false));

        // Rows
        for bucket in buckets {
            let size_str = format_size(bucket.bytes, BINARY);
            let pct_str = format!("{:.1}%", bucket.percent);

            // Determine row color - use ColorResolver if available, otherwise fallback to percentage
            let (name_color, bold) = if let Some(ref resolver) = self.color_resolver {
                let color = resolver
                    .resolve_bucket_color(bucket, bucket.representative_extension.as_deref())
                    .unwrap_or_else(|| "white".to_string());
                let bold = bucket.percent > 20.0;
                (color, bold)
            } else {
                // Fallback to old percentage-based logic when no config
                if bucket.percent > 50.0 {
                    ("red".to_string(), true)
                } else if bucket.percent > 20.0 {
                    ("yellow".to_string(), true)
                } else if bucket.percent > 10.0 {
                    ("yellow".to_string(), false)
                } else {
                    ("white".to_string(), false)
                }
            };

            // Build the bar
            let bar = self.make_bar(bucket, bar_w);

            // Print without color first to get alignment right, then apply colors
            if self.use_color {
                let name_col = format!("{:<width$}", bucket.label, width = name_w);
                let size_col = format!("{:>width$}", size_str, width = size_w);
                let pct_col = format!("{:>width$}", pct_str, width = pct_w);
                let files_col = format!("{:>width$}", bucket.file_count, width = files_w);

                println!(
                    "  {}{}{}{}  {}",
                    self.style(&name_col, &name_color, bold),
                    self.style(&size_col, "green", true),
                    self.style(&pct_col, "magenta", false),
                    self.style(&files_col, "cyan", false),
                    bar
                );
            } else {
                println!(
                    "  {:<name_w$}{:>size_w$}{:>pct_w$}{:>files_w$}  {}",
                    bucket.label,
                    size_str,
                    pct_str,
                    bucket.file_count,
                    bar,
                    name_w = name_w,
                    size_w = size_w,
                    pct_w = pct_w,
                    files_w = files_w,
                );
            }
        }
    }

    fn make_bar(&self, bucket: &Bucket, width: usize) -> String {
        let filled = ((bucket.percent / 100.0) * width as f64).round() as usize;
        let filled = filled.min(width);
        let empty = width - filled;

        if !self.use_color {
            return format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        }

        let mut bar = String::new();

        // Color the filled portion - use ColorResolver if available
        let fill_color = if let Some(ref resolver) = self.color_resolver {
            resolver
                .resolve_bucket_color(bucket, bucket.representative_extension.as_deref())
                .unwrap_or_else(|| {
                    // Fallback to percentage-based coloring
                    if bucket.percent > 50.0 {
                        "red".to_string()
                    } else if bucket.percent > 20.0 {
                        "yellow".to_string()
                    } else {
                        "green".to_string()
                    }
                })
        } else {
            // Fallback to old percentage-based logic when no config
            if bucket.percent > 50.0 {
                "red".to_string()
            } else if bucket.percent > 20.0 {
                "yellow".to_string()
            } else {
                "green".to_string()
            }
        };

        bar.push_str(&self.style(&"█".repeat(filled), &fill_color, false));
        bar.push_str(&self.style(&"░".repeat(empty), "bright_black", false));
        bar
    }

    fn print_top_files(&self, files: &[FileEntry]) {
        if files.is_empty() {
            return;
        }

        println!("  {}", self.style("TOP FILES", "cyan", true));
        println!("  {}", self.style(&"─".repeat(56), "bright_black", false));

        for (i, file) in files.iter().enumerate() {
            let size = format_size(file.bytes, BINARY);
            let path = self.truncate_path(&file.path, 42);

            println!(
                "  {} {:>10}  {}",
                self.style(&format!("{:>2}.", i + 1), "bright_black", false),
                self.style(&size, "green", false),
                path
            );
        }
    }

    fn print_top_dirs(&self, dirs: &[DirEntry]) {
        if dirs.is_empty() {
            return;
        }

        println!("  {}", self.style("TOP DIRECTORIES", "cyan", true));
        println!("  {}", self.style(&"─".repeat(56), "bright_black", false));

        for (i, dir) in dirs.iter().enumerate() {
            let size = format_size(dir.bytes, BINARY);
            let path = self.truncate_path(&dir.path, 42);

            println!(
                "  {} {:>10}  {}",
                self.style(&format!("{:>2}.", i + 1), "bright_black", false),
                self.style(&size, "green", false),
                path
            );
        }
    }

    fn print_warnings(&self, warnings: &[Warning]) {
        if warnings.is_empty() {
            return;
        }

        println!(
            "  {}",
            self.style(&format!("WARNINGS ({} skipped)", warnings.len()), "yellow", true)
        );
        println!("  {}", self.style(&"─".repeat(56), "bright_black", false));

        for warning in warnings.iter().take(5) {
            let path = self.truncate_path(&warning.path, 36);
            println!(
                "    {} {}",
                self.style(&path, "bright_black", false),
                self.style(&format!("({})", warning.error), "red", false)
            );
        }

        if warnings.len() > 5 {
            println!(
                "    {}",
                self.style(&format!("...and {} more", warnings.len() - 5), "bright_black", false)
            );
        }
    }

    fn print_duplicates(&self, duplicates: &[crate::types::DuplicateGroup]) {
        println!(
            "  {}",
            self.style("DUPLICATE FILES", "yellow", true)
        );
        println!("  {}", self.style(&"─".repeat(56), "bright_black", false));

        let total_wasted: u64 = duplicates.iter().map(|d| d.wasted_space).sum();
        println!(
            "  Found {} duplicate groups, wasting {}",
            self.style(&duplicates.len().to_string(), "red", true),
            self.style(&format_size(total_wasted, BINARY), "red", true)
        );
        println!();

        for (i, dup_group) in duplicates.iter().take(10).enumerate() {
            println!(
                "  {}. {} ({} × {} files, wastes {})",
                self.style(&(i + 1).to_string(), "cyan", false),
                self.style(&format_size(dup_group.size, BINARY), "green", true),
                dup_group.paths.len(),
                self.style("duplicate", "red", false),
                self.style(&format_size(dup_group.wasted_space, BINARY), "red", false)
            );

            for path in &dup_group.paths {
                let truncated = self.truncate_path(path, 50);
                println!("     {}", self.style(&truncated, "bright_black", false));
            }
            println!();
        }

        if duplicates.len() > 10 {
            println!(
                "  {}",
                self.style(&format!("...and {} more duplicate groups", duplicates.len() - 10), "bright_black", false)
            );
        }
    }

    fn truncate_path(&self, path: &str, max_len: usize) -> String {
        if path.len() <= max_len {
            path.to_string()
        } else {
            format!("...{}", &path[path.len() - max_len + 3..])
        }
    }

    fn style(&self, text: &str, color: &str, bold: bool) -> String {
        if !self.use_color {
            return text.to_string();
        }

        let colored = match color {
            "red" => text.red(),
            "green" => text.green(),
            "yellow" => text.yellow(),
            "blue" => text.blue(),
            "cyan" => text.cyan(),
            "magenta" => text.magenta(),
            "white" => text.white(),
            "bright_black" => text.bright_black(),
            _ => text.normal(),
        };

        if bold {
            colored.bold().to_string()
        } else {
            colored.to_string()
        }
    }
}

pub struct JsonRenderer;

impl JsonRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        results: &ScanResults,
        output_file: Option<&std::path::Path>,
    ) -> io::Result<()> {
        let json = serde_json::to_string_pretty(results)?;

        if let Some(path) = output_file {
            std::fs::write(path, json)?;
        } else {
            println!("{}", json);
        }

        Ok(())
    }
}
