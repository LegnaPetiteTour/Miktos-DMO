# DMO Phase 2 — Implementation Plan

**Status:** 🔜 Ready to start  
**Baseline commit:** `7d0a323`  
**Research foundation:** [`DMO_Phase2_Research_Architecture.md`](DMO_Phase2_Research_Architecture.md)

---

## Goal

Add a WebGL2 Physarum polycephalum particle simulation rendered on top of the Voronoi terrain.

**Acceptance criterion:** Organisms visibly converge on the same zones that the terrain identifies as waste-heavy. The simulation must run at 60 fps on M-series hardware with 1,000–2,000 agents. No filesystem actions in this phase — pure visualization.

---

## What Is Already in Place (from Phase 1)

Before writing a single line of Phase 2 code, the following infrastructure is ready:

| Hook | Location | State |
|---|---|---|
| `overlayCanvas` | `dmo-app/src/App.svelte` | Bound `<canvas>` element, `pointer-events: none`, `display: none` |
| `terrainCells: TerrainCell[]` | `dmo-app/src/App.svelte` | Updated after every Voronoi layout via `onCells` |
| `TerrainCell` type | `dmo-app/src/lib/types.ts` | `{ polygon, path, waste_score, centroid, area }` |
| `onCells` prop | `dmo-app/src/lib/Treemap.svelte` | Fires after each layout with full terrain layout |
| `scan_id` | `ScanResult` (Rust + TS) | Row ID in `~/.dmo/history.db` for top-waste queries |

**The 2D canvas and WebGL2 canvas are separate elements.** There is no context conflict. The terrain renders with `canvas.getContext("2d")`; the simulation will render with `overlayCanvas.getContext("webgl2")`.

---

## Deliverables

### D1 — Chemoattractant Texture
Build a `Float32Array` of dimensions `width × height` from `terrainCells`.

For each terrain cell:
1. Rasterize the polygon into the texture (point-in-polygon or scanline fill)
2. Write `cell.waste_score` as the chemoattractant concentration at every pixel inside

Result: a grayscale float texture where bright areas = waste-heavy zones = organism food sources.

**Implementation file:** `dmo-app/src/lib/sim/chemoattractant.ts`

```typescript
export function buildChemoattractantTexture(
  cells: TerrainCell[],
  width: number,
  height: number
): Float32Array
```

### D2 — WebGL2 Physarum Simulation

A self-contained WebGL2 simulation class using two ping-pong framebuffers (agent state and trail map) and four shader programs.

**Implementation files:**
- `dmo-app/src/lib/sim/PhysarumSim.ts` — simulation class
- `dmo-app/src/lib/sim/shaders/agent.vert.glsl` — agent vertex shader
- `dmo-app/src/lib/sim/shaders/agent.frag.glsl` — agent update fragment shader
- `dmo-app/src/lib/sim/shaders/diffuse.frag.glsl` — trail map diffuse + decay
- `dmo-app/src/lib/sim/shaders/render.frag.glsl` — render trail map to screen

#### Agent State Encoding
Each agent is a pixel in a `agentCount × 1` RGBA float texture:
- `R` = x position (normalized [0, 1])
- `G` = y position (normalized [0, 1])
- `B` = heading angle (radians)
- `A` = unused (reserved for Phase 3 agent ID)

#### Jones (2010) Parameters (starting values, tunable)
| Parameter | Symbol | Value |
|---|---|---|
| Sensor angle | SA | 45° |
| Sensor offset | SO | 9 px |
| Rotation angle | RA | 45° |
| Step size | — | 1.0 px/frame |
| Trail deposit | — | 1.0 |
| Diffuse rate | — | 0.9 (trail × 0.9 each frame) |
| Decay rate | — | 0.005 (subtract each frame after diffuse) |

#### Simulation Class API

```typescript
class PhysarumSim {
  constructor(gl: WebGL2RenderingContext, width: number, height: number)
  
  initialize(options: {
    agentCount: number;
    chemoMap: Float32Array;        // waste_score rasterized to width × height
    sensorAngle?: number;          // radians
    sensorOffset?: number;         // pixels
    rotationAngle?: number;        // radians
    stepSize?: number;
    trailDecay?: number;
  }): void

  step(): void                     // advance simulation one tick
  
  /** Upload a new chemoattractant map (call after re-scan) */
  updateChemoMap(chemoMap: Float32Array): void

  dispose(): void
}
```

### D3 — Simulation Integration in App.svelte

**Logic to add to `App.svelte`:**

