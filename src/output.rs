use crate::types::{Bucket, DirEntry, FileEntry, ScanResults, Warning};
use colored::*;
use humansize::{format_size, BINARY};
use std::io;

const BOX_TL: &str = "╭";
const BOX_TR: &str = "╮";
const BOX_BL: &str = "╰";
const BOX_BR: &str = "╯";
const BOX_H: &str = "─";
const BOX_V: &str = "│";
const BOX_LT: &str = "├";
const BOX_RT: &str = "┤";

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
        self.print_buckets(&results.buckets, results.totals.total_bytes);

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
        let width = 62;

        // Top border
        println!(
            "  {}{}{}",
            self.colorize(BOX_TL, "bright_black", false),
            self.colorize(&BOX_H.repeat(width), "bright_black", false),
            self.colorize(BOX_TR, "bright_black", false)
        );

        // Title line
        let title = format!("  Storage Analysis: {}", results.scanned_path);
        let title_display = if title.len() > width - 2 {
            format!("{}...", &title[..width - 5])
        } else {
            title.clone()
        };
        let padding = width - title_display.chars().count();
        println!(
            "  {} {}{} {}",
            self.colorize(BOX_V, "bright_black", false),
            self.colorize(&title_display, "cyan", true),
            " ".repeat(padding),
            self.colorize(BOX_V, "bright_black", false)
        );

        // Separator
        println!(
            "  {}{}{}",
            self.colorize(BOX_LT, "bright_black", false),
            self.colorize(&BOX_H.repeat(width), "bright_black", false),
            self.colorize(BOX_RT, "bright_black", false)
        );

        // Disk usage line
        if let Some(disk) = &results.disk_usage {
            let scan_percent = if disk.total_space > 0 {
                (results.totals.total_bytes as f64 / disk.total_space as f64) * 100.0
            } else {
                0.0
            };

            let disk_line = format!(
                "  Disk: {} / {} ({:.1}% used)",
                format_size(disk.used_space, BINARY),
                format_size(disk.total_space, BINARY),
                disk.used_percent
            );
            let disk_padding = width - disk_line.chars().count();

            println!(
                "  {} {} {} / {} ({:.1}% used){} {}",
                self.colorize(BOX_V, "bright_black", false),
                self.colorize(" Disk:", "white", false),
                self.colorize(&format_size(disk.used_space, BINARY), "yellow", false),
                self.colorize(&format_size(disk.total_space, BINARY), "blue", false),
                disk.used_percent,
                " ".repeat(disk_padding.saturating_sub(45)),
                self.colorize(BOX_V, "bright_black", false)
            );

            let scanned_line = format!(
                "  Scanned: {} ({:.2}% of disk)",
                format_size(results.totals.total_bytes, BINARY),
                scan_percent
            );
            let scanned_padding = width - scanned_line.chars().count();

            println!(
                "  {} {} {} ({:.2}% of disk){} {}",
                self.colorize(BOX_V, "bright_black", false),
                self.colorize(" Scanned:", "white", false),
                self.colorize(&format_size(results.totals.total_bytes, BINARY), "green", true),
                scan_percent,
                " ".repeat(scanned_padding.saturating_sub(30)),
                self.colorize(BOX_V, "bright_black", false)
            );
        }

        // Stats line
        let stats_content = format!(
            "  Mode: {}  │  Files: {}  │  Dirs: {}",
            results.mode,
            results.totals.file_count,
            results.totals.dir_count
        );
        let stats_padding = width.saturating_sub(stats_content.chars().count());

        println!(
            "  {} {} {}  {}  {} {}  {}  {} {}{} {}",
            self.colorize(BOX_V, "bright_black", false),
            self.colorize(" Mode:", "white", false),
            self.colorize(&results.mode, "magenta", false),
            self.colorize("│", "bright_black", false),
            self.colorize("Files:", "white", false),
            self.colorize(&results.totals.file_count.to_string(), "cyan", false),
            self.colorize("│", "bright_black", false),
            self.colorize("Dirs:", "white", false),
            self.colorize(&results.totals.dir_count.to_string(), "cyan", false),
            " ".repeat(stats_padding.saturating_sub(25)),
            self.colorize(BOX_V, "bright_black", false)
        );

        // Bottom border
        println!(
            "  {}{}{}",
            self.colorize(BOX_BL, "bright_black", false),
            self.colorize(&BOX_H.repeat(width), "bright_black", false),
            self.colorize(BOX_BR, "bright_black", false)
        );
    }

    fn print_buckets(&self, buckets: &[Bucket], total_bytes: u64) {
        if buckets.is_empty() {
            println!("  No files found.");
            return;
        }

        let max_label_len = buckets
            .iter()
            .map(|b| b.label.len())
            .max()
            .unwrap_or(0)
            .max(10);
        let max_size_len = buckets
            .iter()
            .map(|b| format_size(b.bytes, BINARY).len())
            .max()
            .unwrap_or(0)
            .max(10);

        // Table header
        println!(
            "  {}  {:>size_width$}  {:>8}  {:>8}  {}",
            self.colorize(
                &format!("{:<width$}", "CATEGORY", width = max_label_len),
                "bright_black",
                true
            ),
            self.colorize("SIZE", "bright_black", true),
            self.colorize("PERCENT", "bright_black", true),
            self.colorize("FILES", "bright_black", true),
            self.colorize("DISTRIBUTION", "bright_black", true),
            size_width = max_size_len
        );

        // Separator line
        println!(
            "  {}",
            self.colorize(
                &format!(
                    "{}  {}  {}  {}  {}",
                    "─".repeat(max_label_len),
                    "─".repeat(max_size_len),
                    "─".repeat(8),
                    "─".repeat(8),
                    "─".repeat(24)
                ),
                "bright_black",
                false
            )
        );

        for bucket in buckets {
            self.print_bucket(bucket, total_bytes, max_label_len, max_size_len);
        }
    }

    fn print_bucket(
        &self,
        bucket: &Bucket,
        _total_bytes: u64,
        label_width: usize,
        size_width: usize,
    ) {
        let size_str = format_size(bucket.bytes, BINARY);
        let percent_str = format!("{:.1}%", bucket.percent);

        // Get category icon
        let icon = self.get_category_icon(&bucket.label);

        if self.use_color {
            // Format each column separately
            let label_formatted = format!(
                "{} {:<width$}",
                icon,
                bucket.label,
                width = label_width - 2
            );
            let size_formatted = format!("{:>width$}", size_str, width = size_width);
            let percent_formatted = format!("{:>7}", percent_str);
            let count_formatted = format!("{:>8}", bucket.file_count);

            // Color based on percentage thresholds
            let (label_color, intensity) = if bucket.percent > 50.0 {
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

            let colored_label = self.colorize(&label_formatted, label_color, intensity);
            let colored_size = self.colorize(&size_formatted, "green", false);
            let colored_percent = self.colorize(&percent_formatted, "cyan", false);
            let colored_count = self.colorize(&count_formatted, "blue", false);

            // Create gradient bar
            let bar = self.create_gradient_bar(bucket.percent, 24);

            println!(
                "  {}  {}  {}  {}  {}",
                colored_label, colored_size, colored_percent, colored_count, bar
            );
        } else {
            let bar = self.create_gradient_bar(bucket.percent, 24);
            println!(
                "  {} {:<label_width$}  {:>size_width$}  {:>7}  {:>8}  {}",
                icon,
                bucket.label,
                size_str,
                percent_str,
                bucket.file_count,
                bar,
                label_width = label_width - 2,
                size_width = size_width
            );
        }
    }

    fn get_category_icon(&self, label: &str) -> &'static str {
        match label.to_lowercase().as_str() {
            "other" => "◆",
            "binaries" => "⚙",
            "code" => "◇",
            "archives" => "▣",
            "config" => "⚡",
            "documents" => "▤",
            "fonts" => "◈",
            "images" => "▦",
            "spreadsheets" => "▥",
            "audio" => "♪",
            "video" => "▶",
            "logs" => "▧",
            "databases" => "◉",
            // Size categories
            "tiny (<1kb)" | "tiny" => "·",
            "small (1-100kb)" | "small" => "○",
            "medium (100kb-10mb)" | "medium" => "●",
            "large (10-100mb)" | "large" => "◉",
            "huge (100mb-1gb)" | "huge" => "◎",
            "massive (>1gb)" | "massive" => "⬤",
            // Age categories
            "today" => "★",
            "this week" => "☆",
            "this month" => "◆",
            "this year" => "◇",
            "older" => "○",
            _ => "•",
        }
    }

    fn create_gradient_bar(&self, percent: f64, max_width: usize) -> String {
        let filled = ((percent / 100.0) * max_width as f64) as usize;
        let filled = filled.min(max_width);

        if !self.use_color {
            let empty = max_width - filled;
            return format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        }

        // Create gradient effect with different characters and colors
        let mut bar = String::new();

        for i in 0..max_width {
            if i < filled {
                // Gradient from green to yellow to red based on position in the filled area
                let position_percent = (i as f64 / max_width as f64) * 100.0;
                let char_colored = if position_percent > 60.0 || percent > 50.0 {
                    "█".red().to_string()
                } else if position_percent > 30.0 || percent > 20.0 {
                    "█".yellow().to_string()
                } else {
                    "█".green().to_string()
                };
                bar.push_str(&char_colored);
            } else {
                bar.push_str(&"░".bright_black().to_string());
            }
        }

        bar
    }

    fn print_top_files(&self, files: &[FileEntry]) {
        if files.is_empty() {
            return;
        }

        println!(
            "  {} {}",
            self.colorize("▼", "cyan", false),
            self.colorize("Top Largest Files", "cyan", true)
        );
        println!(
            "  {}",
            self.colorize(&"─".repeat(60), "bright_black", false)
        );

        for (i, file) in files.iter().enumerate() {
            let rank = format!("{:>2}.", i + 1);
            let size = format_size(file.bytes, BINARY);

            // Truncate path if too long
            let max_path_len = 45;
            let path_display = if file.path.len() > max_path_len {
                format!("...{}", &file.path[file.path.len() - max_path_len + 3..])
            } else {
                file.path.clone()
            };

            println!(
                "  {} {:>10}  {}",
                self.colorize(&rank, "bright_black", false),
                self.colorize(&size, "green", false),
                self.colorize(&path_display, "white", false)
            );
        }
    }

    fn print_top_dirs(&self, dirs: &[DirEntry]) {
        if dirs.is_empty() {
            return;
        }

        println!(
            "  {} {}",
            self.colorize("▼", "cyan", false),
            self.colorize("Top Largest Directories", "cyan", true)
        );
        println!(
            "  {}",
            self.colorize(&"─".repeat(60), "bright_black", false)
        );

        for (i, dir) in dirs.iter().enumerate() {
            let rank = format!("{:>2}.", i + 1);
            let size = format_size(dir.bytes, BINARY);

            // Truncate path if too long
            let max_path_len = 45;
            let path_display = if dir.path.len() > max_path_len {
                format!("...{}", &dir.path[dir.path.len() - max_path_len + 3..])
            } else {
                dir.path.clone()
            };

            println!(
                "  {} {:>10}  {}",
                self.colorize(&rank, "bright_black", false),
                self.colorize(&size, "green", false),
                self.colorize(&path_display, "white", false)
            );
        }
    }

    fn print_warnings(&self, warnings: &[Warning]) {
        if warnings.is_empty() {
            return;
        }

        println!(
            "  {} {}",
            self.colorize("⚠", "yellow", false),
            self.colorize(
                &format!("Warnings ({} paths skipped)", warnings.len()),
                "yellow",
                true
            )
        );
        println!(
            "  {}",
            self.colorize(&"─".repeat(60), "bright_black", false)
        );

        let display_count = warnings.len().min(5);
        for warning in warnings.iter().take(display_count) {
            let max_path_len = 40;
            let path_display = if warning.path.len() > max_path_len {
                format!("...{}", &warning.path[warning.path.len() - max_path_len + 3..])
            } else {
                warning.path.clone()
            };
            println!(
                "    {} {}",
                self.colorize(&path_display, "bright_black", false),
                self.colorize(&format!("({})", warning.error), "red", false)
            );
        }

        if warnings.len() > display_count {
            println!(
                "    {}",
                self.colorize(
                    &format!("... and {} more", warnings.len() - display_count),
                    "bright_black",
                    false
                )
            );
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
