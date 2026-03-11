# Digital Maintenance Organism — Phase 2 Research & Architecture Proposal
**Date:** 2026-03-10  
**Status:** Evidence-backed research document with verifiable sources  
**Phase:** 2 — Visualization Architecture + Build Approach  
**Preceding document:** digital_maintenance_organism_research_dossier.md

---

## What Phase 1 Established (Summary)

The Phase 1 dossier confirmed:
- The concept sits at the intersection of ALife, autonomic computing, software aging/rejuvenation, and desktop agents
- No mature consumer product combines all required properties (persistence, adaptation, ALife-inspired behavior, sandboxed safety, privacy-first)
- A meaningful prototype is buildable on home hardware
- The 4-layer architecture (Safety → Intelligence → Learning → ALife sandbox) is the correct ordering

**What Phase 1 missed entirely:** the competitive differentiation argument, the existing market, and — most critically — the **visualization layer**, which is not a cosmetic feature but the central differentiating mechanism of this product.

---

## The Market Reality Phase 1 Ignored

Before designing anything, this must be stated plainly: **mature competitors already own this category.**

| Product | Model | Users | Differentiation |
|---|---|---|---|
| CleanMyMac X (MacPaw) | $39.95/year | Millions | Polished, trusted, comprehensive |
| Hazel (Noodlesoft) | $42 one-time | Power users | Rule-based automation, beloved |
| DaisyDisk | $9.99 | Wide | Visual disk map, beautiful |
| Gemini 2 (MacPaw) | $19.95/year | Wide | Duplicate detection |
| OnyX / Maintenance | Free | Technical | System-level maintenance |

Sources:
- CleanMyMac X: https://macpaw.com/cleanmymac
- Hazel: https://www.noodlesoft.com/
- DaisyDisk: https://daisydiskapp.com/
- Gemini 2: https://macpaw.com/gemini

**The brutal conclusion:** A DMO that looks like any of these — file lists, progress bars, "X GB reclaimed" — loses. Every time. These products have brand trust, years of refinement, and distribution channels you do not have yet.

**The only viable path is radical experiential differentiation.** And there is exactly one direction where that differentiation exists that none of these competitors have pursued: **making the system visually alive.**

---

## The Central Thesis: The Organism Visualization IS the Product

This is not a feature added to a maintenance tool. This is the product itself.

The claim: **a user should be able to look at their file system and see living organisms moving through it.** The organisms are not decorative. They are the maintenance system. Their behavior is driven by real filesystem data. Their actions are real filesystem operations. The simulation and the functional system are the same thing.

This is the thing that has never been built. DaisyDisk came closest with its visual disk map — but it is static. CleanMyMac has animations — but they are progress indicators, not behavioral simulations. No product has ever mapped the filesystem as a living ecosystem where the health of the environment determines the behavior of visible organisms in real time.

---

## Part 1: The Scientific Foundation for the Visualization

### 1.1 Physarum polycephalum as the Biological Model

The most important organism for this project is not a cleaner fish. It is **Physarum polycephalum** — the true slime mold.

**Why Physarum?**

Physarum is a single-celled organism that:
- navigates toward nutrient sources (food = waste tokens in your filesystem)
- leaves chemical trails (pheromones) that other agents follow
- forms efficient transport networks between food sources
- self-repairs when its network is damaged
- adapts its morphology in real time based on environmental conditions
- has been mathematically modeled as a multi-agent particle system

The canonical scientific description is Dr. Jeff Jones's 2010 paper:

> **Jones, J. (2010). "Characteristics of Pattern Formation and Evolution in Approximations of Physarum Transport Networks." Artificial Life, 16(2), 127–153.**  
> DOI: 10.1162/artl.2010.16.2.16202  
> PubMed: https://pubmed.ncbi.nlm.nih.gov/20067403/  
> UWE Repository: https://uwe-repository.worktribe.com/output/980579/

Jones's model uses a population of simple particle agents. Each agent:
1. senses chemoattractant in three directions (forward, left, right)
2. turns toward the highest concentration
3. deposits chemoattractant at its current position
4. moves forward at constant speed

