# Miktos DMO

## Digital Maintenance Organism

A Physarum polycephalum–inspired autonomous filesystem maintenance agent for macOS

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.77+-orange.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-24C8D8.svg)](https://v2.tauri.app/)
[![Status: Phase 1](https://img.shields.io/badge/Status-Phase%201%20%E2%80%94%20Terrain%20Visualization-brightgreen)](https://github.com/LegnaPetiteTour/Miktos-DMO)

---

## What is DMO?

DMO treats your filesystem as a living ecosystem. Waste accumulations — caches, temp files, build artifacts, logs — are **chemoattractant**. A future population of Physarum-style particle agents navigate a 2D Voronoi treemap projection of your real filesystem, converge on high-waste zones, and propose safe cleanup actions.

The core thesis: **making a system visually alive is not a cosmetic feature — it is the product**. Every visual element maps 1:1 to real filesystem data. The organism's behavior *is* the maintenance logic.

> Inspired by Dr. Jeff Jones's (2010) mathematical model of *Physarum polycephalum* transport networks: agents sense chemoattractant, turn toward the highest concentration, deposit trail, and move — producing complex, organic navigation from four simple rules.

---

## Current State: Phase 1 ✅

A fully functional **Tauri desktop app** that:

1. Walks the filesystem with configurable depth (Rust backend, `walkdir`)
2. Classifies every file into one of 14 waste categories using a rule-based engine
3. Computes a `waste_score` per file using a research-derived heuristic formula
4. Persists results to SQLite (bundled, no server required)
5. Renders the filesystem as an **organic Voronoi treemap** — blue = clean, amber/red = waste-heavy
6. Supports drill-down navigation: click any territory to zoom into its subtree
7. Shows a category breakdown panel with per-category bar charts
8. Outputs a ranked waste report via CLI (`dmo`)

### Phase 1 Validation (real machine, `~/Library`)

| Metric | Value |
| --- | --- |
| Files scanned | 202,034 |
| Total size | 487.47 GB |
| Waste candidates | 88,703 |
| Identified waste | **180.77 GB** |
| Scan time | 61.9 s |
| False positives | 0 (verified) |

Top-ranked waste: AR Reality Composer cache blobs (64 MB each, score 0.65), SiriTTS binary caches (113 MB, score 0.57), VS Code / Cursor cache blobs. No user documents, databases, `.git` trees, or system files appeared in results.

---

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│  dmo-cli  (binary)           dmo-app  (Tauri desktop)        │
│  ─────────────────           ──────────────────────────      │
│  clap argument parsing       Svelte 5 frontend               │
│  Text / JSON output          Voronoi treemap canvas          │
│  Scan presets                Drill-down navigation           │
└────────────────────┬─────────────────────┬──────────────────┘
                     │                     │  IPC (Tauri commands)
                     ▼                     ▼
          ┌──────────────────────────────────────┐
          │            dmo-core  (library)        │
          │  ────────────────────────────────     │
          │  scanner.rs    — walkdir traversal    │
          │  classifier.rs — 14 waste categories  │
          │  scorer.rs     — waste_score formula  │
          │  pipeline.rs   — scan → classify →    │
          │                  score → sort         │
          │  db.rs         — SQLite persistence   │
          │  types.rs      — shared data types    │
          └──────────────────────────────────────┘
```

### Waste Score Formula

```text
waste_score(f) = size_weight(f) × age_weight(f) × type_risk(f) × (1 - recency_score(f))
```

| Component | Formula | Notes |
| --- | --- | --- |
| `size_weight` | `log10(bytes / 1024) / 7.0`, clamped [0,1] | Files < 1 KB → 0.0 |
| `age_weight` | `min(days_mtime / 365, 1.0)` | Unknown mtime → 0.5 (conservative) |
| `type_risk` | Lookup from `WasteCategory` | 0.00 for protected, 0.95 for app/pkg caches |
| `recency_score` | `e^(-days_atime / 30)` | Exponential decay; unknown atime → 0.3 |

### Waste Categories

| Category | `type_risk` | Notes |
| --- | --- | --- |
| `ApplicationCache` | 0.95 | `~/Library/Caches/` |
| `PackageManagerCache` | 0.95 | npm, pip, cargo, homebrew |
| `BrowserCache` | 0.90 | Chrome, Firefox, Safari, Brave |
| `TempFile` | 0.90 | `/tmp/`, `.swp`, `.DS_Store` |
| `BuildArtifact` | 0.85 | DerivedData, `node_modules`, `.next/cache` |
| `SystemLog` | 0.80 | `/logs/`, `.log.gz` |
| `StaleDownload` | 0.60 | `~/Downloads/` |
| `Unknown` | 0.10 | Unrecognized files |
| Protected categories | **0.00** | Documents, Databases, `AppBundle`, CloudSync, `.git` working trees |

Protected categories are never scored, never stored in the database, and are invisible to any future organism.

---

## Roadmap

| Phase | Goal | Status |
| --- | --- | --- |
| **0** | CLI scanner — classify, score, SQLite, ranked report | ✅ Complete |
| **1** | Tauri app + Voronoi treemap terrain visualization | ✅ Complete |
| **2** | WebGL2/WebGPU Physarum particle organisms on the terrain | 🔜 Next |
| **3** | Proposal system — organisms cluster → user approve/reject quarantine | Planned |
| **4** | Learning loop — adapt weights from user accept/reject history | Planned |
| **5** | ALife sandbox — evolving agent strategies, fitness-based selection | Research track |

### Phase 2 Preview — The Organisms

Phase 2 adds a WebGL2 Physarum particle simulation rendered on top of the terrain. Based on Jones (2010):

- Each agent senses chemoattractant in three directions (forward-left, forward, forward-right)
- Turns toward the highest concentration
- Deposits trail at current position
- Chemoattractant map is generated from `waste_score` data at each terrain cell

Dense organism clusters identify high-waste filesystem nodes → Phase 3 converts cluster position → Voronoi cell lookup → filesystem node → action proposal.

---

## Getting Started

### Prerequisites

- **Rust** 1.77+ — [rustup](https://rustup.rs/)
- **Node.js** 18+ — [nodejs.org](https://nodejs.org/)
- **Xcode Command Line Tools** (macOS): `xcode-select --install`

### CLI Tool

```bash
cd ~/Desktop/Miktos-DMO

# Build
cargo build --release

# Run tests (11 unit tests across classifier, scorer, db)
cargo test

# Scan your caches (safest first target)
./target/release/dmo --path ~/Library/Caches --max-depth 4 --top 30

# Scan broader
./target/release/dmo --path ~/Library --max-depth 6 --top 50

# JSON output
./target/release/dmo --path ~/Library/Caches --format json | jq '.top_waste[:5]'
```

**CLI flags:**

| Flag | Default | Description |
| --- | --- | --- |
| `--path / -p` | `.` | Directory to scan |
| `--max-depth / -d` | `8` | Max recursion depth |
| `--top / -n` | `25` | Top N waste candidates to display |
| `--db` | `dmo_scan.db` | SQLite output path |
| `--format / -f` | `text` | `text` or `json` |
| `--all-categories` | false | Include zero-score categories in summary |

### Desktop App

```bash
cd ~/Desktop/Miktos-DMO/dmo-app

# Install frontend dependencies
npm install

# Run in dev mode (hot-reload for frontend)
npm run tauri dev

# Build production bundle
npm run tauri build
```

First launch compiles all Tauri + Rust dependencies (~3–4 min). Subsequent launches: ~5 s.

---

## Project Structure

```text
Miktos-DMO/
├── Cargo.toml                    # Workspace root
├── dmo-core/                     # Library crate — all intelligence
│   └── src/
│       ├── types.rs              # WasteCategory, FileNode, ScanConfig, ScanSummary
│       ├── scanner.rs            # walkdir traversal, metadata extraction
│       ├── classifier.rs         # 14-category rule engine (6 unit tests)
│       ├── scorer.rs             # waste_score formula (4 unit tests)
│       ├── pipeline.rs           # scan → classify → score → sort orchestrator
│       ├── db.rs                 # SQLite: snapshots, file_nodes, action_log
│       └── lib.rs                # Public API
├── dmo-cli/                      # Binary crate (`dmo`)
│   └── src/main.rs               # clap CLI, text/JSON output
├── dmo-app/                      # Tauri desktop app
│   ├── src/                      # Svelte 5 frontend
│   │   ├── App.svelte            # Main app: controls, stats, navigation, tooltip
│   │   ├── lib/Treemap.svelte    # Voronoi treemap Canvas2D renderer
│   │   ├── lib/types.ts          # TypeScript interfaces matching Rust IPC structs
│   │   └── app.css               # Global design tokens and styles
│   └── src-tauri/src/
│       ├── lib.rs                # IPC commands: scan_filesystem, get_home_dir
│       └── main.rs               # Tauri Builder entry point
└── Documentation/
    └── DMO_Phase2_Research_Architecture.md   # Full research dossier (52 sources)
```

---

## Safety Model

DMO is built around a non-destructive safety pyramid:

```text
┌─────────────────────────────────────────────────────┐
│  LAYER 4 — User Interface                           │
│  All proposed actions shown before execution.       │
│  Organism pulses to signal pending action.          │
├─────────────────────────────────────────────────────┤
│  LAYER 3 — Quarantine Gate  (Phase 3)               │
│  Zero permanent deletion. All moves go to           │
│  ~/.dmo_quarantine/YYYY-MM-DD/ + metadata.          │
│  Uses `trash` crate for reversible operations.      │
├─────────────────────────────────────────────────────┤
│  LAYER 2 — Confidence Threshold + Strict Denylist   │
│  No action below 0.80 confidence without user OK.   │
│  Hard-coded denylist: Documents, Desktop,           │
│  app bundles, databases, .git, cloud sync.          │
├─────────────────────────────────────────────────────┤
│  LAYER 1 — Read-Only Observer (Phase 0–1 default)   │
│  System starts in observation-only mode.            │
│  No filesystem modifications of any kind.           │
└─────────────────────────────────────────────────────┘
```

**Privacy guarantee:** The Rust backend reads filesystem metadata only (path, size, mtime, atime, file type). File contents are never read. No data leaves the machine — SQLite database is local only. No telemetry.

---

## Scientific Foundation

This project's Physarum model is grounded in peer-reviewed research:

- **Jones, J. (2010).** "Characteristics of Pattern Formation and Evolution in Approximations of Physarum Transport Networks." *Artificial Life, 16*(2), 127–153. [DOI](https://doi.org/10.1162/artl.2010.16.2.16202)
- **Balzer & Deussen (2005).** "Voronoi Treemaps." *IEEE InfoVis.* [DOI](https://doi.org/10.1109/INFVIS.2005.1532128)
- **Nakagaki et al. (2000).** Slime mold solves Tokyo metro. *Nature, 407*, 470. [DOI](https://doi.org/10.1038/35035159)

See [`Documentation/DMO_Phase2_Research_Architecture.md`](Documentation/DMO_Phase2_Research_Architecture.md) for the full research dossier with 52 verified sources.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, branch conventions, and the PR process.

---

## License

[MIT](LICENSE) © 2026 LegnaPetiteTour
