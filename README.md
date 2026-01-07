# storage-check

A beautiful, cross-platform CLI tool for analyzing disk space usage with developer-friendly terminal visualizations and JSON export.

## Features

- **Beautiful terminal output** with colored bars, aligned columns, and clear categorization
- **Disk usage information** showing total disk space, used space, and what percentage your scan represents
- **Multiple categorization modes**: by file type, size buckets, or file age
- **Verbose drill-down** showing top N largest files and directories
- **JSON export** for scripting and automation
- **Cross-platform** support (Linux, macOS, Windows)
- **Memory efficient** streaming aggregation
- **Customizable** bucket boundaries for size and age modes

## Installation

Build from source:

```bash
cd tools/storage-check
cargo build --release
```

The binary will be available at `target/release/storage-check`.

## Usage

### Basic usage

Scan current directory (default: categorize by file type):
```bash
storage-check
```

Scan a specific path:
```bash
storage-check /home/user/projects
```

### Categorization Modes

**By file type** (default):
```bash
storage-check --by type
```

**By size buckets**:
```bash
storage-check --by size
```

**By file age**:
```bash
storage-check --by age
```

### Verbose output

Show top 10 largest files and directories:
```bash
storage-check --verbose
```

Show top 20 items:
```bash
storage-check --verbose --top 20
```

### JSON export

Output to stdout:
```bash
storage-check --json
```

Write to file:
```bash
storage-check --output report.json
```

### Advanced options

**Limit recursion depth**:
```bash
storage-check --max-depth 3
```

**Exclude patterns**:
```bash
storage-check --exclude node_modules --exclude .git
```

**Follow symlinks** (disabled by default):
```bash
storage-check --follow-symlinks
```

**Custom size buckets** (comma-separated bytes):
```bash
storage-check --by size --size-buckets "1024,10240,102400,1048576"
```

**Custom age buckets** (comma-separated days):
```bash
storage-check --by age --age-buckets "1,7,30,90,365"
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
  ]
}
```

**Note**: `disk_usage` may be `null` if disk information cannot be retrieved.

## License

Part of the homelab infrastructure tools.
