mod bounded_heap;
mod cache;
mod categorize;
mod checkpoint;
mod cli;
mod collector;
mod compare;
mod config;
mod duplicates;
mod output;
mod parallel_scanner;
mod path_pool;
mod progress;
mod scanner;
mod sharded_collector;
mod types;

use categorize::{AgeCategorizer, SizeCategorizer, TypeCategorizer};
use clap::Parser;
use cli::Cli;
use collector::SinglePassCollector;
use duplicates::DuplicateFinder;
use output::{JsonRenderer, TerminalRenderer};
use parallel_scanner::ParallelScanner;
use parking_lot::Mutex;
use progress::ScanProgress;
use scanner::Scanner;
use std::sync::Arc;
use sysinfo::Disks;
use types::{DiskUsage, ScanResults, Totals};

fn main() {
    let cli = Cli::parse();

    // Load configuration early
    let config = match config::SpacemapConfig::load(cli.config.as_ref()) {
        Ok(cfg) => Some(cfg),
        Err(e) => {
            if cli.config.is_some() {
                // User specified a config, so fail if it can't load
                eprintln!("Error loading config: {}", e);
                std::process::exit(2);
            } else {
                // No config or default doesn't exist - use defaults silently
                None
            }
        }
    };

    // Handle comparison mode
    if let Some(ref compare_paths) = cli.compare {
        if compare_paths.len() != 2 {
            eprintln!("Error: --compare requires exactly 2 file paths");
            std::process::exit(2);
        }

        let before = match compare::load_scan_results(&compare_paths[0]) {
            Ok(results) => results,
            Err(e) => {
                eprintln!("Error loading {}: {}", compare_paths[0].display(), e);
                std::process::exit(2);
            }
        };

        let after = match compare::load_scan_results(&compare_paths[1]) {
            Ok(results) => results,
            Err(e) => {
                eprintln!("Error loading {}: {}", compare_paths[1].display(), e);
                std::process::exit(2);
            }
        };

        let comparison = compare::compare_scans(&before, &after);
        let use_color = !cli.no_color && std::io::IsTerminal::is_terminal(&std::io::stdout());
        compare::print_comparison(&before, &after, &comparison, use_color);

        std::process::exit(0);
    }

    // Handle resume mode
    if let Some(ref resume_path) = cli.resume {
        match checkpoint::ScanCheckpoint::load(resume_path) {
            Ok(ckpt) => {
                if !cli.should_output_json() {
                    eprintln!("Resuming scan from checkpoint:");
                    eprintln!("  Started: {:?}", ckpt.started_at);
                    eprintln!("  Last checkpoint: {:?}", ckpt.last_checkpoint);
                    eprintln!("  Files scanned: {}", ckpt.stats.file_count);
                    eprintln!("  Bytes scanned: {}", ckpt.stats.total_bytes);
                    eprintln!();
                    eprintln!("Note: Resume restarts the scan with progress restored.");
                    eprintln!();
                }
            }
            Err(e) => {
                eprintln!("Error loading checkpoint from {}: {}", resume_path.display(), e);
                std::process::exit(2);
            }
        }
    }

    // Checkpoint not supported in parallel mode
    if cli.parallel && (cli.checkpoint.is_some() || cli.resume.is_some()) {
        eprintln!("Error: Checkpoint/resume not supported with --parallel mode");
        std::process::exit(2);
    }

    if let Err(e) = cli.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(2);
    }

    let path = cli.get_path();

    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", path.display());
        std::process::exit(2);
    }

    // Check cache if enabled
    if cli.cached {
        match cache::ScanCache::new(cli.cache_dir.clone()) {
            Ok(cache) => {
                if let Some(entry) = cache.get(&path) {
                    // Cache hit!
                    if !cli.should_output_json() {
                        eprintln!("Using cached results (scanned at {:?})", entry.last_scan);
                    }

                    let use_color = !cli.no_color && std::io::IsTerminal::is_terminal(&std::io::stdout());
                    let renderer = output::TerminalRenderer::new(use_color, cli.verbose);
                    renderer.render(&entry.results);
                    std::process::exit(0);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to load cache: {}", e);
            }
        }
    }

    // Only need modified time if using age categorization
    let need_modified = cli.by.as_str() == "age";
    let scanner = Scanner::new(cli.follow_symlinks, cli.max_depth, cli.exclude.clone(), need_modified);

    // Create categorizer based on mode
    let categorizer: Box<dyn categorize::Categorizer> = match cli.by.as_str() {
        "type" => Box::new(TypeCategorizer::with_config(config.as_ref())),
        "size" => {
            let custom_buckets = cli.size_buckets.as_ref().and_then(|s| parse_size_buckets(s));
            Box::new(SizeCategorizer::new(custom_buckets))
        }
        "age" => {
            let custom_buckets = cli.age_buckets.as_ref().and_then(|s| parse_age_buckets(s));
            Box::new(AgeCategorizer::new(custom_buckets))
        }
        _ => unreachable!(),
    };

    // Single-pass collection: categorize files, track top files/dirs in one scan
    let should_collect_tops = cli.verbose || cli.should_output_json();

    // Duplicate finder (only if requested)
    let dup_finder = if cli.find_duplicates {
        Some(Arc::new(Mutex::new(DuplicateFinder::new())))
    } else {
        None
    };

    // Progress indicator: only enabled if --progress flag is used (and not JSON output)
    let show_progress = cli.progress && !cli.should_output_json();
    let progress = ScanProgress::new(show_progress);

    // Create checkpoint if requested
    let mut checkpoint_data = if let Some(ref checkpoint_path) = cli.checkpoint {
        let ckpt = checkpoint::ScanCheckpoint::new(path.clone());
        // Save initial checkpoint
        if let Err(e) = ckpt.save(checkpoint_path) {
            eprintln!("Warning: Failed to create checkpoint: {}", e);
            None
        } else {
            if !cli.should_output_json() {
                eprintln!("Checkpoint enabled: {}", checkpoint_path.display());
            }
            Some((ckpt, checkpoint_path.clone()))
        }
    } else {
        None
    };

    let (stats, results) = if cli.parallel {
        // Parallel filesystem walking with thread-local collectors
        let parallel_scanner = ParallelScanner::new(
            cli.threads,
            cli.follow_symlinks,
            cli.max_depth,
            cli.exclude.clone(),
            need_modified,
        );

        let dup_finder_clone = dup_finder.clone();

        // Each thread processes into its own collector, then merge at end
        let (stats, collector) = parallel_scanner.scan(
            &path,
            categorizer,
            cli.top,
            should_collect_tops,
            &progress,
            |file_meta| {
                if let Some(ref df) = dup_finder_clone {
                    df.lock().add_file(file_meta.path.clone(), file_meta.size);
                }
            },
        );

        let results = collector.finalize(stats.total_bytes);
        (stats, results)
    } else {
        // Sequential scanning (original implementation)
        let mut collector = SinglePassCollector::new(categorizer, cli.top, should_collect_tops);
        let dup_finder_clone = dup_finder.clone();

        let checkpoint_params = checkpoint_data.as_mut().map(|(ckpt, path)| {
            (ckpt, path.as_path(), cli.checkpoint_interval)
        });

        let stats = scanner.scan(&path, |meta| {
            collector.process_file(meta.clone());
            if let Some(ref df) = dup_finder_clone {
                df.lock().add_file(meta.path.clone(), meta.size);
            }
        }, &progress, checkpoint_params);

        let results = collector.finalize(stats.total_bytes);
        (stats, results)
    };

    let buckets = results.buckets;
    let top_files = results.top_files;
    let top_dirs = results.top_dirs;

    // Find duplicates if requested
    let duplicates = if let Some(df) = dup_finder {
        let finder = Arc::try_unwrap(df)
            .unwrap_or_else(|_| panic!("Failed to unwrap duplicate finder"))
            .into_inner();

        let dup_groups = finder.find_duplicates();
        if dup_groups.is_empty() {
            None
        } else {
            Some(dup_groups.into_iter().map(|dg| types::DuplicateGroup {
                size: dg.size,
                hash: dg.hash,
                paths: dg.paths,
                wasted_space: dg.wasted_space,
            }).collect())
        }
    } else {
        None
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
        duplicates,
    };

    // Save to cache if enabled
    if cli.cached {
        if let Ok(mut cache) = cache::ScanCache::new(cli.cache_dir.clone()) {
            if let Err(e) = cache.put(path.clone(), results.clone()) {
                eprintln!("Warning: Failed to save cache: {}", e);
            }
        }
    }

    // Clean up checkpoint file after successful scan
    if let Some((_, checkpoint_path)) = checkpoint_data {
        let _ = std::fs::remove_file(&checkpoint_path);
    }

    if cli.should_output_json() {
        let renderer = JsonRenderer::new();
        if let Err(e) = renderer.render(&results, cli.output.as_deref()) {
            eprintln!("Error writing JSON output: {}", e);
            std::process::exit(3);
        }
    } else {
        let use_color = !cli.no_color && std::io::IsTerminal::is_terminal(&std::io::stdout());
        let renderer = TerminalRenderer::with_config(use_color, cli.verbose, config.as_ref());
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
