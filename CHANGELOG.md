# Changelog

All notable changes to Miktos DMO are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project uses [Conventional Commits](https://www.conventionalcommits.org/) and [Semantic Versioning](https://semver.org/).

---

## [Unreleased] — Phase 2: Physarum Particle Organisms

Work begins after commit `7d0a323`. See [`Documentation/DMO_Phase2_ImplementationPlan.md`](Documentation/DMO_Phase2_ImplementationPlan.md) for the full implementation plan.

### Planned
- WebGL2 Physarum particle simulation rendered on the `.sim-overlay` canvas
- Chemoattractant texture built from `TerrainCell.waste_score` map (bridge already wired)
- Jones (2010) agent parameters: `SA=45°`, `SO=9px`, `RA=45°`, `stepSize=1`
- Density readback to detect organism clusters → Voronoi cell → filesystem node lookup
- Simulation activate/pause toggle in UI

---

## [0.2.1] — 2026-03-12 — Phase 1 Polish + Pre-Phase-2 Architecture

### Added — `7d0a323`
- **SQLite persistence wired into scan command** (`dmo-app/src-tauri/src/lib.rs`)
  - Every scan now persisted to `~/.dmo/history.db` via `DmoDb`
  - `scan_id` (i64 row ID) returned in every `ScanResult` IPC response
  - Best-effort: DB failure logs a warning but never fails the scan
- **`TerrainCell` exported type** (`dmo-app/src/lib/types.ts`)
  - Interface: `{ polygon: [number,number][]; path: string; waste_score: number; centroid: [number,number]; area: number; }`
  - Provides the filesystem→canvas coordinate mapping for the Phase 2 simulation
- **`onCells` callback on Treemap** (`dmo-app/src/lib/Treemap.svelte`)
  - Optional prop: `onCells?: (cells: TerrainCell[]) => void`
  - Fires after every Voronoi layout computation with the full terrain layout
  - App.svelte stores result in `terrainCells: TerrainCell[]` state
- **WebGL2 overlay canvas** (`dmo-app/src/App.svelte`)
  - `<canvas class="sim-overlay">` positioned absolutely above the terrain
  - `pointer-events: none` — terrain mouse interaction fully preserved
  - `display: none` until Phase 2 activates it via `getContext('webgl2')`
  - `overlayCanvas` ref bound and accessible in App scope

### Fixed — `7d0a323`
- **macOS `atime` unreliability in scorer** (`dmo-core/src/scorer.rs`)
  - APFS `noatime` mounts cause `metadata.accessed()` to mirror `mtime` or be stale
  - New heuristic: treat `atime` as genuine only when `atime > mtime`; otherwise fall back to `mtime` as a recency proxy
  - Unknown-time neutral fallback: `0.3` (pessimistic) → `0.5` (neutral)
  - Waste scoring measurably more accurate on APFS volumes

### Fixed — `7e2b7a2`
- **Scanner denylist blocked its own configured root**
  - Presets and folder picker for `/Applications`, `~/Documents`, `~/Desktop` returned zero results because the path was in `default_denylist()`
  - Fix: `.filter(|p| p != &root)` removes the root from the denylist before constructing `ScanConfig`

### Fixed — `128f4e3`
- **Presets and folder picker silently did not scan**
  - `setPreset()` only set `scanPath`/`maxDepth`, never called `startScan()`
  - `pickFolder()` set path from dialog, never scanned
  - Fix: both functions now `await startScan()` at the end

### Added — `3872e53`
- **Native folder picker** via `tauri-plugin-dialog`
  - Added npm package, Cargo dep, `"dialog:allow-open"` capability permission
  - `⋯` button next to path input opens macOS native "Choose Folder" sheet
- **"Scan here" button** in breadcrumb trail
  - `⊙ Scan here` appears when drilled into a subdirectory
  - Promotes drilled node to scan root, resets view stack, starts scan immediately
- **7 quick-access preset buttons**: `~/Caches`, `~/Downloads`, `~/Documents`, `~/Desktop`, `~/Library`, `Home dir`, `/Applications`
  - Expanded from the original 3 presets; all auto-scan on click

### Fixed — `1817a35`
- **Path input clipped long paths** — only showed first ~22 chars
  - `bind:this={pathInputEl}` + reactive `$effect` scrolls input `scrollLeft` to `scrollWidth` on every path change (when unfocused)
  - `focus` handler calls `select()` so user can replace the path immediately

### Fixed — `12bf2b5`
- **Path display strip** replaced two-line overlay approach with `word-break: break-all; white-space: normal` single element — full path always visible without ellipsis

### Added — `be98039`
- Address bar hover preview: hovered node path replaces current path in strip in real time
- Depth selector extended to 1–8 levels
- Path history dropdown: last 10 scanned paths, deduplicated, most recent first
- Category panel redesigned: score-colored bar charts, legend, context line

### Fixed — `d707cbf` `1a3a586`
- Voronoi parameters corrected: `minWeightRatio(0.01)`, `convergenceRatio(0.01)`, `maxIterationCount(150)`
- Replaced async Voronoi recompute with synchronous debounce — eliminated "No zones to display" flicker on resize

### Fixed — `a3d792e` `9aef3d2`
- Zero TypeScript errors baseline: resolved all strict-mode errors, Svelte 5 rune warnings, and Vite config issues

---

## [0.2.0] — 2026-03-11 — Phase 1: Terrain Visualization

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

[Unreleased]: https://github.com/LegnaPetiteTour/Miktos-DMO/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/LegnaPetiteTour/Miktos-DMO/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/LegnaPetiteTour/Miktos-DMO/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/LegnaPetiteTour/Miktos-DMO/releases/tag/v0.1.0