```typescript
let sim: PhysarumSim | null = $state(null);
let simRunning: boolean = $state(false);

function startSim() {
  if (!overlayCanvas || terrainCells.length === 0) return;
  const gl = overlayCanvas.getContext('webgl2');
  if (!gl) { console.warn('WebGL2 not available'); return; }
  
  overlayCanvas.width = overlayCanvas.offsetWidth;
  overlayCanvas.height = overlayCanvas.offsetHeight;
  overlayCanvas.style.display = 'block';
  
  const chemoMap = buildChemoattractantTexture(terrainCells, overlayCanvas.width, overlayCanvas.height);
  
  sim = new PhysarumSim(gl, overlayCanvas.width, overlayCanvas.height);
  sim.initialize({ agentCount: 1500, chemoMap });
  simRunning = true;
  
  requestAnimationFrame(simLoop);
}

function simLoop() {
  if (!simRunning || !sim) return;
  sim.step();
  requestAnimationFrame(simLoop);
}

function stopSim() {
  simRunning = false;
  sim?.dispose();
  sim = null;
  if (overlayCanvas) overlayCanvas.style.display = 'none';
}
```

**After `startScan()` completes:** if `simRunning`, call `sim.updateChemoMap(...)` with the new terrain.

**UI toggle:** Add a button to the header — `🔬 Organism` or `⚛ Sim` — that starts/stops the simulation.

### D4 — Simulation Controls (optional, do after D1–D3 working)

A collapsible parameter panel allowing real-time tweaking:
- Agent count (500 / 1000 / 1500 / 2000)
- Sensor angle (15°–90°)
- Sensor offset (4–18 px)
- Trail decay rate (0.90–0.995)
- Chemoattractant influence weight (0.5–3.0×)

---

## Implementation Sequence

```
Step 1 — Chemoattractant texture builder (pure TypeScript, no WebGL)
         Test: visualize texture as a grayscale image on a temp canvas

Step 2 — WebGL2 setup + ping-pong framebuffers
         Test: render a static texture to the overlay canvas

Step 3 — Agent initialization shader (scatter agents randomly)
         Test: see N points rendered on overlay canvas

Step 4 — Agent update shader (sense → rotate → move)
         Test: agents move; no chemoattractant yet (random walk)

Step 5 — Wire chemoattractant texture into sensor reads
         Test: agents cluster toward red zones on the terrain

Step 6 — Trail map + diffuse + decay shaders
         Test: bioluminescent trails visible, fade correctly

Step 7 — Integration: start/stop in App.svelte, chemo map rebuild on rescan

Step 8 — (optional) Parameter controls panel
```

---

## File Structure for Phase 2

```
dmo-app/src/lib/
├── sim/
│   ├── PhysarumSim.ts              # Main simulation class
│   ├── chemoattractant.ts         # Texture builder from TerrainCell[]
│   └── shaders/
│       ├── agent.frag.glsl        # Agent: sense → rotate → move → deposit
│       ├── diffuse.frag.glsl      # Trail map: diffuse + decay
│       └── render.frag.glsl       # Fullscreen quad: render trail to screen
```

No new Rust backend work is required for Phase 2. All processing is on the GPU in the browser/webview.

---

## WebGL2 Compatibility

Tauri's webview on macOS (WKWebView / Chromium) supports WebGL2. Verified reference: `maximilianklein.github.io/showcase/physarum/` runs 1M+ particles in-browser using WebGL2 with zero dependencies.

If WebGL2 is unavailable (checked via `canvas.getContext('webgl2') === null`), the simulation silently does not start — the terrain visualization continues to work normally.

---

## Performance Targets

| Metric | Target |
|---|---|
| Agent count | 1,000–2,000 (empirically calibrated) |
| Framerate | 60 fps on M-series Mac |
| CPU overhead | < 1% (all work on GPU) |
| Memory | < 32 MB GPU memory for textures |
| Behaviour on rescan | Simulation pauses, chemo map rebuilt, simulation resumes |
| Behaviour on drill-down | Simulation pauses (terrain geometry changes), resumes on re-layout |

---

## What Phase 2 Explicitly Does Not Do

- No filesystem modifications of any kind
- No cluster detection or action proposals (Phase 3)
- No organism "memory" or persistence between sessions (Phase 4)
- No WGPU/WASM compute shaders (start with WebGL2; upgrade if needed)

---

## References

| Concept | Source |
|---|---|
| Jones (2010) Physarum algorithm | https://pubmed.ncbi.nlm.nih.gov/20067403/ |
| WebGL2 Physarum (zero deps, 1M+ agents) | https://maximilianklein.github.io/showcase/physarum/ |
| Physarum WebGPU + TypeScript | https://github.com/SuboptimalEng/slime-sim-webgpu |
| Physarum Rust + WebGPU | https://github.com/tom-strowger/physarum-rust |
| Full reference list (52 sources) | [`DMO_Phase2_Research_Architecture.md`](DMO_Phase2_Research_Architecture.md) |