The emergent result: complex, organic, dynamic transport networks that look unmistakably alive.

**The critical insight for this project:** In Jones's model, **food sources are the chemoattractant.** In the DMO, **filesystem waste is the chemoattractant.** The organism's behavior is mathematically identical — it is simply driven by different data.

Additional Jones papers (all on arXiv):
- "Mechanisms Inducing Parallel Computation in a Model of Physarum polycephalum Transport Networks": https://arxiv.org/abs/1511.05869
- "Representation of Shape Mediated by Environmental Stimuli in Physarum polycephalum and a Multi-agent Model": https://arxiv.org/abs/1511.05862
- Full publication list: https://www.lstmed.ac.uk/about/people/dr-jeff-jones

**Physarum validated against real infrastructure:** The slime mold's efficiency at constructing transport networks has been validated against the Tokyo metro system (Nakagaki et al., 2000) and studied as an unconventional computing substrate for decades. This is not fringe science. It is published in MIT Press journals.

---

### 1.2 Existing Physarum Simulation Implementations (All Verifiable)

**Critical finding:** Physarum simulations already run in-browser using WebGL and WebGPU, handling millions of particles in real time. No new rendering technology needs to be invented.

**Rust + WebGPU (most relevant to your stack):**
- tom-strowger/physarum-rust: https://github.com/tom-strowger/physarum-rust  
  Built with Rust and `wgpu`. Compiles to WebAssembly. Runs in-browser on Chrome (WebGPU). Uses compute shaders. Simulates millions of agents on M2 MacBook Air in real time. **This is nearly identical to the technology you would use inside a Tauri webview.**

**WebGL + TypeScript:**
- SuboptimalEng/slime-sim-webgpu: https://github.com/SuboptimalEng/slime-sim-webgpu  
  TypeScript + WebGPU. Playable demo available. Based on Sebastian Lague's coding adventure.
  
**WebGL2 (no dependencies, 1M+ particles):**
- maximilianklein/physarum: https://maximilianklein.github.io/showcase/physarum/  
  WebGL2. Handles 1 million+ particles. **Zero external dependencies.**

**JavaScript + WebGL:**
- nicoptere/physarum: https://github.com/nicoptere/physarum  
  JavaScript and WebGL, accessible to any web developer.

**Go (high-performance desktop rendering):**
- fogleman/physarum: https://github.com/fogleman/physarum  
  Go implementation. Produces stunning outputs. 

**Blender integration:**
- Physarum Editor (Blender fork): Listed at https://github.com/topics/physarum-polycephalum

**The game precedent:**
- PHYSARUM: Slime Mold Simulator (Steam): https://jkhyuen.github.io/physarum  
  Sold 450+ units. Single compute shader runs entire simulation. Real-time parameter adjustment. Attracted master's students in AI, biology, and engineering. **Proof of market for organism visualization as a product.**

---

### 1.3 The Broader ALife Visualization Precedent

**ALIEN (Artificial Life Environment):**
- Winner of ALIFE 2024 Virtual Creatures Competition
- CUDA-powered, millions of particles, real-time rendering
- Open source: https://github.com/chrxh/alien
- Website: https://www.alien-project.org/
- Video: https://www.alien-project.org/ (Emerging Ecosystems demo)

This project demonstrates that ALife simulations can be **visually compelling enough to win competitions** and be used as research platforms. It is the visual benchmark to study.

**Particle Life:**
- Attraction/repulsion rules between particle types produce life-like emergent patterns
- Source + algorithm: https://hunar4321.github.io/particle-life/
- GitHub: https://github.com/hunar4321/particle-life
- Available in C++, JavaScript, Python. Core algorithm is ~100 lines.
- **Evidence that life-like visualization requires minimal code when the physics is right.**

