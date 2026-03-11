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

  let hoveredNode: TreeNode | null = $state(null);
  let tooltipX: number = $state(0);
  let tooltipY: number = $state(0);

  let viewStack: TreeNode[] = $state([]);
  let currentView: TreeNode | null = $derived(
    viewStack.length > 0 ? viewStack[viewStack.length - 1] : scanResult?.tree ?? null
  );

  let showCategories: boolean = $state(false);

  // ─── Initialize ───
  async function init() {
    try {
      const home: string = await invoke("get_home_dir");
      scanPath = `${home}/Library/Caches`;
    } catch { scanPath = "/tmp"; }
  }
  init();

  // ─── Scan ───
  async function startScan() {
    if (scanning || !scanPath) return;
    scanning = true;
    error = null;
    scanResult = null;
    viewStack = [];
    showCategories = false;

    try {
      scanResult = await invoke("scan_filesystem", { path: scanPath, maxDepth: maxDepth });
    } catch (e: any) {
      error = typeof e === "string" ? e : e.message || "Scan failed";
    } finally {
      scanning = false;
    }
  }

  // ─── Navigation ───
  function handleCellClick(node: TreeNode) {
    // Only drill into directories that have children
    if (node.is_directory && node.children && node.children.length > 0) {
      viewStack = [...viewStack, node];
      hoveredNode = null;
    }
  }

  function navigateBack() {
    if (viewStack.length > 0) {
      viewStack = viewStack.slice(0, -1);
      hoveredNode = null;
    }
  }

  function navigateToLevel(index: number) {
    viewStack = index < 0 ? [] : viewStack.slice(0, index + 1);
    hoveredNode = null;
  }

  // ─── Tooltip ───
  function handleHover(node: TreeNode | null, x: number, y: number) {
    hoveredNode = node;
    tooltipX = x;
    tooltipY = y;
  }

  // ─── Keyboard ───
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !scanning && document.activeElement?.tagName !== "SELECT") {
      startScan();
    }
    if ((e.key === "Escape" || e.key === "Backspace") && viewStack.length > 0) {
      if (document.activeElement?.tagName !== "INPUT") {
        e.preventDefault();
        navigateBack();
      }
    }
  }

  // ─── Helpers ───
  function fmtBytes(bytes: number): string {
    if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
    if (bytes >= 1024 ** 2) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${bytes} B`;
  }

  function fmtScore(s: number): string {
    return s < 0.001 ? "<0.001" : s.toFixed(4);
  }

  function scoreColor(s: number): string {
    if (s >= 0.3) return "#ef4444";
    if (s >= 0.15) return "#f97316";
    if (s >= 0.05) return "#eab308";
    return "#22c55e";
  }

  function abbrevPath(path: string): string {
    const i = path.indexOf("/Users/");
    if (i >= 0) {
      const after = path.substring(i + 7);
      const slash = after.indexOf("/");
      if (slash >= 0) return "~" + after.substring(slash);
    }
    return path;
  }

  function wastePercent(): string {
    if (!scanResult || !scanResult.summary.total_size) return "0";
    return ((scanResult.summary.waste_size / scanResult.summary.total_size) * 100).toFixed(1);
  }

  function canDrillDown(node: TreeNode | null): boolean {
    return !!node && node.is_directory && !!node.children && node.children.length > 0;
  }

  async function setPreset(p: string) {
    try {
      const home: string = await invoke("get_home_dir");
      if (p === "caches") { scanPath = `${home}/Library/Caches`; maxDepth = 4; }
      else if (p === "library") { scanPath = `${home}/Library`; maxDepth = 5; }
      else if (p === "home") { scanPath = home; maxDepth = 4; }
    } catch {}
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="app">
  <!-- HEADER -->
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
          <div class="stat"><span class="stat-label">Files</span><span class="stat-value">{scanResult.summary.total_files.toLocaleString()}</span></div>
          <div class="stat"><span class="stat-label">Total</span><span class="stat-value">{fmtBytes(scanResult.summary.total_size)}</span></div>
          <div class="stat waste"><span class="stat-label">Waste</span><span class="stat-value">{fmtBytes(scanResult.summary.waste_size)}</span></div>
          <div class="stat waste"><span class="stat-label">Waste%</span><span class="stat-value">{wastePercent()}%</span></div>
          <div class="stat"><span class="stat-label">Time</span><span class="stat-value">{scanResult.summary.scan_time_ms.toLocaleString()}ms</span></div>
        </div>
      {/if}
    </div>
  </header>

  <!-- CONTROLS -->
  <div class="controls">
    <div class="input-group">
      <input type="text" bind:value={scanPath} placeholder="Path to scan..." class="path-input" disabled={scanning} />
      <select bind:value={maxDepth} class="depth-select" disabled={scanning}>
        <option value={3}>Depth 3</option>
        <option value={4}>Depth 4</option>
        <option value={5}>Depth 5</option>
        <option value={6}>Depth 6</option>
        <option value={8}>Depth 8</option>
      </select>
      <button onclick={startScan} class="scan-btn" disabled={scanning}>
        {#if scanning}<span class="spinner"></span> Scanning...{:else}Scan{/if}
      </button>
      {#if scanResult}
        <button class="cat-toggle" class:cat-active={showCategories} onclick={() => { showCategories = !showCategories; }} title="Category breakdown">☰</button>
      {/if}
    </div>
    <div class="row2">
      <div class="presets">
        <button class="preset" onclick={() => setPreset("caches")}>~/Library/Caches</button>
        <button class="preset" onclick={() => setPreset("library")}>~/Library</button>
        <button class="preset" onclick={() => setPreset("home")}>Home</button>
      </div>
      {#if viewStack.length > 0}
        <nav class="breadcrumb">
          <button class="crumb" onclick={() => navigateToLevel(-1)}>{scanResult?.tree.name ?? "Root"}</button>
          {#each viewStack as crumb, i}
            <span class="sep">›</span>
            <button class="crumb" class:current={i === viewStack.length - 1} onclick={() => navigateToLevel(i)}>{crumb.name}</button>
          {/each}
          <button class="back-btn" onclick={navigateBack} title="Back (Esc)">←</button>
        </nav>
      {/if}
    </div>
  </div>

  <!-- MAIN -->
  <div class="content">
    {#if error}
      <div class="center"><div class="err-icon">✕</div><p class="err-text">{error}</p><button class="retry" onclick={startScan}>Retry</button></div>
    {:else if scanning}
      <div class="center">
        <div class="scan-anim"><div class="ring"></div><div class="ring r2"></div><div class="dot"></div></div>
        <p class="load-text">Scanning filesystem...</p>
        <p class="load-sub">Observing the terrain</p>
      </div>
    {:else if currentView}
      <div class="treemap-area">
        <Treemap tree={currentView} onHover={handleHover} onClick={handleCellClick} />
      </div>
    {:else}
      <div class="center">
        <div class="empty-org"><div class="org-ring"></div><div class="org-dot">◉</div></div>
        <p>Select a path and scan to map the terrain</p>
        <p class="sub">The organism will navigate toward waste concentrations</p>
        <p class="hint">Press Enter to scan</p>
      </div>
    {/if}

    <!-- CATEGORY PANEL — always on top of treemap -->
    {#if showCategories && scanResult}
      <div class="cat-panel">
        <div class="cat-title">Category Breakdown <button class="cat-close" onclick={() => { showCategories = false; }}>✕</button></div>
        {#each [...scanResult.summary.categories].sort((a, b) => b.total_bytes - a.total_bytes) as cat}
          <div class="cat-row">
            <div class="cat-name">{cat.name}</div>
            <div class="cat-meta">
              <span>{fmtBytes(cat.total_bytes)}</span>
              <span class="cat-count">{cat.file_count.toLocaleString()} files</span>
            </div>
            <div class="cat-bar"><div class="cat-fill" style="width: {Math.min(100, (cat.total_bytes / scanResult.summary.total_size) * 100)}%; background: {scoreColor(cat.avg_score)}"></div></div>
          </div>
        {/each}
      </div>
    {/if}

    <!-- TOOLTIP -->
    {#if hoveredNode}
      <div class="tooltip" style="left: {Math.min(tooltipX + 16, (typeof window !== 'undefined' ? window.innerWidth - 320 : 700))}px; top: {Math.max(tooltipY - 10, 8)}px;">
        <div class="tt-name">{hoveredNode.name}</div>
        <div class="tt-r"><span class="tt-l">Size</span><span class="tt-v">{fmtBytes(hoveredNode.size)}</span></div>
        <div class="tt-r"><span class="tt-l">Category</span><span class="tt-v tt-cat">{hoveredNode.category}</span></div>
        <div class="tt-r"><span class="tt-l">Score</span><span class="tt-v" style="color: {scoreColor(hoveredNode.waste_score)}; font-weight: 700;">{fmtScore(hoveredNode.waste_score)}</span></div>
        {#if hoveredNode.file_count > 1}
          <div class="tt-r"><span class="tt-l">Files</span><span class="tt-v">{hoveredNode.file_count.toLocaleString()}</span></div>
        {/if}
        {#if canDrillDown(hoveredNode)}
          <div class="tt-r"><span class="tt-l">Contents</span><span class="tt-v">{hoveredNode.children.length} items</span></div>
          <div class="tt-drill">Click to explore →</div>
        {/if}
        <div class="tt-path">{abbrevPath(hoveredNode.path)}</div>
      </div>
    {/if}
  </div>
</main>

<style>
  .app { display: flex; flex-direction: column; height: 100vh; overflow: hidden; }

  /* Header */
  .header { display: flex; justify-content: space-between; align-items: center; padding: 10px 20px; background: var(--bg-secondary); border-bottom: 1px solid var(--border); flex-shrink: 0; }
  .title { font-size: 14px; font-weight: 400; display: flex; align-items: baseline; gap: 8px; }
  .title-accent { font-weight: 800; color: var(--accent); font-family: var(--font-mono); font-size: 16px; letter-spacing: 1px; }
  .title-dim { color: var(--text-muted); font-size: 11px; }
  .stats-bar { display: flex; gap: 18px; }
  .stat { display: flex; flex-direction: column; align-items: flex-end; }
  .stat-label { font-size: 9px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.6px; font-weight: 600; }
  .stat-value { font-size: 13px; font-family: var(--font-mono); color: var(--text-primary); }
  .stat.waste .stat-value { color: var(--danger); }

  /* Controls */
  .controls { padding: 8px 20px 10px; background: var(--bg-secondary); border-bottom: 1px solid var(--border); flex-shrink: 0; }
  .input-group { display: flex; gap: 6px; }
  .row2 { display: flex; justify-content: space-between; align-items: center; margin-top: 6px; min-height: 24px; }
  .path-input { flex: 1; padding: 7px 12px; background: var(--bg-primary); border: 1px solid var(--border); border-radius: 6px; color: var(--text-primary); font-family: var(--font-mono); font-size: 12px; outline: none; }
  .path-input:focus { border-color: var(--accent-dim); }
  .depth-select { padding: 7px 10px; background: var(--bg-primary); border: 1px solid var(--border); border-radius: 6px; color: var(--text-primary); font-size: 12px; outline: none; cursor: pointer; }
  .scan-btn { padding: 7px 22px; background: var(--accent-dim); border: none; border-radius: 6px; color: white; font-weight: 600; font-size: 12px; cursor: pointer; display: flex; align-items: center; gap: 6px; }
  .scan-btn:hover:not(:disabled) { background: var(--accent); }
  .scan-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .spinner { display: inline-block; width: 12px; height: 12px; border: 2px solid rgba(255,255,255,0.3); border-top-color: white; border-radius: 50%; animation: spin 0.6s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .cat-toggle { padding: 7px 12px; background: transparent; border: 1px solid var(--border); border-radius: 6px; color: var(--text-muted); font-size: 14px; cursor: pointer; }
  .cat-toggle:hover, .cat-active { border-color: var(--accent-dim); color: var(--accent); background: rgba(14, 165, 233, 0.08); }

  .presets { display: flex; gap: 5px; }
  .preset { padding: 3px 9px; background: transparent; border: 1px solid var(--border); border-radius: 4px; color: var(--text-muted); font-size: 10px; font-family: var(--font-mono); cursor: pointer; }
  .preset:hover { border-color: var(--accent-dim); color: var(--text-primary); }

  /* Breadcrumbs */
  .breadcrumb { display: flex; align-items: center; gap: 4px; }
  .crumb { background: none; border: none; color: var(--accent-dim); cursor: pointer; padding: 2px 4px; border-radius: 3px; font-size: 11px; font-family: var(--font-mono); }
  .crumb:hover { background: rgba(14, 165, 233, 0.1); color: var(--accent); }
  .crumb.current { color: var(--text-primary); font-weight: 600; }
  .sep { color: var(--text-muted); font-size: 12px; }
  .back-btn { background: none; border: 1px solid var(--border); color: var(--text-muted); cursor: pointer; padding: 2px 8px; border-radius: 4px; font-size: 12px; margin-left: 6px; }
  .back-btn:hover { border-color: var(--accent-dim); color: var(--text-primary); }

  /* Content */
  .content { flex: 1; position: relative; overflow: hidden; }
  .treemap-area { width: 100%; height: 100%; }

  /* Tooltip — fixed positioning so it's never clipped */
  .tooltip {
    position: fixed; z-index: 200;
    background: rgba(8, 12, 20, 0.95); border: 1px solid rgba(56, 189, 248, 0.15);
    border-radius: 8px; padding: 10px 14px; min-width: 220px; max-width: 320px;
    backdrop-filter: blur(12px); pointer-events: none;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
  }
  .tt-name { font-weight: 700; font-size: 13px; margin-bottom: 6px; color: var(--text-primary); font-family: var(--font-mono); }
  .tt-r { display: flex; justify-content: space-between; font-size: 11px; margin-bottom: 2px; }
  .tt-l { color: var(--text-muted); }
  .tt-v { color: var(--text-secondary); font-family: var(--font-mono); }
  .tt-cat { font-size: 10px; }
  .tt-drill { font-size: 10px; color: var(--accent); margin-top: 6px; font-weight: 600; }
  .tt-path { font-size: 10px; color: var(--text-muted); font-family: var(--font-mono); margin-top: 6px; word-break: break-all; opacity: 0.7; }

  /* Category panel — overlays content area */
  .cat-panel {
    position: absolute; top: 0; left: 0; z-index: 100;
    width: 300px; height: 100%;
    background: rgba(8, 12, 20, 0.96); border-right: 1px solid var(--border);
    backdrop-filter: blur(16px); overflow-y: auto; padding: 16px;
  }
  .cat-title { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.8px; color: var(--text-muted); margin-bottom: 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center; }
  .cat-close { background: transparent; border: none; color: var(--text-muted); font-size: 13px; cursor: pointer; padding: 0 2px; line-height: 1; }
  .cat-close:hover { color: var(--text-primary); }
  .cat-row { margin-bottom: 14px; }
  .cat-name { font-size: 12px; font-weight: 600; color: var(--text-primary); margin-bottom: 2px; }
  .cat-meta { display: flex; justify-content: space-between; font-size: 10px; font-family: var(--font-mono); margin-bottom: 4px; color: var(--text-secondary); }
  .cat-count { color: var(--text-muted); }
  .cat-bar { height: 4px; background: rgba(255,255,255,0.05); border-radius: 2px; overflow: hidden; }
  .cat-fill { height: 100%; border-radius: 2px; }

  /* Center panels */
  .center { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 12px; color: var(--text-secondary); }
  .err-icon { font-size: 36px; color: var(--critical); opacity: 0.7; }
  .err-text { color: var(--critical); font-size: 14px; text-align: center; max-width: 400px; }
  .retry { padding: 6px 18px; background: transparent; border: 1px solid var(--critical); border-radius: 6px; color: var(--critical); cursor: pointer; font-size: 12px; }
  .retry:hover { background: rgba(239, 68, 68, 0.1); }
  .load-text { font-size: 14px; }
  .load-sub, .sub { color: var(--text-muted); font-size: 12px; font-style: italic; }
  .hint { color: var(--text-muted); font-size: 10px; font-family: var(--font-mono); margin-top: 8px; opacity: 0.5; }

  .scan-anim { position: relative; width: 60px; height: 60px; }
  .ring { position: absolute; inset: 0; border: 2px solid var(--accent); border-radius: 50%; opacity: 0.4; animation: pulse 2s ease-in-out infinite; }
  .ring.r2 { animation-delay: 0.5s; border-color: var(--accent-dim); }
  .dot { position: absolute; top: 50%; left: 50%; width: 8px; height: 8px; margin: -4px 0 0 -4px; background: var(--accent); border-radius: 50%; animation: dotpulse 1s ease-in-out infinite; }
  @keyframes pulse { 0% { transform: scale(0.6); opacity: 0.6; } 50% { transform: scale(1.2); opacity: 0.2; } 100% { transform: scale(0.6); opacity: 0.6; } }
  @keyframes dotpulse { 0%,100% { opacity: 1; } 50% { opacity: 0.4; transform: scale(0.7); } }

  .empty-org { position: relative; width: 80px; height: 80px; display: flex; align-items: center; justify-content: center; }
  .org-ring { position: absolute; inset: 0; border: 1px solid var(--accent-dim); border-radius: 50%; opacity: 0.2; animation: breathe 4s ease-in-out infinite; }
  .org-dot { font-size: 32px; color: var(--accent-dim); opacity: 0.3; }
  @keyframes breathe { 0%,100% { transform: scale(0.9); opacity: 0.15; } 50% { transform: scale(1.1); opacity: 0.3; } }
</style>
