# spacemap Roadmap

## Implementation Status

### ✅ Completed Features (Phase 1 + Phase 2)

**Phase 1: Performance Optimization**
- ✅ 1.1 Parallel Directory Scanning (using jwalk + rayon)
- ✅ 1.2 Lazy Metadata Loading (skip modified() in type/size modes)
- ✅ 1.3 Memory Optimization (path interning with u32 IDs)
- ✅ 1.4 Progress Indicators (indicatif-based progress bars)
- ✅ 1.5 Early Directory Pruning (filter_entry for exclusions)

**Phase 2: Advanced Features**
- ✅ 2.1 Incremental/Resumable Scans (checkpoint/resume support)
- ✅ 2.2 Smart Caching (directory hash-based cache validation)
- ✅ 2.4 Duplicate File Detection (BLAKE3 progressive hashing)
- ✅ 2.5 Comparison Mode (before/after scan comparison)

### 🚧 Future Work

## Phase 1: Performance Optimization (✅ COMPLETED)

The current single-threaded implementation is slow for large directories (~100GB+). This phase focuses on making spacemap significantly faster.

### 1.1 Parallel Directory Scanning
**Problem:** Single-threaded `walkdir` is slow for large directories
**Solution:** Implement parallel filesystem traversal

- Use `rayon` or `jwalk` (parallel walkdir) for concurrent directory scanning
- Implement work-stealing queue for balanced load distribution
- Handle shared state (category_stats, dir_accumulator) with lock-free data structures or sharding
- Benchmark: Target 3-5x speedup on multi-core systems

**Impact:** High - Most significant performance improvement
**Complexity:** Medium - Requires careful synchronization

### 1.2 Lazy Metadata Loading
**Problem:** `metadata.modified()` system call on every file, even when not needed
**Solution:** Only load metadata fields when required by the current mode

- Skip `modified()` call unless using `--by age` mode
- Benchmark impact of metadata calls vs computation
- Consider batching metadata requests

**Impact:** Medium - Reduces syscalls by ~30% in type/size modes
**Complexity:** Low - Simple conditional logic

### 1.3 Memory Optimization for Directory Tracking
**Problem:** `HashMap<PathBuf, u64>` grows unbounded for deep directory trees
**Solution:** Optimize directory accumulation memory usage

- Use path interning (store paths once, reference by ID)
- Consider streaming directory size calculation instead of full accumulation
- Implement configurable memory limits with degraded functionality fallback

**Impact:** Medium - Reduces memory usage for deep hierarchies
**Complexity:** Medium

### 1.4 Progress Indicators
**Problem:** No feedback during long scans; user doesn't know if tool is frozen
**Solution:** Add real-time progress reporting

- Show current file count, bytes scanned, scan rate
- Display current directory being scanned
- Optional `--quiet` flag to disable
- Use `indicatif` crate for clean progress bars

**Impact:** Low (UX) - Doesn't improve speed but improves perceived performance
**Complexity:** Low

### 1.5 Early Directory Pruning
**Problem:** Exclusion patterns checked after directory is entered
**Solution:** Skip excluded directories before traversal

- Modify exclusion logic to skip directory descent entirely
- Add `--prune` flag for explicit directory blacklist
- Benchmark impact on large codebases with node_modules, target/, etc.

**Impact:** Medium - Massive speedup when excluding large directories
**Complexity:** Low

---

## Phase 2: Advanced Features

### 2.1 Incremental/Resumable Scans
For extremely large directories (TB+), support resuming interrupted scans

- Store partial scan state to disk (JSON/binary)
- `--resume <state-file>` flag to continue previous scan
- `--checkpoint-interval` to periodically save progress
- Detect filesystem changes since last checkpoint

**Use case:** NAS systems, backup servers, data lakes
**Complexity:** High

### 2.2 Smart Caching
Cache scan results and only rescan changed directories

- Store hash/timestamp of directory contents
- `--cached` mode: skip unchanged directories
- `--cache-dir` to specify cache location (default: `~/.cache/spacemap`)
- Cache invalidation strategies (timestamp, inode change detection)

**Use case:** Repeated scans of same directory (CI/CD, monitoring)
**Complexity:** High

### 2.3 Filesystem-Specific Optimizations
Leverage filesystem-specific features for faster scanning

- **ext4/XFS:** Use `getdents64` directly instead of libc wrappers
- **BTRFS:** Use subvolume-aware scanning
- **ZFS:** Leverage dataset properties for quick size queries
- Detect filesystem type and choose optimal strategy

**Complexity:** Very High - Platform-specific

### 2.4 Duplicate File Detection
Identify duplicate files by content hash

- `--find-duplicates` flag to enable
- Use fast hashing (xxHash, BLAKE3) instead of cryptographic hashes
- Progressive hashing: only hash files with same size
- Group duplicates in output with total wasted space

**Use case:** Deduplication, finding redundant backups
**Complexity:** Medium

### 2.5 Comparison Mode
Compare two directory scans to show changes over time

- `spacemap --compare <before.json> <after.json>`
- Show: added files, deleted files, size changes, moved files
- Diff-style output showing space growth/reduction per category

**Use case:** Monitoring storage growth, understanding what changed
**Complexity:** Medium

---

## Phase 3: Output & Visualization

### 3.1 Interactive TUI Mode
Terminal UI for exploring results interactively

- Navigate directory tree with arrow keys
- Expand/collapse directories
- Filter by file type, size, age
- Delete files directly from TUI (with confirmation)
- Built with `ratatui` (successor to `tui-rs`)

**Complexity:** High