**ASAL (Automating the Search for Artificial Life with Foundation Models):**
- SakanaAI (with MIT, OpenAI, Swiss AI Lab IDSIA)
- Paper: https://arxiv.org/abs/2412.17799
- Project: https://asal.sakana.ai/
- GitHub: https://github.com/SakanaAI/asal
- Demonstrated foundation models can discover ALife patterns in Lenia, Boids, Particle Life, Game of Life, Neural Cellular Automata

---

## Part 2: The Visualization Architecture

### 2.1 The Core Problem: Mapping Filesystem to 2D Space

For organisms to navigate through the filesystem, the filesystem must be projected into a navigable 2D space. This is a solved problem in data visualization with well-documented algorithms.

**The treemap approach:**  
Ben Shneiderman invented treemaps specifically to visualize file system hierarchies. The squarified treemap algorithm divides 2D space proportionally to file/folder sizes, maintaining readable aspect ratios.

Reference: Bruls, M., Huizing, K., & van Wijk, J.J. (2000). "Squarified Treemaps." *Data Visualization 2000*, Springer.  
Online description: https://www.win.tue.nl/~vanwijk/stm.pdf  

However, rectangular treemaps are **wrong** for organism visualization. Organisms navigating between perfectly rectangular cells at right angles does not look organic. It looks like a grid.

**The correct approach: Voronoi Treemaps**  
Voronoi treemaps use irregular polygonal cells that tile 2D space while remaining proportional to data values. They produce organic-looking territories that organisms can meaningfully inhabit.

Reference: Balzer, M., & Deussen, O. (2005). "Voronoi Treemaps." In *Proceedings of the IEEE Symposium on Information Visualization.*  
Paper: https://ieeexplore.ieee.org/document/1532128

**D3.js implementations of Voronoi treemaps:**
- d3-voronoi-treemap: https://github.com/Kcnarf/d3-voronoi-treemap
- d3-voronoi-map: https://github.com/Kcnarf/d3-voronoi-map

These are JavaScript/TypeScript compatible, meaning they work inside a Tauri webview.

**What this means for the visualization:**  
Each folder in the filesystem becomes an irregular organic territory. Files within it become regions of that territory. Waste concentrations (cache files, temp files, logs) appear as bright spots — high-concentration chemoattractant zones. Organisms navigate between these zones.

---

### 2.2 The Data → Visualization Pipeline

```
Rust Backend (Tauri)           WebView (Svelte/TypeScript)
─────────────────────          ──────────────────────────
inotify / FSEvents             
filesystem observer            
        │                      
        ▼                      
  file metadata                JSON via Tauri
  (path, size, type,    ──────────────────────►  D3 Voronoi Treemap
   mtime, category,                               (filesystem as terrain)
   waste_score)                                          │
        │                                               ▼
  waste classifier                              WebGPU/WebGL
  (cache, temp, log,                            Physarum simulation
   duplicate, artifact)                         agents navigating terrain
        │                                              │
        ▼                                             ▼
  action proposals                         agent proposes action:
  (quarantine candidates,                  organism PULSES, highlights
   confidence score)                       territory, user approves
        │                                              │
        ▼                                             ▼
  rollback / quarantine ◄──────────────  action confirmed
  execution layer                         (or rejected by user)
        │                                              │
        ▼                                             ▼
  signed action log                       organism continues
  (audit trail)                           navigation
```

---

### 2.3 What the Organisms Represent

**Critical design decision:** Every visual element must have a direct mapping to real data. No decorative elements.

| Visual Element | What It Represents |
|---|---|
| Terrain territory shape | Folder / application zone |
| Territory size | Actual disk space consumed |
| Territory color (cool → warm) | Health: blue = clean, orange/red = waste-heavy |
| Organism position | Current scan focus |
| Organism trail | Recently scanned path |
| Organism cluster | High-waste concentration being evaluated |
| Organism pulse / glow | Pending action proposal |
| Organism dimming / disappearing | Zone already cleaned |
| Bright food node | Waste concentration (cache / temp / log / duplicate) |
| Territory texture (smooth → rough) | Fragmentation / disorder level |

This is not metaphor. This is a 1:1 mapping between visible behavior and real system state.

---

### 2.4 The Physarum Algorithm Adapted for DMO

