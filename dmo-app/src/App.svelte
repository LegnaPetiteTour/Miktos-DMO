<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Treemap from "./lib/Treemap.svelte";
  import type { ScanResult, TreeNode } from "./lib/types";

  // ─── State ───
  let scanResult: ScanResult | null = $state(null);
  let scanning: boolean = $state(false);
  let error: string | null = $state(null);
  let scanPath: string = $state("");
  let maxDepth: number = $state(5);

  // Tooltip state
  let hoveredNode: TreeNode | null = $state(null);
  let tooltipX: number = $state(0);
  let tooltipY: number = $state(0);

  // Drill-down navigation
  let viewStack: TreeNode[] = $state([]);
  let currentView: TreeNode | null = $derived(
    viewStack.length > 0 ? viewStack[viewStack.length - 1] : scanResult?.tree ?? null
  );

  // Category panel
  let showCategories: boolean = $state(false);

  // ─── Initialize ───
  async function init() {
    try {
      const home: string = await invoke("get_home_dir");
      scanPath = `${home}/Library/Caches`;
    } catch {
      scanPath = "/tmp";
    }
  }
  init();

  // ─── Scan ───
  async function startScan() {
    if (scanning || !scanPath) return;
    scanning = true;
    error = null;
    scanResult = null;
    viewStack = [];

    try {
      const result: ScanResult = await invoke("scan_filesystem", {
        path: scanPath,
        maxDepth: maxDepth,
      });
      scanResult = result;
    } catch (e: any) {
      error = typeof e === "string" ? e : e.message || "Scan failed";
    } finally {
      scanning = false;
    }
  }

  // ─── Navigation ───
  function handleCellClick(node: TreeNode) {
    if (node.is_directory && node.children && node.children.length > 0) {
      viewStack = [...viewStack, node];
    }
  }

  function navigateBack() {
    if (viewStack.length > 0) {
      viewStack = viewStack.slice(0, -1);
    }
  }

  function navigateToLevel(index: number) {
    if (index < 0) {
      viewStack = [];
    } else {
      viewStack = viewStack.slice(0, index + 1);
    }
  }

  // ─── Tooltip ───
  function handleHover(node: TreeNode | null, x: number, y: number) {
    hoveredNode = node;
    tooltipX = x;
    tooltipY = y;
  }

  // ─── Keyboard ───
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !scanning) {
      startScan();
    }
    if (e.key === "Escape" || e.key === "Backspace") {
      if (viewStack.length > 0) {
        e.preventDefault();
        navigateBack();
      }
    }
  }

  // ─── Format helpers ───
  function formatBytes(bytes: number): string {
    if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
    if (bytes >= 1024 ** 2) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${bytes} B`;
  }

  function formatScore(score: number): string {
    if (score < 0.001) return "<0.001";
    return score.toFixed(4);
  }

  function wastePercent(): string {
    if (!scanResult) return "0";
    const pct = (scanResult.summary.waste_size / scanResult.summary.total_size) * 100;
    return pct.toFixed(1);
  }

  function scoreColor(score: number): string {
    if (score >= 0.3) return "#ef4444";
    if (score >= 0.15) return "#f97316";
    if (score >= 0.05) return "#eab308";
    return "#22c55e";
  }

  function abbreviatePath(path: string): string {
    const home = path.indexOf("/Users/");
    if (home >= 0) {
      const afterUsers = path.substring(home + 7);
      const slash = afterUsers.indexOf("/");
      if (slash >= 0) return "~" + afterUsers.substring(slash);
    }
    return path;
  }

  // Preset functions
  async function setPreset(preset: string) {
    try {
      const home: string = await invoke("get_home_dir");
      switch (preset) {
        case "caches": scanPath = `${home}/Library/Caches`; maxDepth = 4; break;
        case "library": scanPath = `${home}/Library`; maxDepth = 5; break;
        case "home": scanPath = home; maxDepth = 4; break;
      }
    } catch {}
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="app">
  <!-- ═══ HEADER ═══ -->
  <header class="header">
    <div class="header-left">
      <h1 class="title">
        <span class="title-accent">DMO</span>
        <span class="title-dim">Digital Maintenance Organism</span>
      </h1>
    </div>
    <div class="header-right">
      {#if scanResult}
        <div class="stats-bar">
          <span class="stat">
            <span class="stat-label">Files</span>
            <span class="stat-value">{scanResult.summary.total_files.toLocaleString()}</span>
          </span>
          <span class="stat">
            <span class="stat-label">Total</span>
            <span class="stat-value">{formatBytes(scanResult.summary.total_size)}</span>
          </span>
          <span class="stat waste">
            <span class="stat-label">Waste</span>
            <span class="stat-value">{formatBytes(scanResult.summary.waste_size)}</span>
          </span>
          <span class="stat waste-pct">
            <span class="stat-label">Waste %</span>
            <span class="stat-value">{wastePercent()}%</span>
          </span>
          <span class="stat">
            <span class="stat-label">Time</span>
            <span class="stat-value">{scanResult.summary.scan_time_ms.toLocaleString()}ms</span>
          </span>
        </div>
      {/if}
    </div>
  </header>

  <!-- ═══ CONTROLS ═══ -->
  <div class="controls">
    <div class="input-group">
      <input
        type="text"
        bind:value={scanPath}
        placeholder="Path to scan..."
        class="path-input"
        disabled={scanning}
      />
      <select bind:value={maxDepth} class="depth-select" disabled={scanning}>
        <option value={3}>Depth 3</option>
        <option value={4}>Depth 4</option>
        <option value={5}>Depth 5</option>
        <option value={6}>Depth 6</option>
        <option value={8}>Depth 8</option>
      </select>
      <button onclick={startScan} class="scan-btn" disabled={scanning}>
        {#if scanning}
          <span class="scan-spinner"></span> Scanning...
        {:else}
          Scan
        {/if}
      </button>
      {#if scanResult}
        <button
          class="cat-btn"
          class:active={showCategories}
          onclick={() => showCategories = !showCategories}
          title="Toggle category breakdown"
        >
          ☰
        </button>
      {/if}
    </div>
    <div class="controls-row-2">
      <div class="presets">
        <button class="preset-btn" onclick={() => setPreset("caches")}>~/Library/Caches</button>
        <button class="preset-btn" onclick={() => setPreset("library")}>~/Library</button>
        <button class="preset-btn" onclick={() => setPreset("home")}>Home</button>
      </div>
      <!-- Breadcrumb navigation -->
      {#if viewStack.length > 0}
        <nav class="breadcrumb">
          <button class="crumb" onclick={() => navigateToLevel(-1)}>
            {scanResult?.tree.name ?? "Root"}
          </button>
          {#each viewStack as crumb, i}
            <span class="crumb-sep">›</span>
            <button
              class="crumb"
              class:crumb-current={i === viewStack.length - 1}
              onclick={() => navigateToLevel(i)}
            >
              {crumb.name}
            </button>
          {/each}
          <button class="back-btn" onclick={navigateBack} title="Go back (Esc)">←</button>
        </nav>
      {/if}
    </div>
  </div>

  <!-- ═══ MAIN CONTENT ═══ -->
  <div class="content">
    {#if error}
      <div class="center-panel">
        <div class="error-icon">✕</div>
        <p class="error-text">{error}</p>
        <button class="retry-btn" onclick={startScan}>Retry</button>
      </div>
    {:else if scanning}
      <div class="center-panel">
        <div class="scan-anim">
          <div class="scan-ring"></div>
          <div class="scan-ring r2"></div>
          <div class="scan-dot"></div>
        </div>
        <p class="loading-text">Scanning filesystem...</p>
        <p class="loading-sub">Observing the terrain</p>
      </div>
    {:else if currentView}
      <div class="treemap-container">
        <Treemap tree={currentView} onHover={handleHover} onClick={handleCellClick} />
      </div>

      <!-- Category breakdown panel -->
      {#if showCategories && scanResult}
        <div class="categories-panel">
          <div class="cat-header">Category Breakdown</div>
          {#each scanResult.summary.categories.sort((a, b) => b.total_bytes - a.total_bytes) as cat}
            <div class="cat-row">
              <div class="cat-name">{cat.name}</div>
              <div class="cat-stats">
                <span class="cat-size">{formatBytes(cat.total_bytes)}</span>
                <span class="cat-count">{cat.file_count.toLocaleString()} files</span>
              </div>
              <div class="cat-bar-bg">
                <div
                  class="cat-bar-fill"
                  style="width: {Math.min(100, (cat.total_bytes / scanResult.summary.total_size) * 100)}%; background: {scoreColor(cat.avg_score)}"
                ></div>
              </div>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Floating tooltip -->
      {#if hoveredNode}
        <div
          class="tooltip"
          style="left: {Math.min(tooltipX + 16, (typeof window !== 'undefined' ? window.innerWidth - 300 : 800))}px; top: {Math.max(tooltipY - 10, 8)}px;"
        >
          <div class="tt-name">{hoveredNode.name}</div>
          <div class="tt-row">
            <span class="tt-label">Size</span>
            <span class="tt-value">{formatBytes(hoveredNode.size)}</span>
          </div>
          <div class="tt-row">
            <span class="tt-label">Category</span>
            <span class="tt-value tt-cat">{hoveredNode.category}</span>
          </div>
          <div class="tt-row">
            <span class="tt-label">Score</span>
            <span class="tt-value" style="color: {scoreColor(hoveredNode.waste_score)}; font-weight: 700;">
              {formatScore(hoveredNode.waste_score)}
            </span>
          </div>
          {#if hoveredNode.is_directory && hoveredNode.children?.length}
            <div class="tt-row">
              <span class="tt-label">Children</span>
              <span class="tt-value">{hoveredNode.children.length} zones</span>
            </div>
            <div class="tt-drill">Click to explore →</div>
          {/if}
          <div class="tt-path">{abbreviatePath(hoveredNode.path)}</div>
        </div>
      {/if}

    {:else}
      <div class="center-panel">
        <div class="empty-organism">
          <div class="org-ring"></div>
          <div class="org-core">◉</div>
        </div>
        <p class="empty-title">Select a path and scan to map the terrain</p>
        <p class="empty-sub">The organism will navigate toward waste concentrations</p>
        <p class="empty-hint">Press Enter to scan</p>
      </div>
    {/if}
  </div>
</main>

<style>
  /* ═══ LAYOUT ═══ */
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  /* ═══ HEADER ═══ */
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 20px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .title {
    font-size: 14px;
    font-weight: 400;
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .title-accent {
    font-weight: 800;
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: 16px;
    letter-spacing: 1px;
  }
  .title-dim {
    color: var(--text-muted);
    font-size: 11px;
  }
  .stats-bar {
    display: flex;
    gap: 18px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
  }
  .stat-label {
    font-size: 9px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.6px;
    font-weight: 600;
  }
  .stat-value {
    font-size: 13px;
    font-family: var(--font-mono);
    color: var(--text-primary);
  }
  .stat.waste .stat-value,
  .stat.waste-pct .stat-value {
    color: var(--danger);
  }

  /* ═══ CONTROLS ═══ */
  .controls {
    padding: 8px 20px 10px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .input-group {
    display: flex;
    gap: 6px;
  }
  .controls-row-2 {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 6px;
    min-height: 24px;
  }
  .path-input {
    flex: 1;
    padding: 7px 12px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 12px;
    outline: none;
    transition: border-color 0.15s;
  }
  .path-input:focus {
    border-color: var(--accent-dim);
  }
  .depth-select {
    padding: 7px 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 12px;
    outline: none;
    cursor: pointer;
  }
  .scan-btn {
    padding: 7px 22px;
    background: var(--accent-dim);
    border: none;
    border-radius: 6px;
    color: white;
    font-weight: 600;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .scan-btn:hover:not(:disabled) {
    background: var(--accent);
    transform: translateY(-1px);
  }
  .scan-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .scan-spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .cat-btn {
    padding: 7px 10px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-muted);
    font-size: 14px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .cat-btn:hover, .cat-btn.active {
    border-color: var(--accent-dim);
    color: var(--accent);
  }

  .presets {
    display: flex;
    gap: 5px;
  }
  .preset-btn {
    padding: 3px 9px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-muted);
    font-size: 10px;
    font-family: var(--font-mono);
    cursor: pointer;
    transition: all 0.15s;
  }
  .preset-btn:hover {
    border-color: var(--accent-dim);
    color: var(--text-primary);
  }

  /* ═══ BREADCRUMBS ═══ */
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-family: var(--font-mono);
  }
  .crumb {
    background: none;
    border: none;
    color: var(--accent-dim);
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 3px;
    font-size: 11px;
    font-family: var(--font-mono);
    transition: all 0.1s;
  }
  .crumb:hover {
    background: rgba(14, 165, 233, 0.1);
    color: var(--accent);
  }
  .crumb-current {
    color: var(--text-primary);
    font-weight: 600;
  }
  .crumb-sep {
    color: var(--text-muted);
    font-size: 12px;
  }
  .back-btn {
    background: none;
    border: 1px solid var(--border);
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 12px;
    margin-left: 6px;
    transition: all 0.15s;
  }
  .back-btn:hover {
    border-color: var(--accent-dim);
    color: var(--text-primary);
  }

  /* ═══ CONTENT ═══ */
  .content {
    flex: 1;
    position: relative;
    overflow: hidden;
  }
  .treemap-container {
    width: 100%;
    height: 100%;
  }

  /* ═══ TOOLTIP ═══ */
  .tooltip {
    position: fixed;
    background: rgba(8, 12, 20, 0.94);
    border: 1px solid rgba(56, 189, 248, 0.15);
    border-radius: 8px;
    padding: 10px 14px;
    min-width: 220px;
    max-width: 320px;
    backdrop-filter: blur(12px);
    pointer-events: none;
    z-index: 100;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    transition: opacity 0.1s;
  }
  .tt-name {
    font-weight: 700;
    font-size: 13px;
    margin-bottom: 6px;
    color: var(--text-primary);
    font-family: var(--font-mono);
  }
  .tt-row {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    margin-bottom: 2px;
  }
  .tt-label {
    color: var(--text-muted);
  }
  .tt-value {
    color: var(--text-secondary);
    font-family: var(--font-mono);
  }
  .tt-cat {
    font-size: 10px;
  }
  .tt-drill {
    font-size: 10px;
    color: var(--accent-dim);
    margin-top: 6px;
    font-style: italic;
  }
  .tt-path {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    margin-top: 6px;
    word-break: break-all;
    opacity: 0.7;
  }

  /* ═══ CATEGORIES PANEL ═══ */
  .categories-panel {
    position: absolute;
    top: 0;
    left: 0;
    width: 280px;
    height: 100%;
    background: rgba(8, 12, 20, 0.94);
    border-right: 1px solid var(--border);
    backdrop-filter: blur(12px);
    overflow-y: auto;
    padding: 14px;
    z-index: 50;
  }
  .cat-header {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--text-muted);
    margin-bottom: 14px;
  }
  .cat-row {
    margin-bottom: 12px;
  }
  .cat-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 2px;
  }
  .cat-stats {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    margin-bottom: 4px;
  }
  .cat-size { color: var(--text-secondary); }
  .cat-count { color: var(--text-muted); }
  .cat-bar-bg {
    height: 4px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 2px;
    overflow: hidden;
  }
  .cat-bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  /* ═══ CENTER PANELS ═══ */
  .center-panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
  }
  .error-icon {
    font-size: 36px;
    color: var(--critical);
    opacity: 0.7;
  }
  .error-text {
    color: var(--critical);
    font-size: 14px;
    max-width: 400px;
    text-align: center;
  }
  .retry-btn {
    padding: 6px 18px;
    background: transparent;
    border: 1px solid var(--critical);
    border-radius: 6px;
    color: var(--critical);
    cursor: pointer;
    font-size: 12px;
    transition: all 0.15s;
  }
  .retry-btn:hover {
    background: rgba(239, 68, 68, 0.1);
  }

  .loading-text {
    color: var(--text-secondary);
    font-size: 14px;
  }
  .loading-sub {
    color: var(--text-muted);
    font-size: 12px;
    font-style: italic;
  }

  /* Scan animation */
  .scan-anim {
    position: relative;
    width: 60px;
    height: 60px;
  }
  .scan-ring {
    position: absolute;
    inset: 0;
    border: 2px solid var(--accent);
    border-radius: 50%;
    opacity: 0.4;
    animation: scan-pulse 2s ease-in-out infinite;
  }
  .scan-ring.r2 {
    animation-delay: 0.5s;
    border-color: var(--accent-dim);
  }
  .scan-dot {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 8px;
    height: 8px;
    margin: -4px 0 0 -4px;
    background: var(--accent);
    border-radius: 50%;
    animation: scan-dot-pulse 1s ease-in-out infinite;
  }
  @keyframes scan-pulse {
    0% { transform: scale(0.6); opacity: 0.6; }
    50% { transform: scale(1.2); opacity: 0.2; }
    100% { transform: scale(0.6); opacity: 0.6; }
  }
  @keyframes scan-dot-pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(0.7); }
  }

  /* Empty state */
  .empty-organism {
    position: relative;
    width: 80px;
    height: 80px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .org-ring {
    position: absolute;
    inset: 0;
    border: 1px solid var(--accent-dim);
    border-radius: 50%;
    opacity: 0.2;
    animation: org-breathe 4s ease-in-out infinite;
  }
  .org-core {
    font-size: 32px;
    color: var(--accent-dim);
    opacity: 0.3;
  }
  @keyframes org-breathe {
    0%, 100% { transform: scale(0.9); opacity: 0.15; }
    50% { transform: scale(1.1); opacity: 0.3; }
  }
  .empty-title {
    color: var(--text-secondary);
    font-size: 14px;
  }
  .empty-sub {
    color: var(--text-muted);
    font-size: 12px;
    font-style: italic;
  }
  .empty-hint {
    color: var(--text-muted);
    font-size: 10px;
    font-family: var(--font-mono);
    margin-top: 8px;
    opacity: 0.5;
  }
</style>
