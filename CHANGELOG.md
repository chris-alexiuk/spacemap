# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.2] - 2026-01-10

(Same as 1.1.0 - republished due to yanked version)

## [1.1.0] - 2026-01-10 (YANKED)

### Added
- **TOML configuration support** - Configure spacemap via `~/.config/spacemap/config.toml`
- **Custom file type categories** - Define entirely new categories with custom extensions (e.g., "ML Models" for .pt, .onnx, .h5 files)
- **Extension remapping** - Reassign existing extensions to different categories
- **Color customization** with priority system:
  - Per-extension colors (highest priority) - e.g., .rs → red, .py → green
  - Per-category colors - e.g., all "Code" files → cyan
  - Percentage-based fallback (default behavior)
- **CLI flag `--config <path>`** - Override default config location
- Representative extension tracking - Most common extension per category used for color resolution

### Changed
- TypeCategorizer now uses HashMap-based approach instead of static match statements
- Bucket struct includes optional `color` and `representative_extension` fields
- Collector tracks extension frequencies per category for accurate color resolution

### Technical Details
- Config format: TOML (using `toml = "0.8"` crate)
- Config location: `~/.config/spacemap/config.toml` (XDG standard, cross-platform via `dirs = "5.0"`)
- Error handling: Clear error messages for invalid config files
- Backward compatibility: 100% compatible - works identically without config file

### Example Config
```toml
# Custom categories
[[categories]]
name = "ML Models"
extensions = ["pt", "onnx", "h5", "pb", "keras"]
color = "magenta"

# Extension remapping
[[remaps]]
extensions = ["md", "txt"]
category = "Documentation"

# Category-level colors
[category_colors]
"Code" = "cyan"
"Images" = "blue"

# Per-extension colors (overrides category)
[extension_colors]
"rs" = "red"
"py" = "green"
"js" = "yellow"

# Display settings
[display]
use_percentage_colors = false  # Disable percentage-based coloring
```

## [1.0.7] - 2026-01-09

### Changed
- Remove unused dependencies (chrono and dashmap) to reduce dependency bloat

### Performance
- Improve parallel mode performance from 1.4x to 4.8x speedup on large directories
- Eliminate atomic operations in parallel scanner to reduce cache line contention
- Fix parallel mode merge bottleneck by using PathBuf directly instead of path pool interning

## [1.0.3] - 2026-01-09

### Fixed
- Removed docs.rs badge (binary-only crate, no library documentation)
- Clarified installation instructions about binary location
- Added GitHub Releases section to README
- Created GitHub release with pre-built binary

## [1.0.2] - 2026-01-09

### Changed
- Renamed GitHub repository from storage-check to spacemap
- Updated all repository URLs to reflect new name
- Updated installation instructions with new repository name

## [1.0.1] - 2026-01-09

### Changed
- Updated README.md to remove homelab references
- Fixed repository URL in Cargo.toml to point to standalone repo
- Fixed license badge to link to repository license section
- Added proper License section to README with contribution guidelines

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

[1.1.0]: https://github.com/chris-alexiuk/spacemap/releases/tag/v1.1.0
[1.0.7]: https://github.com/chris-alexiuk/spacemap/releases/tag/v1.0.7
[1.0.3]: https://github.com/chris-alexiuk/spacemap/releases/tag/v1.0.3
[1.0.2]: https://github.com/chris-alexiuk/spacemap/releases/tag/v1.0.2
[1.0.1]: https://github.com/chris-alexiuk/spacemap/releases/tag/v1.0.1
[1.0.0]: https://github.com/chris-alexiuk/spacemap/releases/tag/v1.0.0