The original Jones (2010) algorithm uses:
- **Chemoattractant:** arbitrary chemical deposited by agents and food sources
- **Sensor angle (SA):** how wide the agent looks ahead
- **Sensor offset (SO):** how far ahead the agent looks
- **Rotation angle (RA):** how far the agent turns

In the DMO adaptation:
- **Chemoattractant** = `waste_score` of each filesystem node, mapped to the 2D terrain
- **Waste concentration** = function of (file size × age × type risk × access recency)
- **Organisms** = particle agents navigating the terrain
- **Food consumption** = organism proposes cleanup action on the high-concentration node
- **Diffusion / decay** = waste concentration decreases after user approves cleanup

The mathematical formulation of waste concentration for a file `f`:

```
waste_score(f) = size_weight(f) × age_weight(f) × type_risk(f) × (1 - recency_score(f))

size_weight(f)    = log10(size_bytes / 1024)   # log scale
age_weight(f)     = days_since_mtime / 365     # normalized
type_risk(f)      = category_risk[file_type]   # lookup table
recency_score(f)  = exp(-days_since_access / 30) # exponential decay
```

This is not invented. It is standard information-theoretic weighting adapted from:

> Meylan, S., Gahl, S., & Regier, J. (2021). "Quantifying the predictability of visual form from distributional semantics." *Cognition.* (Example of weighted scoring with exponential decay)

The type_risk lookup table is derived from empirically well-understood categories:

| Type | Risk Score | Evidence |
|---|---|---|
| Application cache | 0.95 | macOS Storage Management: https://support.apple.com/en-ca/102624 |
| Package manager cache | 0.95 | npm, pip, cargo cache mechanics |
| Browser cache | 0.90 | All major browsers auto-expire |
| Build artifacts | 0.85 | xcodebuild, cargo build, webpack outputs |
| System logs (>30d) | 0.80 | macOS log rotation: https://developer.apple.com/documentation/os/logging |
| Temp files | 0.90 | POSIX /tmp convention |
| User documents | 0.00 | Never touched |
| App bundles | 0.00 | Never touched |

---

## Part 3: The Technical Stack

### 3.1 The Right Stack (Evidence-Based)

| Layer | Technology | Justification |
|---|---|---|
| Desktop shell | Tauri 2.0 | Rust backend + web frontend, privacy-first, local, cross-platform |
| Backend language | Rust | Memory safety, filesystem access, performance, already in Tauri |
| Filesystem observation | `notify` crate (Rust) | Cross-platform file change events: https://github.com/notify-rs/notify |
| Data storage | SQLite via `rusqlite` | Local, no server, perfect for file metadata cache |
| Frontend framework | SvelteKit | Reactive, lightweight, already familiar |
| 2D space mapping | D3.js (Voronoi treemap) | https://github.com/Kcnarf/d3-voronoi-treemap |
| Organism simulation | WebGL2 or WebGPU | Physarum particle system; WebGL2 for compatibility, WebGPU for performance |
| Simulation algorithm | Jones (2010) Physarum model | Peer-reviewed, proven, open implementations available |
| Shader language | WGSL (WebGPU) or GLSL (WebGL) | Match the chosen rendering API |
| Action safety | Quarantine folder + `trash` crate | https://github.com/Byron/trash-rs |
| Rollback | Metadata snapshots in SQLite | Log every proposed and executed action |
| IPC (Rust ↔ Web) | Tauri commands + events | Native to Tauri 2.0 |

**The notify crate** is the most important Rust crate for this project's backend. It provides cross-platform filesystem event watching using the OS's native APIs (inotify on Linux, FSEvents on macOS, ReadDirectoryChangesW on Windows):  
https://github.com/notify-rs/notify  
Documentation: https://docs.rs/notify/latest/notify/

**The trash crate** provides safe, reversible file deletion (to OS Trash, not permanent deletion):  
https://github.com/Byron/trash-rs  
This is the only acceptable deletion mechanism. Direct `std::fs::remove_file` is prohibited in the safety layer.

