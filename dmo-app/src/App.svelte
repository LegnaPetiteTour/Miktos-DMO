<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Treemap from "./lib/Treemap.svelte";
  import type { ScanResult, TreeNode } from "./lib/types";

  let scanResult: ScanResult | null = $state(null);
  let scanning: boolean = $state(false);
  let error: string | null = $state(null);
  let scanPath: string = $state("");
  let maxDepth: number = $state(5);
  let hoveredNode: TreeNode | null = $state(null);
  let tooltipX: number = $state(0);
  let tooltipY: number = $state(0);
  let viewStack: TreeNode[] = $state([]);
  let categoriesOpen: boolean = $state(false);

  const currentView: TreeNode | null = $derived(
    viewStack.length > 0 ? viewStack[viewStack.length - 1] : ((scanResult as ScanResult | null)?.tree ?? null)
  );

  async function init() {
    try {
      const home: string = await invoke("get_home_dir");
      scanPath = `${home}/Library/Caches`;
    } catch { scanPath = "/tmp"; }
  }
  init();

  async function startScan() {
    if (scanning || !scanPath) return;
    scanning = true; error = null; scanResult = null; viewStack = []; categoriesOpen = false;
    try {
      scanResult = await invoke("scan_filesystem", { path: scanPath, maxDepth: maxDepth });
    } catch (e: any) {
      error = typeof e === "string" ? e : e.message || "Scan failed";
    } finally { scanning = false; }
  }

  function handleCellClick(node: TreeNode) {
    if (node.is_directory && node.children && node.children.length > 0) {
      viewStack = [...viewStack, node];
      hoveredNode = null;
    }
  }

  function navigateBack() {
    if (viewStack.length > 0) { viewStack = viewStack.slice(0, -1); hoveredNode = null; }
  }

  function navigateToLevel(index: number) {
    viewStack = index < 0 ? [] : viewStack.slice(0, index + 1);
    hoveredNode = null;
  }

  function handleHover(node: TreeNode | null, x: number, y: number) {
    hoveredNode = node; tooltipX = x; tooltipY = y;
  }

  function toggleCategories() { categoriesOpen = !categoriesOpen; }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !scanning && document.activeElement?.tagName !== "SELECT") startScan();
    if ((e.key === "Escape" || e.key === "Backspace") && viewStack.length > 0 && document.activeElement?.tagName !== "INPUT") {
      e.preventDefault(); navigateBack();
    }
    if (e.key === "Escape" && categoriesOpen) { categoriesOpen = false; }
  }

  function fmtBytes(b: number): string {
    if (b >= 1073741824) return `${(b / 1073741824).toFixed(2)} GB`;
    if (b >= 1048576) return `${(b / 1048576).toFixed(1)} MB`;
    if (b >= 1024) return `${(b / 1024).toFixed(0)} KB`;
    return `${b} B`;
  }
  function fmtScore(s: number): string { return s < 0.001 ? "<0.001" : s.toFixed(4); }
  function scoreColor(s: number): string {
    if (s >= 0.3) return "#ef4444"; if (s >= 0.15) return "#f97316"; if (s >= 0.05) return "#eab308"; return "#22c55e";
  }
  function abbrevPath(p: string): string {
    const i = p.indexOf("/Users/");
    if (i >= 0) { const a = p.substring(i + 7); const s = a.indexOf("/"); if (s >= 0) return "~" + a.substring(s); }
    return p;
  }
  function wastePercent(): string {
    if (!scanResult || !scanResult.summary.total_size) return "0";
    return ((scanResult.summary.waste_size / scanResult.summary.total_size) * 100).toFixed(1);
  }
  function canDrill(n: TreeNode | null): boolean { return !!n && n.is_directory && !!n.children && n.children.length > 0; }

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
  <header class="header">
    <div class="header-left">
      <h1 class="title"><span class="accent">DMO</span><span class="dim">Digital Maintenance Organism</span></h1>
    </div>
    {#if scanResult}
      <div class="stats">
        <div class="st"><span class="sl">Files</span><span class="sv">{scanResult.summary.total_files.toLocaleString()}</span></div>
        <div class="st"><span class="sl">Total</span><span class="sv">{fmtBytes(scanResult.summary.total_size)}</span></div>
        <div class="st waste"><span class="sl">Waste</span><span class="sv">{fmtBytes(scanResult.summary.waste_size)}</span></div>
        <div class="st waste"><span class="sl">Waste%</span><span class="sv">{wastePercent()}%</span></div>
        <div class="st"><span class="sl">Time</span><span class="sv">{scanResult.summary.scan_time_ms.toLocaleString()}ms</span></div>
      </div>
    {/if}
  </header>

  <div class="controls">
    <div class="row1">
      <input type="text" bind:value={scanPath} placeholder="Path to scan..." class="pinput" disabled={scanning} />
      <select bind:value={maxDepth} class="dsel" disabled={scanning}>
        <option value={3}>Depth 3</option><option value={4}>Depth 4</option><option value={5}>Depth 5</option><option value={6}>Depth 6</option><option value={8}>Depth 8</option>
      </select>
      <button onclick={startScan} class="sbtn" disabled={scanning}>
        {#if scanning}<span class="spin"></span>Scanning...{:else}Scan{/if}
      </button>
      {#if scanResult}
        <button class="cbtn" class:copen={categoriesOpen} onclick={toggleCategories} title="Category breakdown">☰</button>
      {/if}
    </div>
    <div class="row2">
      <div class="presets">
        <button class="pr" onclick={() => setPreset("caches")}>~/Library/Caches</button>
        <button class="pr" onclick={() => setPreset("library")}>~/Library</button>
        <button class="pr" onclick={() => setPreset("home")}>Home</button>
      </div>
      {#if viewStack.length > 0}
        <nav class="bc">
          <button class="crumb" onclick={() => navigateToLevel(-1)}>{scanResult?.tree.name ?? "Root"}</button>
          {#each viewStack as c, i}
            <span class="bsep">›</span>
            <button class="crumb" class:cur={i === viewStack.length - 1} onclick={() => navigateToLevel(i)}>{c.name}</button>
          {/each}
          <button class="bbk" onclick={navigateBack}>← Back</button>
        </nav>
      {/if}
    </div>
  </div>

  <div class="content">
    {#if error}
      <div class="mid"><p style="color: var(--critical);">Error: {error}</p><button class="rbtn" onclick={startScan}>Retry</button></div>
    {:else if scanning}
      <div class="mid">
        <div class="anim"><div class="ring"></div><div class="ring r2"></div></div>
        <p>Scanning filesystem...</p><p class="sub">Observing the terrain</p>
      </div>
    {:else if currentView}
      <Treemap tree={currentView} onHover={handleHover} onClick={handleCellClick} />
    {:else}
      <div class="mid">
        <p>Select a path and press Scan</p><p class="sub">The organism will navigate toward waste</p>
      </div>
    {/if}
  </div>
</main>

<!-- CATEGORY PANEL -->
{#if categoriesOpen && scanResult}
  <button class="cat-overlay" onclick={toggleCategories} onkeydown={(e) => e.key === 'Escape' && toggleCategories()} aria-label="Close categories"></button>
  <div class="cat-panel">
    <div class="cat-head">
      <span>Category Breakdown</span>
      <button class="cat-close" onclick={toggleCategories}>✕</button>
    </div>
    {#each [...scanResult.summary.categories].sort((a, b) => b.total_bytes - a.total_bytes) as cat}
      <div class="cat-row">
        <div class="cat-name">{cat.name}</div>
        <div class="cat-meta">
          <span>{fmtBytes(cat.total_bytes)}</span>
          <span class="cat-fc">{cat.file_count.toLocaleString()} files</span>
        </div>
        <div class="cat-bg"><div class="cat-fill" style="width: {Math.min(100, (cat.total_bytes / scanResult.summary.total_size) * 100)}%; background: {scoreColor(cat.avg_score)}"></div></div>
      </div>
    {/each}
  </div>
{/if}

<!-- TOOLTIP -->
{#if hoveredNode}
  <div class="tt" style="left:{Math.min(tooltipX + 16, (typeof window !== 'undefined' ? window.innerWidth - 320 : 700))}px;top:{Math.max(tooltipY - 10, 8)}px;">
    <div class="tt-name">{hoveredNode.name}</div>
    <div class="tt-r"><span class="tt-l">Size</span><span class="tt-v">{fmtBytes(hoveredNode.size)}</span></div>
    <div class="tt-r"><span class="tt-l">Category</span><span class="tt-v" style="font-size:10px">{hoveredNode.category}</span></div>
    <div class="tt-r"><span class="tt-l">Score</span><span class="tt-v" style="color:{scoreColor(hoveredNode.waste_score)};font-weight:700">{fmtScore(hoveredNode.waste_score)}</span></div>
    {#if hoveredNode.file_count > 1}
      <div class="tt-r"><span class="tt-l">Files</span><span class="tt-v">{hoveredNode.file_count.toLocaleString()}</span></div>
    {/if}
    {#if canDrill(hoveredNode)}
      <div class="tt-r"><span class="tt-l">Contents</span><span class="tt-v">{hoveredNode.children.length} items</span></div>
      <div class="tt-drill">⬊ Click zone to explore inside</div>
    {:else if !hoveredNode.is_directory}
      <div class="tt-file-tag">File</div>
    {/if}
    <div class="tt-path">{abbrevPath(hoveredNode.path)}</div>
  </div>
{/if}

<style>
  .app { display: flex; flex-direction: column; height: 100vh; overflow: hidden; }

  .header { display: flex; justify-content: space-between; align-items: center; padding: 10px 20px; background: var(--bg-secondary); border-bottom: 1px solid var(--border); flex-shrink: 0; }
  .title { display: flex; align-items: baseline; gap: 8px; }
  .accent { font-weight: 800; color: var(--accent); font-family: var(--font-mono); font-size: 16px; letter-spacing: 1px; }
  .dim { color: var(--text-muted); font-size: 11px; font-weight: 400; }
  .stats { display: flex; gap: 18px; }
  .st { display: flex; flex-direction: column; align-items: flex-end; }
  .sl { font-size: 9px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.6px; font-weight: 600; }
  .sv { font-size: 13px; font-family: var(--font-mono); color: var(--text-primary); }
  .st.waste .sv { color: var(--danger); }

  .controls { padding: 8px 20px 10px; background: var(--bg-secondary); border-bottom: 1px solid var(--border); flex-shrink: 0; }
  .row1 { display: flex; gap: 6px; }
  .row2 { display: flex; justify-content: space-between; align-items: center; margin-top: 6px; min-height: 24px; }
  .pinput { flex: 1; padding: 7px 12px; background: var(--bg-primary); border: 1px solid var(--border); border-radius: 6px; color: var(--text-primary); font-family: var(--font-mono); font-size: 12px; outline: none; }
  .pinput:focus { border-color: var(--accent-dim); }
  .dsel { padding: 7px 10px; background: var(--bg-primary); border: 1px solid var(--border); border-radius: 6px; color: var(--text-primary); font-size: 12px; outline: none; cursor: pointer; }
  .sbtn { padding: 7px 22px; background: var(--accent-dim); border: none; border-radius: 6px; color: white; font-weight: 600; font-size: 12px; cursor: pointer; display: flex; align-items: center; gap: 6px; }
  .sbtn:hover:not(:disabled) { background: var(--accent); }
  .sbtn:disabled { opacity: 0.6; cursor: not-allowed; }
  .spin { display: inline-block; width: 12px; height: 12px; border: 2px solid rgba(255,255,255,0.3); border-top-color: white; border-radius: 50%; animation: sp 0.6s linear infinite; }
  @keyframes sp { to { transform: rotate(360deg); } }

  .cbtn { padding: 7px 12px; background: var(--bg-surface); border: 1px solid var(--border); border-radius: 6px; color: var(--text-secondary); font-size: 16px; cursor: pointer; line-height: 1; }
  .cbtn:hover, .copen { border-color: var(--accent-dim); color: var(--accent); background: rgba(14,165,233,0.08); }

  .presets { display: flex; gap: 5px; }
  .pr { padding: 3px 9px; background: transparent; border: 1px solid var(--border); border-radius: 4px; color: var(--text-muted); font-size: 10px; font-family: var(--font-mono); cursor: pointer; }
  .pr:hover { border-color: var(--accent-dim); color: var(--text-primary); }

  .bc { display: flex; align-items: center; gap: 4px; }
  .crumb { background: none; border: none; color: var(--accent-dim); cursor: pointer; padding: 2px 4px; border-radius: 3px; font-size: 11px; font-family: var(--font-mono); }
  .crumb:hover { background: rgba(14,165,233,0.1); color: var(--accent); }
  .crumb.cur { color: var(--text-primary); font-weight: 600; }
  .bsep { color: var(--text-muted); font-size: 12px; }
  .bbk { background: none; border: 1px solid var(--border); color: var(--text-muted); cursor: pointer; padding: 2px 8px; border-radius: 4px; font-size: 11px; margin-left: 6px; font-family: var(--font-mono); }
  .bbk:hover { border-color: var(--accent-dim); color: var(--text-primary); }

  .content { flex: 1; position: relative; overflow: hidden; }

  .mid { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 12px; color: var(--text-secondary); }
  .sub { color: var(--text-muted); font-size: 12px; font-style: italic; }
  .rbtn { padding: 6px 18px; background: transparent; border: 1px solid var(--critical); border-radius: 6px; color: var(--critical); cursor: pointer; font-size: 12px; }
  .anim { position: relative; width: 50px; height: 50px; }
  .ring { position: absolute; inset: 0; border: 2px solid var(--accent); border-radius: 50%; opacity: 0.4; animation: pul 2s ease-in-out infinite; }
  .ring.r2 { animation-delay: 0.5s; border-color: var(--accent-dim); }
  @keyframes pul { 0% { transform: scale(0.6); opacity: 0.6; } 50% { transform: scale(1.2); opacity: 0.2; } 100% { transform: scale(0.6); opacity: 0.6; } }

  .cat-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.4); z-index: 900; border: none; cursor: default; }
  .cat-panel {
    position: fixed; top: 0; left: 0; z-index: 1000;
    width: 320px; height: 100vh;
    background: rgba(8,12,20,0.98); border-right: 1px solid var(--border);
    overflow-y: auto; padding: 16px;
    box-shadow: 4px 0 24px rgba(0,0,0,0.5);
  }
  .cat-head { display: flex; justify-content: space-between; align-items: center; font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.8px; color: var(--text-muted); margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border); }
  .cat-close { background: none; border: none; color: var(--text-muted); cursor: pointer; font-size: 14px; padding: 4px 8px; border-radius: 4px; }
  .cat-close:hover { color: var(--text-primary); background: rgba(255,255,255,0.05); }
  .cat-row { margin-bottom: 14px; }
  .cat-name { font-size: 12px; font-weight: 600; color: var(--text-primary); margin-bottom: 2px; }
  .cat-meta { display: flex; justify-content: space-between; font-size: 10px; font-family: var(--font-mono); margin-bottom: 4px; color: var(--text-secondary); }
  .cat-fc { color: var(--text-muted); }
  .cat-bg { height: 4px; background: rgba(255,255,255,0.05); border-radius: 2px; overflow: hidden; }
  .cat-fill { height: 100%; border-radius: 2px; }

  .tt { position: fixed; z-index: 800; background: rgba(8,12,20,0.95); border: 1px solid rgba(56,189,248,0.15); border-radius: 8px; padding: 10px 14px; min-width: 220px; max-width: 320px; backdrop-filter: blur(12px); pointer-events: none; box-shadow: 0 8px 32px rgba(0,0,0,0.6); }
  .tt-name { font-weight: 700; font-size: 13px; margin-bottom: 6px; color: var(--text-primary); font-family: var(--font-mono); }
  .tt-r { display: flex; justify-content: space-between; font-size: 11px; margin-bottom: 2px; }
  .tt-l { color: var(--text-muted); }
  .tt-v { color: var(--text-secondary); font-family: var(--font-mono); }
  .tt-drill { font-size: 10px; color: var(--accent); margin-top: 6px; font-weight: 600; }
  .tt-file-tag { font-size: 9px; color: var(--text-muted); margin-top: 4px; text-transform: uppercase; letter-spacing: 0.5px; }
  .tt-path { font-size: 10px; color: var(--text-muted); font-family: var(--font-mono); margin-top: 6px; word-break: break-all; opacity: 0.7; }
</style>
