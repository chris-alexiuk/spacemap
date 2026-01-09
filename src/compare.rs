use crate::types::ScanResults;
use colored::*;
use humansize::{format_size, BINARY};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct ScanComparison {
    pub added_bytes: u64,
    pub removed_bytes: u64,
    pub added_files: u64,
    pub removed_files: u64,
    pub category_changes: HashMap<String, i64>,
}

pub fn load_scan_results(path: &Path) -> std::io::Result<ScanResults> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let results = serde_json::from_reader(reader)?;
    Ok(results)
}

pub fn compare_scans(before: &ScanResults, after: &ScanResults) -> ScanComparison {
    // Compare totals
    let added_bytes = after.totals.total_bytes.saturating_sub(before.totals.total_bytes);
    let removed_bytes = before.totals.total_bytes.saturating_sub(after.totals.total_bytes);
    let added_files = after.totals.file_count.saturating_sub(before.totals.file_count);
    let removed_files = before.totals.file_count.saturating_sub(after.totals.file_count);

    // Compare categories
    let mut category_changes = HashMap::new();

    // Build maps for easier comparison
    let mut before_map: HashMap<String, u64> = before
        .buckets
        .iter()
        .map(|b| (b.key.clone(), b.bytes))
        .collect();

    let after_map: HashMap<String, u64> = after
        .buckets
        .iter()
        .map(|b| (b.key.clone(), b.bytes))
        .collect();

    // Find changes
    for (key, after_bytes) in &after_map {
        let before_bytes = before_map.remove(key).unwrap_or(0);
        let delta = *after_bytes as i64 - before_bytes as i64;
        if delta != 0 {
            category_changes.insert(key.clone(), delta);
        }
    }

    // Remaining keys in before_map are categories that were removed
    for (key, before_bytes) in before_map {
        category_changes.insert(key, -(before_bytes as i64));
    }

    ScanComparison {
        added_bytes,
        removed_bytes,
        added_files,
        removed_files,
        category_changes,
    }
}

pub fn print_comparison(
    before: &ScanResults,
    after: &ScanResults,
    comparison: &ScanComparison,
    use_color: bool,
) {
    println!();
    println!("  {}", style_text("Storage Comparison", "cyan", true, use_color));
    println!("  {}", style_text(&"─".repeat(56), "bright_black", false, use_color));

    println!(
        "  Before: {} ({} files)",
        style_text(&before.scanned_path, "white", false, use_color),
        before.totals.file_count
    );
    println!(
        "  After:  {} ({} files)",
        style_text(&after.scanned_path, "white", false, use_color),
        after.totals.file_count
    );
    println!();

    // Overall changes
    println!("  {}", style_text("OVERALL CHANGES", "yellow", true, use_color));
    println!("  {}", style_text(&"─".repeat(56), "bright_black", false, use_color));

    if comparison.added_bytes > comparison.removed_bytes {
        let net_growth = comparison.added_bytes - comparison.removed_bytes;
        println!(
            "  Net growth: {} (+{})",
            style_text(&format_size(net_growth, BINARY), "red", true, use_color),
            style_text(&format!("{:.1}%", (net_growth as f64 / before.totals.total_bytes as f64) * 100.0), "red", false, use_color)
        );
    } else if comparison.removed_bytes > comparison.added_bytes {
        let net_reduction = comparison.removed_bytes - comparison.added_bytes;
        println!(
            "  Net reduction: {} (-{:.1}%)",
            style_text(&format_size(net_reduction, BINARY), "green", true, use_color),
            (net_reduction as f64 / before.totals.total_bytes as f64) * 100.0
        );
    } else {
        println!("  No net change in size");
    }

    if comparison.added_files > 0 {
        println!(
            "  Added {} files",
            style_text(&comparison.added_files.to_string(), "green", false, use_color)
        );
    }
    if comparison.removed_files > 0 {
        println!(
            "  Removed {} files",
            style_text(&comparison.removed_files.to_string(), "red", false, use_color)
        );
    }
    println!();

    // Category changes
    if !comparison.category_changes.is_empty() {
        println!("  {}", style_text("CATEGORY CHANGES", "yellow", true, use_color));
        println!("  {}", style_text(&"─".repeat(56), "bright_black", false, use_color));

        let mut changes: Vec<_> = comparison.category_changes.iter().collect();
        changes.sort_by(|a, b| b.1.abs().cmp(&a.1.abs()));

        for (category, delta) in changes.iter().take(10) {
            let before_label = before
                .buckets
                .iter()
                .find(|b| &b.key == *category)
                .map(|b| b.label.as_str())
                .unwrap_or(category);

            if **delta > 0 {
                println!(
                    "  {} +{}",
                    style_text(&format!("{:<20}", before_label), "white", false, use_color),
                    style_text(&format_size(**delta as u64, BINARY), "green", false, use_color)
                );
            } else {
                println!(
                    "  {} -{}",
                    style_text(&format!("{:<20}", before_label), "white", false, use_color),
                    style_text(&format_size(delta.unsigned_abs(), BINARY), "red", false, use_color)
                );
            }
        }
    }

    println!();
}

fn style_text(text: &str, color: &str, bold: bool, use_color: bool) -> String {
    if !use_color {
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