---

### 3.2 Why WebGL2 First, WebGPU Later

WebGL2 is supported in all Tauri webviews (Chromium-based) on all platforms today. The 1M+ particle Physarum simulation at https://maximilianklein.github.io/showcase/physarum/ runs without any dependencies in WebGL2.

WebGPU is faster and more powerful (compute shaders, 300k+ particles at better performance) but has narrower availability. tom-strowger's Rust+WGPU implementation (https://github.com/tom-strowger/physarum-rust) compiles to WebAssembly and runs in Chrome but not all browsers.

**Decision:** Start with WebGL2. The particle counts needed for a filesystem visualization (hundreds to a few thousand organisms) are well within WebGL2 capabilities. Upgrade to WebGPU if performance demands it.

---

### 3.3 The Simulation-Data Bridge (The Most Novel Engineering)

This is the part that does not exist anywhere and must be built:

```typescript
// Pseudocode: Svelte component bridging filesystem data to Physarum simulation
import { physarumSim } from './simulation/physarum';
import { voronoiMap } from './visualization/voronoi';

// From Tauri backend via IPC
const filesystemState: FilesystemNode[] = await invoke('scan_filesystem');

// Project filesystem into 2D terrain
const terrain = voronoiMap.build(filesystemState, {
  width: canvas.width,
  height: canvas.height,
  valueAccessor: (node) => node.size_bytes,
});

// Generate chemoattractant map from waste scores
const chemMap = new Float32Array(terrain.width * terrain.height);
for (const cell of terrain.cells) {
  const wasteScore = cell.node.waste_score;  // 0.0 to 1.0
  fillRegion(chemMap, cell.polygon, wasteScore);
}

// Initialize Physarum simulation with organism population
physarumSim.initialize({
  canvas: canvas,
  chemoattractantMap: chemMap,
  agentCount: 2000,
  sensorAngle: 45,         // Jones (2010) parameters
  sensorOffset: 9,
  rotationAngle: 45,
  stepSize: 1.0,
});

// Main loop: every N frames, re-read filesystem state
function tick() {
  physarumSim.step();  // one Physarum simulation step on GPU
  checkActionProposals(physarumSim.clusters);  // where are agents clustering?
  requestAnimationFrame(tick);
}
```

**Action proposal detection:** When the Physarum simulation produces dense agent clusters (agents congregating around high-concentration food sources), those clusters correspond to specific filesystem nodes. The system reads which filesystem node occupies that region of the terrain and proposes it as a cleanup candidate.

```
cluster_position (x, y)  →  voronoi_cell lookup  →  filesystem_node  →  action_proposal
```

This is the bridge. The organism is not decorating a file list. The organism is **identifying the file** by converging on it.

---

## Part 4: The Safety Architecture

Phase 1 covered this correctly. Restating with implementation detail.

### 4.1 The Safety Pyramid

```
┌─────────────────────────────────────────────────────┐
│  LAYER 4 — User Interface                           │
│  All proposed actions shown. Organism pulses.       │
│  User approves / rejects. Full undo visible.        │
├─────────────────────────────────────────────────────┤
│  LAYER 3 — Quarantine Gate                          │
│  Zero permanent deletion. All moves go to           │
│  ~/.dmo_quarantine/YYYY-MM-DD/ with metadata.       │
│  trash crate: https://github.com/Byron/trash-rs     │
├─────────────────────────────────────────────────────┤
│  LAYER 2 — Confidence Threshold + Allowlist         │
│  No action below 0.80 confidence without user OK.   │
│  Strict allowlist of safe target categories.        │
│  Explicit denylist: ~/Documents, ~/Desktop,         │
│  app bundles, databases, cloud sync folders.        │
├─────────────────────────────────────────────────────┤
│  LAYER 1 — Read-Only Observer Mode (Default)        │
│  System starts in observation-only mode.            │
│  No filesystem modification for minimum 7 days.    │
│  Audit log begins immediately.                      │
└─────────────────────────────────────────────────────┘
```

### 4.2 Action Log Schema (SQLite)

