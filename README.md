# spacemap

[![Crates.io](https://img.shields.io/crates/v/spacemap.svg)](https://crates.io/crates/spacemap)
[![Downloads](https://img.shields.io/crates/d/spacemap.svg)](https://crates.io/crates/spacemap)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/chris-alexiuk/spacemap#license)

A beautiful, high-performance CLI tool for analyzing disk space usage with developer-friendly terminal visualizations and JSON export.

📖 **[View Full Documentation](https://chris-alexiuk.github.io/spacemap/)**

## Features

**Core Functionality:**
- **Terminal visualizations** with colored bars, aligned columns, and clear categorization
- **Disk usage information** showing total disk space, used space, and scan percentage
- **Multiple categorization modes**: by file type, size buckets, or file age
- **Verbose drill-down** showing top N largest files and directories
- **JSON export** for scripting and automation
- **Cross-platform** support (Linux, macOS, Windows)

**Performance Optimizations:**
- **Parallel scanning** - multi-threaded directory traversal using jwalk and rayon
- **Lazy metadata loading** - skip unnecessary syscalls for 20-30% speedup
- **Early directory pruning** - exclude patterns skip entire directory trees
- **Path interning** - 50%+ memory reduction for deep hierarchies
- **Progress indicators** - real-time feedback during long scans

**Advanced Features:**
- **Duplicate detection** - find duplicate files using BLAKE3 progressive hashing (`--find-duplicates`)
- **Comparison mode** - compare two scans to show changes over time (`--compare`)
- **Smart caching** - cache results and skip unchanged directories (`--cached`)
- **Resumable scans** - checkpoint long scans and resume if interrupted (`--checkpoint`, `--resume`)

## Installation

### From crates.io (recommended)

```bash
cargo install spacemap
```

After installation, the `spacemap` command will be available in your PATH.

### From GitHub Releases

Download pre-built binaries from the [Releases page](https://github.com/chris-alexiuk/spacemap/releases).

### Build from source

```bash
git clone https://github.com/chris-alexiuk/spacemap
cd spacemap
cargo build --release
```

The compiled binary will be created at `target/release/spacemap`.

## Usage

### Basic usage

Scan current directory (default: categorize by file type):
```bash
spacemap
```

Scan a specific path:
```bash
spacemap /home/user/projects
```

### Categorization Modes

**By file type** (default):
```bash
spacemap --by type
```

**By size buckets**:
```bash
spacemap --by size
```

**By file age**:
```bash
spacemap --by age
```

### Verbose output

Show top 10 largest files and directories:
```bash
spacemap --verbose
```

Show top 20 items:
```bash
spacemap --verbose --top 20
```

### JSON export

Output to stdout:
```bash
spacemap --json
```

Write to file:
```bash
spacemap --output report.json
```

### Advanced options

**Limit recursion depth**:
```bash
spacemap --max-depth 3
```

**Exclude patterns**:
```bash
spacemap --exclude node_modules --exclude .git
```

**Follow symlinks** (disabled by default):
```bash
spacemap --follow-symlinks
```

**Custom size buckets** (comma-separated bytes):
```bash
spacemap --by size --size-buckets "1024,10240,102400,1048576"
```

**Custom age buckets** (comma-separated days):
```bash
spacemap --by age --age-buckets "1,7,30,90,365"
```

### Performance Features

**Parallel scanning** for faster performance on multi-core systems:
```bash
spacemap --parallel --threads 8
```

**Disable progress indicator** for faster scanning or scripting:
```bash
spacemap --quiet
```

### Advanced Features

**Find duplicate files**:
```bash
spacemap --find-duplicates --verbose
```

**Compare two scans** to see what changed:
```bash
spacemap /data --output before.json
# ... time passes, files change ...
spacemap /data --output after.json
spacemap --compare before.json after.json
```

**Use caching** for faster repeated scans:
```bash
spacemap /data --cached              # First run: cache miss, full scan
spacemap /data --cached              # Second run: cache hit if unchanged
```

**Checkpoint long scans** (resumable if interrupted):
```bash
spacemap /huge/directory --checkpoint scan.ckpt --checkpoint-interval 60
# If interrupted, resume with:
spacemap /huge/directory --resume scan.ckpt
```

## Example Output

### Type categorization (default)
```
Storage Analysis: /home/user/projects
======================================
Disk: 42.24 GiB / 97.87 GiB (43.2% used)  |  Scanned: 791.99 MiB (0.79% of disk)
Mode: type  |  Files: 2962  |  Dirs: 501


Category           Size  Percent       Files  Distribution
---------------------------------------------------------------------------------
Other        708.69 MiB    89.5%        2560  ██████████████████████████░░░░
Binaries      81.49 MiB    10.3%          18  ███░░░░░░░░░░░░░░░░░░░░░░░░░░░
Code           1.61 MiB     0.2%          37  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
Config       185.87 KiB     0.0%         331  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
Documents     15.57 KiB     0.0%          16  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
```

### Size categorization
```
Storage Analysis: /home/user/projects
======================================
Disk: 42.24 GiB / 97.87 GiB (43.2% used)  |  Scanned: 791.99 MiB (0.79% of disk)
Mode: size  |  Files: 2962  |  Dirs: 501


Category              Size  Percent       Files  Distribution
------------------------------------------------------------------------------------
1-10 MiB        505.87 MiB    63.9%         165  ███████████████████░░░░░░░░░░░
10-100 MiB      166.56 MiB    21.0%           7  ██████░░░░░░░░░░░░░░░░░░░░░░░░
100 KiB-1 MiB    98.75 MiB    12.5%         282  ███░░░░░░░░░░░░░░░░░░░░░░░░░░░
10-100 KiB       18.38 MiB     2.3%         522  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
1-10 KiB          2.17 MiB     0.3%         487  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
0-1 KiB         263.80 KiB     0.0%        1499  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
```

### Verbose mode
```
Top Largest Files
--------------------------------------------------------------------------------
 1.    31.71 MiB  /home/user/projects/target/debug/binary
 2.    18.20 MiB  /home/user/projects/target/debug/deps/libclap.rlib
 3.    12.45 MiB  /home/user/projects/data/dataset.csv
 4.     8.92 MiB  /home/user/projects/build/output.bin
 5.     5.33 MiB  /home/user/projects/assets/image.png

Top Largest Directories
--------------------------------------------------------------------------------
 1.   267.59 MiB  /home/user/projects/target/debug/deps
 2.   146.47 MiB  /home/user/projects/target/release/deps
 3.    70.04 MiB  /home/user/projects/node_modules
 4.    21.58 MiB  /home/user/projects/build
 5.    12.34 MiB  /home/user/projects/assets
```

## File Type Categories

The tool recognizes the following file type categories:

- **Images**: jpg, png, gif, svg, webp, etc.
- **Videos**: mp4, avi, mkv, mov, webm, etc.
- **Audio**: mp3, wav, flac, aac, ogg, etc.
- **Documents**: pdf, doc, docx, txt, md, etc.
- **Spreadsheets**: xls, xlsx, csv, ods
- **Presentations**: ppt, pptx, odp
- **Archives**: zip, tar, gz, 7z, rar, etc.
- **Code**: rs, py, js, java, c, cpp, go, etc.
- **Config**: json, xml, yaml, toml, ini, etc.
- **Binaries**: exe, dll, so, bin, etc.
- **Disk Images**: iso, img, dmg, vdi
- **Databases**: db, sqlite, sql
- **Logs**: log
- **Fonts**: ttf, otf, woff
- **Other**: all other files

## Exit Codes

- `0`: Success
- `1`: Scan completed with partial errors (some paths unreadable)
- `2`: Invalid arguments
- `3`: Runtime failure

## JSON Schema

When using `--json` or `--output`, the tool outputs the following structure:

```json
{
  "scanned_path": "string",
  "mode": "type|size|age",
  "totals": {
    "total_bytes": 0,
    "file_count": 0,
    "dir_count": 0,
    "skipped_paths": 0
  },
  "disk_usage": {
    "total_space": 0,
    "available_space": 0,
    "used_space": 0,
    "used_percent": 0.0
  },
  "buckets": [
    {
      "key": "string",
      "label": "string",
      "bytes": 0,
      "percent": 0.0,
      "file_count": 0
    }
  ],
  "top_files": [
    { "path": "string", "bytes": 0 }
  ],
  "top_dirs": [
    { "path": "string", "bytes": 0 }
  ],
  "warnings": [
    { "path": "string", "error": "string" }
  ],
  "duplicates": [
    {
      "size": 0,
      "hash": "string",
      "paths": ["string"],
      "wasted_space": 0
    }
  ]
}
```

**Note**: `disk_usage` may be `null` if disk information cannot be retrieved.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
