# Changelog

All notable changes to Miktos DMO are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project uses [Conventional Commits](https://www.conventionalcommits.org/) and [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Planned
- Phase 2: WebGL2/WebGPU Physarum particle organism simulation on terrain
- Particle-to-Voronoi-cell hit detection for proposal generation
- Real-time chemoattractant diffusion and decay on the GPU

---

## [0.2.0] — 2026 — Phase 1: Terrain Visualization

### Added
- **Tauri 2.x desktop app** (`dmo-app`) with Svelte 5 frontend
- **Voronoi treemap renderer** using `d3-voronoi-treemap@1.1.1` and Canvas2D
  - 5-stop perceptual color gradient (deep ocean → teal → amber → orange → red)
  - Perceptual score curve: `perceptual(raw) = min(raw × 2.5, 1.0)^0.55` to stretch the low-end
  - Adaptive border alpha and inner glow for high-waste zones
  - Zone labels with text shadow; secondary size label for areas > 8000 px²
- **Drill-down navigation**: click any Voronoi territory to zoom into its filesystem subtree
  - View stack with breadcrumb nav and `Escape`/`Backspace` keyboard shortcuts
  - Back-navigation to any ancestor level via breadcrumb clicks
- **Floating cursor-tracked tooltip**: shows file name, size, waste score, and last-accessed time
- **Category breakdown panel**: toggleable sidebar with per-category bar charts colored by average score
- **Waste % stat**: `waste_size / total_size × 100` displayed alongside total size and file count
- **Scan spinner** and animated loading state (concentric breathing rings)
- **IPC commands**: `scan_filesystem(path, max_depth, denylist)` and `get_home_dir()` via Tauri
- **Tree builder** (`build_tree`): aggregates flat `Vec<FileNode>` → hierarchical `TreeNode` with depth ≤ 2 as top-level children, deeper nodes rolled up into their parent
- **`TreeNode` / `ScanResult`** TypeScript interfaces matching Rust IPC structs

### Fixed
- `@sveltejs/vite-plugin-svelte@4` incompatibility with Vite 6 → bumped to `^5.0.0`
- `#[tauri::command]` / `#[macro_export]` collision when commands defined at crate root → moved to `pub mod commands {}`
- Missing icon files causing `tauri::generate_context!()` panic → generated RGBA PNG placeholders
- `.icns` / `.ico` references in `tauri.conf.json` for non-existent files → removed

---

## [0.1.0] — 2026 — Phase 0: CLI Foundation

### Added
- **`dmo-core`** library crate with the complete scan/classify/score pipeline
  - `scanner.rs`: `walkdir`-based filesystem traversal; extracts path, size, mtime, atime; skips `.app` bundles, iCloud placeholders, denylisted paths
  - `classifier.rs`: two-phase first-match rule engine — protected categories checked first, waste categories second; all rules case-insensitive
  - `scorer.rs`: `waste_score = size_weight × age_weight × type_risk × (1 - recency_score)`, all components bounded [0, 1]
  - `db.rs`: SQLite persistence via `rusqlite` (bundled); tables: `scan_snapshots`, `file_nodes`, `action_log`
  - `types.rs`: `WasteCategory` (14 variants, 8 scorable / 6 protected), `FileNode`, `ScanConfig`, `ScanSummary`, `CategorySummary`
  - `pipeline.rs`: orchestrates scan → classify → score → sort; returns `ScanSummary` with ranked `top_waste`
- **`dmo-cli`** binary crate (`dmo`)
  - `clap`-based argument parsing: `--path`, `--max-depth`, `--top`, `--db`, `--format`, `--all-categories`
  - Text output: rich ranked waste report with ASCII table
  - JSON output: full `ScanSummary` serialized via `serde_json`
- **11 unit tests** across classifier (6), scorer (4), db (1) — all passing
- **`.gitignore`**: comprehensive rules for Rust, Tauri, Node.js, macOS, editors, secrets, SQLite databases

### Validated
- `~/Library/Caches` (12.14 GB, 4,973 candidates, 776 ms): zero false positives
- `~/Library` (487.47 GB, 88,703 candidates, 61.9 s): zero false positives
- Top-ranked waste correctly identifies: AR cache blobs (0.65), SiriTTS binaries (0.57), VS Code/Cursor caches

---

[Unreleased]: https://github.com/LegnaPetiteTour/Miktos-DMO/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/LegnaPetiteTour/Miktos-DMO/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/LegnaPetiteTour/Miktos-DMO/releases/tag/v0.1.0