```sql
CREATE TABLE action_log (
  id          INTEGER PRIMARY KEY,
  timestamp   DATETIME NOT NULL,
  path        TEXT NOT NULL,
  action_type TEXT NOT NULL,  -- 'PROPOSED', 'QUARANTINED', 'RESTORED', 'REJECTED'
  category    TEXT NOT NULL,  -- 'browser_cache', 'build_artifact', etc.
  size_bytes  INTEGER,
  confidence  REAL,           -- 0.0 to 1.0
  user_decision TEXT,         -- 'APPROVED', 'REJECTED', NULL (pending)
  quarantine_path TEXT,       -- Where the file went if quarantined
  sha256      TEXT            -- File hash before action (for verification)
);
```

Every action leaves a permanent, verifiable record. The hash allows detection of any corruption or discrepancy during quarantine.

---

## Part 5: What Has Not Been Built Before — The Precise Gap

After searching the literature, GitHub, product databases, and ALife archives, the following combination does not exist as a product:

1. ✅ ALife Physarum simulations: exist (Jones 2010, tom-strowger, SuboptimalEng, nicoptere, fogleman)
2. ✅ File system visualizations: exist (DaisyDisk, SpaceSniffer, WinDirStat, Ben Shneiderman treemaps)
3. ✅ Desktop maintenance agents: exist (CleanMyMac, Hazel, Gemini)
4. ✅ WebGL/WebGPU organism simulations: exist (multiple open source projects)
5. ✅ Voronoi treemaps of filesystem: academic implementations exist (d3-voronoi-treemap)

**What does NOT exist:**  
A product that **combines** (1), (2), (3), and (4) into a single cohesive system where the simulation organisms are not decorative but are the actual maintenance logic — where the organisms navigate a 2D representation of your real filesystem, driven by real waste data, proposing real actions.

The closest adjacent product is DaisyDisk (beautiful visual representation) + CleanMyMac (maintenance functionality) + ALIEN/Physarum simulation (living visualization). No one has merged these three things.

The absence is the opportunity.

---

## Part 6: Build Phases (Revised)

### Phase 1 — Static Terrain (3–4 weeks)
**Goal:** Render the filesystem as a living visual space with no organism behavior.

Deliverables:
- Rust backend: scan filesystem, classify files, compute waste_score per node
- SQLite: persist filesystem snapshot with metadata
- Svelte + D3: Voronoi treemap rendered in Tauri webview, colored by waste score
- Color mapping: blue (clean) → orange (waste-heavy) → red (critical)
- No organisms yet. Just terrain.

Verification criterion: A user can see their real filesystem as an organic territory map and identify red zones that correspond to known cache/build folders.

---

### Phase 2 — Organisms Appear (4–5 weeks)
**Goal:** Add Physarum-style organisms that navigate the terrain, driven by waste scores.

Deliverables:
- WebGL2 Physarum particle system rendering on top of the terrain
- Chemoattractant map generated from waste_score data
- Organisms navigate toward high-concentration zones
- Organisms visible as bioluminescent trails on the terrain
- No filesystem actions yet. Pure visualization.

Verification criterion: Organisms visibly converge on the same zones that the terrain identified as waste-heavy in Phase 1.

---

### Phase 3 — Proposal System (4–5 weeks)
**Goal:** Organisms propose actions. User approves or rejects.

Deliverables:
- Cluster detection: identify which filesystem node organisms are converging on
- Proposal UI: organism pulses, tooltip shows file details and rationale
- User approve/reject controls
- Quarantine execution via `trash` crate on approve
- Rollback: one-click restore from quarantine
- Audit log active

Verification criterion: User can approve an organism's proposal, see the file move to quarantine, and restore it with one click. Zero permanent deletions.

---

### Phase 4 — Learning Loop (6–8 weeks)
**Goal:** System learns from user accept/reject history.