### 3.2 Export Formats
Support additional output formats

- CSV export for spreadsheet analysis
- HTML report with embedded charts
- SQLite database for complex queries
- Prometheus metrics format for monitoring integration

**Complexity:** Low-Medium (per format)

### 3.3 Visualization Enhancements
Improve terminal visualizations

- Treemap visualization (ASCII art)
- Sunburst chart (if terminal supports Unicode box drawing)
- Color-coded directory depth visualization
- Configurable color schemes

**Complexity:** Medium

---

## Phase 4: Analysis Features

### 4.1 Smart Recommendations
Provide actionable cleanup suggestions

- Identify: old build artifacts, package manager caches, temp files
- Built-in patterns for common space wasters (node_modules, target/, __pycache__)
- `--suggest-cleanup` flag with safe-to-delete recommendations
- Integrate with `.gitignore` and `.dockerignore` for cleanup hints

**Complexity:** Medium

### 4.2 Storage Policies
Define and check storage policies

- Config file (`.spacemap.toml`) with rules:
  - Max age for temp files
  - Max size for log directories
  - Required free space thresholds
- `--check-policy` exits with error if violations found
- Integration with monitoring systems

**Use case:** Automated storage compliance checking
**Complexity:** Medium

### 4.3 File Classification
Advanced categorization beyond basic file extensions

- Use `tree-magic-mini` or similar for content-based type detection
- Identify: generated files, downloaded files, user-created files
- Machine learning models (optional, via feature flag)
- Custom classification rules via config

**Complexity:** High

---

## Phase 5: Integration & Tooling

### 5.1 Watch Mode
Continuous monitoring of directory changes

- `spacemap --watch` to run continuously
- Detect filesystem events (inotify/FSEvents/ReadDirectoryChangesW)
- Real-time updates to size statistics
- Alert on threshold breaches

**Use case:** Monitoring production systems
**Complexity:** High

### 5.2 HTTP API Server
Run spacemap as a service with REST API

- `spacemap serve --port 8080`
- Endpoints: `/scan`, `/results`, `/compare`, `/health`
- Web UI for visualization (separate frontend)
- Authentication and rate limiting

**Complexity:** High

### 5.3 Grafana/Prometheus Integration
Export metrics for monitoring dashboards

- Prometheus exporter mode: `spacemap --export-prometheus`
- Expose: total size, file count, growth rate per category
- Grafana dashboard template included
- Alert rules for disk space thresholds

**Complexity:** Medium

### 5.4 CI/CD Integration
Fail builds if repository size exceeds limits

- GitHub Action: `spacemap-action`
- GitLab CI template
- Pre-commit hook support
- Policy enforcement in pipelines

**Complexity:** Low-Medium

---

## Phase 6: Platform & Distribution

### 6.1 Package Manager Support
Make installation easier across platforms

- Homebrew formula for macOS/Linux
- Apt/Yum repositories for Debian/RHEL
- Chocolatey package for Windows
- AUR package for Arch Linux
- Nix package

**Complexity:** Low-Medium (per platform)

### 6.2 Docker Image
Containerized scanning for cloud environments

- Official Docker image: `spacemap:latest`
- Multi-arch builds (amd64, arm64)
- Volume mounting for scanning host directories
- Minimal Alpine-based image (<10MB)

**Use case:** Kubernetes storage monitoring, cloud scanning
**Complexity:** Low

### 6.3 Windows Optimization
Improve Windows performance and UX

- Native Windows path handling
- NTFS-specific optimizations
- Windows Terminal color support
- Installer with PATH setup

**Complexity:** Medium

---

## Implementation Priority

### Immediate (Next 2-4 weeks)
1. **Phase 1.1: Parallel scanning** - Biggest performance win
2. **Phase 1.2: Lazy metadata loading** - Quick improvement
3. **Phase 1.4: Progress indicators** - UX improvement while scanning
4. **Phase 1.5: Early directory pruning** - Easy optimization

### Short-term (1-3 months)
5. **Phase 2.4: Duplicate detection** - High user value
6. **Phase 2.5: Comparison mode** - Monitoring use case
7. **Phase 3.2: CSV/HTML export** - Expand use cases
8. **Phase 4.1: Smart recommendations** - Actionable insights

### Medium-term (3-6 months)
9. **Phase 2.2: Smart caching** - Repeated scan optimization
10. **Phase 3.1: Interactive TUI** - Power user feature
11. **Phase 5.3: Prometheus integration** - Ops/monitoring use case
12. **Phase 6.1: Package managers** - Easier distribution

### Long-term (6+ months)
13. **Phase 2.1: Resumable scans** - Edge case for massive directories
14. **Phase 5.2: HTTP API server** - Advanced integration
15. **Phase 2.3: Filesystem-specific optimizations** - Diminishing returns
16. **Phase 4.3: ML-based classification** - Research project

---

## Success Metrics

### Performance
- **Parallel scanning:** 3-5x faster on 8-core systems for 100GB+ directories
- **Memory usage:** <100MB for most scans, <500MB for TB-scale scans
- **Time to first result:** <1s for directories under 10K files

### Adoption
- 10K+ downloads on crates.io (currently tracking toward this)
- Packaged in major Linux distros
- 100+ GitHub stars

### Quality
- <1% error rate on permission-denied paths
- Accurate to within 1% of `du -sh` results
- Zero crashes on malformed filesystems

---

## Non-Goals

What spacemap will NOT do:
- File recovery or undelete functionality
- Antivirus or malware scanning
- Network filesystem optimization (NFS, SMB) - out of scope
- GUI application (TUI only)
- File compression or archiving
