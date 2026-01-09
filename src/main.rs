mod bounded_heap;
mod categorize;
mod cli;
mod output;
mod scanner;
mod types;

use categorize::{aggregate_by_category, AgeCategorizer, SizeCategorizer, TypeCategorizer};
use clap::Parser;
use cli::Cli;
use output::{JsonRenderer, TerminalRenderer};
use scanner::Scanner;
use sysinfo::Disks;
use types::{DirEntry, DiskUsage, FileEntry, ScanResults, Totals};

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(2);
    }

    let path = cli.get_path();

    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", path.display());
        std::process::exit(2);
    }

    let scanner = Scanner::new(cli.follow_symlinks, cli.max_depth, cli.exclude.clone());

    let mut files = Vec::new();
    let stats = scanner.scan(&path, |meta| {
        files.push(meta);
    });

    let buckets = match cli.by.as_str() {
        "type" => {
            let categorizer = TypeCategorizer::new();
            aggregate_by_category(&categorizer, files, stats.total_bytes)
        }
        "size" => {
            let custom_buckets = cli.size_buckets.as_ref().and_then(|s| parse_size_buckets(s));
            let categorizer = SizeCategorizer::new(custom_buckets);
            aggregate_by_category(&categorizer, files, stats.total_bytes)
        }
        "age" => {
            let custom_buckets = cli.age_buckets.as_ref().and_then(|s| parse_age_buckets(s));
            let categorizer = AgeCategorizer::new(custom_buckets);
            aggregate_by_category(&categorizer, files, stats.total_bytes)
        }
        _ => unreachable!(),
    };

    let top_files: Vec<FileEntry> = if cli.verbose || cli.should_output_json() {
        scanner
            .collect_top_files(&path, cli.top)
            .into_iter()
            .map(|(path, bytes)| FileEntry {
                path: path.display().to_string(),
                bytes,
            })
            .collect()
    } else {
        Vec::new()
    };

    let top_dirs: Vec<DirEntry> = if cli.verbose || cli.should_output_json() {
        scanner
            .collect_top_dirs(&path, cli.top)
            .into_iter()
            .map(|(path, bytes)| DirEntry {
                path: path.display().to_string(),
                bytes,
            })
            .collect()
    } else {
        Vec::new()
    };

    let disk_usage = get_disk_usage(&path);

    let results = ScanResults {
        scanned_path: path.display().to_string(),
        mode: cli.by.clone(),
        totals: Totals {
            total_bytes: stats.total_bytes,
            file_count: stats.file_count,
            dir_count: stats.dir_count,
            skipped_paths: stats.warnings.len() as u64,
        },
        disk_usage,
        buckets,
        top_files,
        top_dirs,
        warnings: stats.warnings,
    };

    if cli.should_output_json() {
        let renderer = JsonRenderer::new();
        if let Err(e) = renderer.render(&results, cli.output.as_deref()) {
            eprintln!("Error writing JSON output: {}", e);
            std::process::exit(3);
        }
    } else {
        let use_color = !cli.no_color && std::io::IsTerminal::is_terminal(&std::io::stdout());
        let renderer = TerminalRenderer::new(use_color, cli.verbose);
        renderer.render(&results);
    }

    let exit_code = if results.warnings.is_empty() { 0 } else { 1 };
    std::process::exit(exit_code);
}

fn parse_size_buckets(spec: &str) -> Option<Vec<u64>> {
    let buckets: Result<Vec<u64>, _> = spec.split(',').map(|s| s.trim().parse::<u64>()).collect();
    buckets.ok()
}

fn parse_age_buckets(spec: &str) -> Option<Vec<u64>> {
    let buckets: Result<Vec<u64>, _> = spec.split(',').map(|s| s.trim().parse::<u64>()).collect();
    buckets.ok()
}

fn get_disk_usage(path: &std::path::Path) -> Option<DiskUsage> {
    let disks = Disks::new_with_refreshed_list();

    let canonical_path = path.canonicalize().ok()?;

    // Find the disk that contains this path
    let disk = disks.iter().find(|d| {
        canonical_path.starts_with(d.mount_point())
    })?;

    let total_space = disk.total_space();
    let available_space = disk.available_space();
    let used_space = total_space.saturating_sub(available_space);
    let used_percent = if total_space > 0 {
        (used_space as f64 / total_space as f64) * 100.0
    } else {
        0.0
    };

    Some(DiskUsage {
        total_space,
        available_space,
        used_space,
        used_percent,
    })
}
