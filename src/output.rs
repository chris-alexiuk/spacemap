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
        self.print_header(results);
        println!();
        self.print_buckets(&results.buckets, results.totals.total_bytes);

        if self.verbose {
            println!();
            self.print_top_files(&results.top_files);

            if !results.top_dirs.is_empty() {
                println!();
                self.print_top_dirs(&results.top_dirs);
            }

            if !results.warnings.is_empty() {
                println!();
                self.print_warnings(&results.warnings);
            }
        }
    }

    fn print_header(&self, results: &ScanResults) {
        let title = format!("Storage Analysis: {}", results.scanned_path);
        println!("{}", self.colorize(&title, "cyan", true));
        println!("{}", "=".repeat(title.len()));

        if let Some(disk) = &results.disk_usage {
            let scan_percent = if disk.total_space > 0 {
                (results.totals.total_bytes as f64 / disk.total_space as f64) * 100.0
            } else {
                0.0
            };

            println!(
                "Disk: {} / {} ({:.1}% used)  |  Scanned: {} ({:.2}% of disk)",
                self.colorize(&format_size(disk.used_space, BINARY), "yellow", false),
                self.colorize(&format_size(disk.total_space, BINARY), "cyan", false),
                disk.used_percent,
                self.colorize(&format_size(results.totals.total_bytes, BINARY), "green", true),
                scan_percent
            );
        }

        println!(
            "Mode: {}  |  Files: {}  |  Dirs: {}",
            self.colorize(&results.mode, "yellow", false),
            results.totals.file_count,
            results.totals.dir_count
        );
    }

    fn print_buckets(&self, buckets: &[Bucket], total_bytes: u64) {
        if buckets.is_empty() {
            println!("No files found.");
            return;
        }

        let max_label_len = buckets.iter().map(|b| b.label.len()).max().unwrap_or(0).max(8);
        let max_size_len = buckets
            .iter()
            .map(|b| format_size(b.bytes, BINARY).len())
            .max()
            .unwrap_or(0)
            .max(12);

        println!();
        println!(
            "{:<width$}  {:>size_width$}  {:>7}  {:>10}  {}",
            "Category",
            "Size",
            "Percent",
            "Files",
            "Distribution",
            width = max_label_len,
            size_width = max_size_len
        );
        println!("{}", "-".repeat(max_label_len + max_size_len + 60));

        for bucket in buckets {
            self.print_bucket(bucket, total_bytes, max_label_len, max_size_len);
        }
    }

    fn print_bucket(&self, bucket: &Bucket, _total_bytes: u64, label_width: usize, size_width: usize) {
        let size_str = format_size(bucket.bytes, BINARY);
        let percent_str = format!("{:>6.1}%", bucket.percent);

        // Format everything WITHOUT colors first to get proper alignment
        let formatted = format!(
            "{:<label_width$}  {:>size_width$}  {:>7}  {:>10}  ",
            bucket.label,
            size_str,
            percent_str,
            bucket.file_count,
            label_width = label_width,
            size_width = size_width
        );

        // Now apply colors to the formatted line
        if self.use_color {
            // Split the formatted line into parts to colorize individually
            let parts: Vec<&str> = formatted.split("  ").collect();

            // Colorize label based on percentage
            let colored_label = if bucket.percent > 20.0 {
                parts[0].red().bold().to_string()
            } else if bucket.percent > 10.0 {
                parts[0].yellow().to_string()
            } else {
                parts[0].to_string()
            };

            // Colorize size (green)
            let colored_size = parts[1].green().to_string();

            // Colorize percent (cyan)
            let colored_percent = parts[2].cyan().to_string();

            // File count stays uncolored
            let file_count = parts[3];

            // Create and colorize bar
            let bar = self.create_bar(bucket.percent, 30);

            println!("{}  {}  {}  {}  {}",
                colored_label,
                colored_size,
                colored_percent,
                file_count,
                bar
            );
        } else {
            // No colors - just print formatted line with bar
            let bar = self.create_bar(bucket.percent, 30);
            print!("{}", formatted);
            println!("{}", bar);
        }
    }

    fn create_bar(&self, percent: f64, max_width: usize) -> String {
        let filled = ((percent / 100.0) * max_width as f64) as usize;
        let filled = filled.min(max_width);
        let empty = max_width - filled;

        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        if self.use_color {
            if percent > 20.0 {
                bar.red().to_string()
            } else if percent > 10.0 {
                bar.yellow().to_string()
            } else {
                bar.green().to_string()
            }
        } else {
            bar
        }
    }

    fn print_top_files(&self, files: &[FileEntry]) {
        if files.is_empty() {
            return;
        }

        println!("{}", self.colorize("Top Largest Files", "cyan", true));
        println!("{}", "-".repeat(80));

        for (i, file) in files.iter().enumerate() {
            println!(
                "{:2}. {:>12}  {}",
                i + 1,
                self.colorize(&format_size(file.bytes, BINARY), "green", false),
                file.path
            );
        }
    }

    fn print_top_dirs(&self, dirs: &[DirEntry]) {
        if dirs.is_empty() {
            return;
        }

        println!("{}", self.colorize("Top Largest Directories", "cyan", true));
        println!("{}", "-".repeat(80));

        for (i, dir) in dirs.iter().enumerate() {
            println!(
                "{:2}. {:>12}  {}",
                i + 1,
                self.colorize(&format_size(dir.bytes, BINARY), "green", false),
                dir.path
            );
        }
    }

    fn print_warnings(&self, warnings: &[Warning]) {
        if warnings.is_empty() {
            return;
        }

        println!("{}", self.colorize("Warnings and Errors", "yellow", true));
        println!("{}", "-".repeat(80));
        println!("Skipped {} paths due to errors:", warnings.len());

        let display_count = warnings.len().min(10);
        for warning in warnings.iter().take(display_count) {
            println!("  {} - {}", warning.path, warning.error);
        }

        if warnings.len() > display_count {
            println!("  ... and {} more", warnings.len() - display_count);
        }
    }

    fn colorize(&self, text: &str, color: &str, bold: bool) -> String {
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

    pub fn render(&self, results: &ScanResults, output_file: Option<&std::path::Path>) -> io::Result<()> {
        let json = serde_json::to_string_pretty(results)?;

        if let Some(path) = output_file {
            std::fs::write(path, json)?;
        } else {
            println!("{}", json);
        }

        Ok(())
    }
}