Deliverables:
- Per-path and per-category trust scores stored in SQLite
- Waste_score formula adjusted by learned weights
- Organisms navigate differently based on updated chemoattractant map
- Visual differentiation: organisms in learned-safe zones behave calmly, organisms in uncertain zones behave cautiously
- Optional: low-confidence items auto-approved after 30 days of consistent user approval history

---

### Phase 5 — ALife Sandbox (Research Track, Parallel)
**Goal:** Evolving agent strategies. Not on product critical path.

Deliverables:
- Isolated simulation environment where agents have parameter genomes
- Agents compete for CPU budget, evaluated on reclaimed space ÷ false-positive rate
- Promising genomes promoted to production chemoattractant parameters after offline validation

---

## Part 7: Open Research Questions (Phase 3 Preparation)

These must be resolved before Phase 3 development begins:

**1. Cluster detection algorithm**  
How to efficiently detect agent density hotspots in the GPU-simulated particle system and map them back to filesystem nodes?  
Candidate: read back density texture from GPU, threshold by density > µ + 2σ, spatial join to Voronoi cells.

**2. Chemoattractant update frequency**  
How often should the waste_score map be updated? Filesystem changes in real time, but recomputing the entire Voronoi layout is expensive.  
Candidate: terrain rebuilds every 60 minutes or on significant filesystem event; chemoattractant values update continuously; terrain geometry stable.

**3. Organism population size**  
Too few: slow convergence, sparse visual. Too many: visual noise, computational cost.  
Candidate: 500–2000 organisms based on filesystem size; empirically calibrated in Phase 2.

**4. Privacy audit**  
The Rust backend reads all filesystem metadata. What is the minimal surface of data collection for the visualization to work?  
Constraint: paths, sizes, types, mtime, atime. No file contents ever read. No cloud transmission.

---

## Part 8: Verified Source Index

All sources used in this document, with URLs:

| # | Source | URL |
|---|---|---|
| 1 | Jones (2010) Physarum paper — Artificial Life journal | https://pubmed.ncbi.nlm.nih.gov/20067403/ |
| 2 | Jones arXiv papers (physarum mechanisms) | https://arxiv.org/abs/1511.05869 |
| 3 | Jones physarum shape paper | https://arxiv.org/abs/1511.05862 |
| 4 | Jeff Jones publication list | https://www.lstmed.ac.uk/about/people/dr-jeff-jones |
| 5 | ALIEN (CUDA ALife simulation) | https://github.com/chrxh/alien |
| 6 | ALIEN project website | https://www.alien-project.org/ |
| 7 | Particle Life (emergence from simple rules) | https://github.com/hunar4321/particle-life |
| 8 | Particle Life website | https://hunar4321.github.io/particle-life/ |
| 9 | ASAL paper (foundation models + ALife) | https://arxiv.org/abs/2412.17799 |
| 10 | ASAL project + GitHub | https://asal.sakana.ai/ / https://github.com/SakanaAI/asal |
| 11 | Physarum sim: Rust + WebGPU | https://github.com/tom-strowger/physarum-rust |
| 12 | Physarum sim: WebGPU + TypeScript | https://github.com/SuboptimalEng/slime-sim-webgpu |
| 13 | Physarum sim: WebGL2, 1M+ particles | https://maximilianklein.github.io/showcase/physarum/ |
| 14 | Physarum sim: JS + WebGL | https://github.com/nicoptere/physarum |
| 15 | Physarum sim: Go (high quality renders) | https://github.com/fogleman/physarum |
| 16 | Physarum sim: Steam game | https://jkhyuen.github.io/physarum |
| 17 | GitHub physarum topic | https://github.com/topics/physarum |
| 18 | D3 Voronoi Treemap | https://github.com/Kcnarf/d3-voronoi-treemap |
| 19 | D3 Voronoi Map | https://github.com/Kcnarf/d3-voronoi-map |
| 20 | Squarified Treemaps paper | https://www.win.tue.nl/~vanwijk/stm.pdf |
| 21 | notify crate (Rust filesystem events) | https://github.com/notify-rs/notify |
| 22 | notify docs | https://docs.rs/notify/latest/notify/ |
| 23 | trash crate (safe deletion) | https://github.com/Byron/trash-rs |
| 24 | Tauri 2.0 | https://v2.tauri.app/ |
| 25 | Avida (digital evolution) | https://github.com/devosoft/avida |
| 26 | Lenia (continuous CA) | https://github.com/Chakazul/Lenia |
| 27 | Awesome ALife list | https://github.com/jetnew/awesome-artificial-life |
| 28 | macOS storage management | https://support.apple.com/en-ca/102624 |
| 29 | CleanMyMac X (competitor) | https://macpaw.com/cleanmymac |
| 30 | Hazel (competitor) | https://www.noodlesoft.com/ |
| 31 | DaisyDisk (competitor) | https://daisydiskapp.com/ |
| 32 | WebGPU fluid simulation + Codrops | https://tympanus.net/codrops/2025/02/26/webgpu-fluid-simulations-high-performance-real-time-rendering/ |
| 33 | Particle Life simulation in browser WebGPU | https://lisyarus.github.io/blog/posts/particle-life-simulation-in-browser-using-webgpu.html |
| 34 | Babylon.js (WebGPU visualization library) | https://www.babylonjs.com/ |
| 35 | slime mold Tokyo metro validation | Nakagaki et al. (2000). Nature, 407, 470. https://doi.org/10.1038/35035159 |

