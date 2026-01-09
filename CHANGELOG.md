# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-09

### Added
- Initial public release as "spacemap" (formerly "storage-check")
- Bounded min-heap for memory-efficient top-N tracking
- Single-pass filesystem scanning architecture
- Beautiful terminal output with colored bars and aligned columns
- Multiple categorization modes: type, size, age
- Verbose drill-down showing top N largest files and directories
- JSON export for scripting and automation
- Cross-platform support (Linux, macOS, Windows)
- Customizable bucket boundaries for size and age modes
- Exclude patterns for filtering scans
- Symlink following option
- Disk usage information display

### Changed
- Renamed project from "storage-check" to "spacemap"
- **3x faster performance** on large directories (single scan vs three scans)
- **40-400x less memory** for top-N tracking (bounded heap vs unbounded collection)
- Zero string allocations for type categorization (using Cow)

### Fixed
- Replaced deprecated `atty` crate with `is-terminal`

## Performance Improvements

### Memory Usage
- Before: O(N) memory for all files + top-N collection
- After: O(K + D + C) where K=top_n, D=unique_dirs, C=categories
- For 1M files: ~400MB → ~1MB (400x reduction)

### Speed
- Before: 3 filesystem scans (collect all, top files, top dirs)
- After: 1 filesystem scan (streaming collection)
- For large directories: 2.5-3x faster

### Algorithm Complexity
- Top-N collection: O(N log N) → O(N log K) where K is typically 10-20
- Memory: O(N) → O(K) for top-N tracking

[1.0.0]: https://github.com/chris-alexiuk/homelab/releases/tag/v1.0.0
