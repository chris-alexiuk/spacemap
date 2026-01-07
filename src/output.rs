use crate::types::{Bucket, DirEntry, FileEntry, ScanResults, Warning};
use colored::*;
use humansize::{format_size, BINARY};
use std::io;

pub struct TerminalRenderer {
    use_color: bool,
    verbose: bool,
}

impl TerminalRenderer {
    pub fn new(use_color: bool, verbose: bool) -> Self {
        Self { use_color, verbose }
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
                self.style("DISK", "bright_black", false),
                self.style(&format_size(disk.used_space, BINARY), "yellow", false),
                self.style("/", "bright_black", false),
                self.style(&format_size(disk.total_space, BINARY), "white", false),
                disk.used_percent
            );

            println!(
                "  {}  {} ({:.2}% of disk)",
                self.style("SCAN", "bright_black", false),
                self.style(&format_size(results.totals.total_bytes, BINARY), "green", true),
                scan_percent
            );
        }

        // Stats row
        println!(
            "  {}  {}    {}  {}    {}  {}",
            self.style("MODE", "bright_black", false),
            self.style(&results.mode, "magenta", false),
            self.style("FILES", "bright_black", false),
            self.style(&format!("{}", results.totals.file_count), "cyan", false),
            self.style("DIRS", "bright_black", false),
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

        // Header
        println!(
            "  {:<name_w$}{:>size_w$}{:>pct_w$}{:>files_w$}  {}",
            self.style("CATEGORY", "bright_black", true),
            self.style("SIZE", "bright_black", true),
            self.style("PCT", "bright_black", true),
            self.style("FILES", "bright_black", true),
            self.style("DISTRIBUTION", "bright_black", true),
            name_w = name_w,
            size_w = size_w,
            pct_w = pct_w,
            files_w = files_w,
        );

        // Separator
        println!(
            "  {}",
            self.style(
                &format!(
                    "{:<name_w$}{:>size_w$}{:>pct_w$}{:>files_w$}  {}",
                    "─".repeat(name_w - 2),
                    "─".repeat(size_w - 2),
                    "─".repeat(pct_w - 2),
                    "─".repeat(files_w - 2),
                    "─".repeat(bar_w),
                    name_w = name_w,
                    size_w = size_w,
                    pct_w = pct_w,
                    files_w = files_w,
                ),
                "bright_black",
                false
            )
        );

        // Rows
        for bucket in buckets {
            let size_str = format_size(bucket.bytes, BINARY);
            let pct_str = format!("{:.1}%", bucket.percent);

            // Determine row color based on percentage
            let (name_color, bold) = if bucket.percent > 50.0 {
                ("red", true)
            } else if bucket.percent > 20.0 {
                ("red", false)
            } else if bucket.percent > 10.0 {
                ("yellow", false)
            } else if bucket.percent > 5.0 {
                ("white", false)
            } else {
                ("bright_black", false)
            };

            // Build the bar
            let bar = self.make_bar(bucket.percent, bar_w);

            // Print without color first to get alignment right, then apply colors
            if self.use_color {
                let name_col = format!("{:<width$}", bucket.label, width = name_w);
                let size_col = format!("{:>width$}", size_str, width = size_w);
                let pct_col = format!("{:>width$}", pct_str, width = pct_w);
                let files_col = format!("{:>width$}", bucket.file_count, width = files_w);

                println!(
                    "  {}{}{}{}  {}",
                    self.style(&name_col, name_color, bold),
                    self.style(&size_col, "green", false),
                    self.style(&pct_col, "cyan", false),
                    self.style(&files_col, "blue", false),
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

    fn make_bar(&self, percent: f64, width: usize) -> String {
        let filled = ((percent / 100.0) * width as f64).round() as usize;
        let filled = filled.min(width);
        let empty = width - filled;

        if !self.use_color {
            return format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        }

        let mut bar = String::new();

        // Color the filled portion based on the overall percentage
        let fill_color = if percent > 50.0 {
            "red"
        } else if percent > 20.0 {
            "yellow"
        } else {
            "green"
        };

        bar.push_str(&self.style(&"█".repeat(filled), fill_color, false));
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