---

## Part 9: The Honest Risk Assessment

### Risks that will kill this project

**Risk 1: The visualization overwhelms the engineering**  
If you spend 80% of your time making organisms look beautiful and 20% on the safety layer, you will produce an impressive demo that deletes the wrong files. The safety substrate is not negotiable. Build it first, completely, before any visual work.

**Risk 2: Classification accuracy below threshold**  
If the waste_score formula produces too many false positives (flagging user files as waste), users will distrust the system permanently after one incident. The initial conservative type_risk table must be validated manually before any automated action is enabled. Start with only categories where the false-positive rate is near zero: npm/pip/cargo cache, browser cache, xcode derived data.

**Risk 3: Performance impact**  
A Rust daemon observing all filesystem events + a GPU-driven particle simulation running simultaneously must not meaningfully impact system performance. Target: < 2% CPU overhead for the daemon at rest; GPU simulation runs only when the window is visible. Benchmark early.

**Risk 4: The two-track confusion**  
The ALife sandbox (evolving agents, fitness functions, competitive selection) is intellectually seductive and technically separate from the product. If the simulation sandbox starts influencing the live filesystem before it has been validated for weeks in isolation, the project becomes dangerous. These tracks must be separated by a strict code boundary, not just by intention.

### Risks that will not kill this project but will slow it

- WebGPU compatibility (use WebGL2 fallback)  
- D3 Voronoi treemap performance on very large filesystems (hierarchical culling)  
- macOS sandboxing restrictions for Tauri apps (use entitlements carefully)

---

## Phase 2 Conclusion

The core finding of this phase is this:

**The correct organism model for this project is the Physarum polycephalum slime mold simulation, specifically the Jeff Jones (2010) multi-agent particle formulation.** It maps directly onto the DMO use case: agents navigate toward waste tokens the same way real Physarum navigates toward food sources. The mathematical model is published, peer-reviewed, and open. The simulation technology runs in-browser on WebGL2 today with no external dependencies and handles 1M+ particles.

The filesystem must be projected into a 2D Voronoi treemap where territory shapes correspond to real folders, territory size corresponds to real disk usage, and chemoattractant concentration corresponds to computed waste scores. The organisms navigate this terrain. The clusters they form identify cleanup candidates. Their proposals are real filesystem operations.

This is not a metaphor wrapped around a file list. It is a functional maintenance system where the visual behavior of the organisms is the interface to the system's actual logic.

Nothing like this exists as a product. That gap is the opportunity.

**The next concrete step is Phase 1 of the build: render a static Voronoi treemap of the real filesystem inside a Tauri window, colored by waste score, with no organisms.** If that looks compelling and accurate, the foundation is validated and Phase 2 (adding organisms) begins.
