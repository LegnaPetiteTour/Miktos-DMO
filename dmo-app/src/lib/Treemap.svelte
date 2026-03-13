<script lang="ts">
  import { onMount } from "svelte";
  import { hierarchy } from "d3-hierarchy";
  import { voronoiTreemap } from "d3-voronoi-treemap";
  import type { TreeNode, TerrainCell } from "./types";

  let {
    tree,
    onHover,
    onClick,
    onCells,
  }: {
    tree: TreeNode;
    onHover: (node: TreeNode | null, x: number, y: number) => void;
    onClick: (node: TreeNode) => void;
    /** Called after each layout computation with the screen-space terrain map.
     *  Phase 2 uses this to build the Physarum chemoattractant texture. */
    onCells?: (cells: TerrainCell[]) => void;
  } = $props();

  let canvas: HTMLCanvasElement | undefined = $state(undefined);
  let container: HTMLDivElement | undefined = $state(undefined);
  let width = $state(800);
  let height = $state(600);
  let cells: RenderedCell[] = $state([]);
  let hoveredIndex: number = $state(-1);
  let mounted = false;
  let layoutVersion = 0;

  const MAX_ZONES = 80;

  interface RenderedCell {
    polygon: [number, number][];
    node: TreeNode;
    fillColor: string;
    glowColor: string;
    centroid: [number, number];
    area: number;
    drillable: boolean;
  }

  function visualIntensity(score: number, size: number): number {
    const scorePart = Math.pow(Math.min(score * 2.5, 1.0), 0.55);
    const mb = size / (1024 * 1024);
    const sizeBoost = mb > 100 ? Math.min(Math.log10(mb / 100) * 0.15, 0.25) : 0;
    return Math.min(scorePart + sizeBoost, 1.0);
  }

  function wasteColor(score: number, size: number): string {
    const s = visualIntensity(score, size);
    let r: number, g: number, b: number;
    if (s < 0.20)      { const t = s/0.20;        r = lerp(10,14,t);  g = lerp(30,80,t);   b = lerp(60,95,t); }
    else if (s < 0.40)  { const t = (s-0.20)/0.20; r = lerp(14,50,t);  g = lerp(80,115,t);  b = lerp(95,80,t); }
    else if (s < 0.60)  { const t = (s-0.40)/0.20; r = lerp(50,185,t); g = lerp(115,125,t); b = lerp(80,25,t); }
    else if (s < 0.80)  { const t = (s-0.60)/0.20; r = lerp(185,225,t); g = lerp(125,80,t); b = lerp(25,10,t); }
    else                 { const t = (s-0.80)/0.20; r = lerp(225,200,t); g = lerp(80,25,t);  b = lerp(10,25,t); }
    return `rgb(${Math.round(r)},${Math.round(g)},${Math.round(b)})`;
  }

  function glowColor(score: number, size: number): string {
    const s = visualIntensity(score, size);
    if (s > 0.6) return "rgba(239,68,68,0.28)";
    if (s > 0.4) return "rgba(245,158,11,0.20)";
    if (s > 0.2) return "rgba(20,184,166,0.12)";
    return "rgba(56,189,248,0.06)";
  }

  function borderStyle(score: number, size: number): string {
    const s = visualIntensity(score, size);
    if (s > 0.6) return "rgba(239,68,68,0.45)";
    if (s > 0.4) return "rgba(245,158,11,0.28)";
    if (s > 0.2) return "rgba(100,200,200,0.16)";
    return "rgba(56,189,248,0.08)";
  }

  function lerp(a: number, b: number, t: number): number { return a + (b - a) * t; }

  function prepareChildren(treeData: TreeNode): TreeNode[] {
    let items = (treeData.children || []).filter(c => c.size > 0);
    if (items.length === 0) return [];
    items.sort((a, b) => b.size - a.size);
    if (items.length <= MAX_ZONES) return items;
    const keep = items.slice(0, MAX_ZONES - 1);
    const rest = items.slice(MAX_ZONES - 1);
    const otherSize = rest.reduce((s, c) => s + c.size, 0);
    const otherFiles = rest.reduce((s, c) => s + (c.file_count || 1), 0);
    const otherScore = otherSize > 0 ? rest.reduce((s, c) => s + c.waste_score * c.size, 0) / otherSize : 0;
    return [...keep, {
      name: `Other (${rest.length} items)`, path: treeData.path + "/__other__",
      size: otherSize, file_count: otherFiles, waste_score: otherScore,
      category: "Mixed", is_directory: false, children: [],
    }];
  }

  function computeLayout(treeData: TreeNode, w: number, h: number): RenderedCell[] {
    const items = prepareChildren(treeData);
    if (items.length === 0) return [];
    const leaves = items.map(item => ({
      name: item.name, path: item.path, size: Math.sqrt(item.size),
      waste_score: item.waste_score, file_count: item.file_count,
      category: item.category, is_directory: item.is_directory,
    }));
    const wrapper = { name: treeData.name, size: 0, children: leaves };
    const root = hierarchy(wrapper, (d: any) => d.children && d.children.length > 0 ? d.children : null)
      .sum((d: any) => (!d.children || d.children.length === 0) ? Math.max(d.size || 0, 1) : 0)
      .sort((a: any, b: any) => (b.value || 0) - (a.value || 0));
    if (!root.value) return [];
    const pad = 6;
    const clip: [number, number][] = [[pad, pad], [w - pad, pad], [w - pad, h - pad], [pad, h - pad]];
    try { voronoiTreemap().clip(clip).minWeightRatio(0.01).maxIterationCount(150)(root); }
    catch (e) { console.warn("[DMO] Voronoi failed:", e); return []; }
    const result: RenderedCell[] = [];
    for (const leaf of root.leaves()) {
      const poly = (leaf as any).polygon;
      if (!poly || poly.length < 3) continue;
      const ld = leaf.data as any;
      const orig = items.find(i => i.path === ld.path) || ld;
      const points: [number, number][] = poly.map((p: any) => [p[0], p[1]]);
      let cx = 0, cy = 0;
      for (const [x, y] of points) { cx += x; cy += y; }
      cx /= points.length; cy /= points.length;
      result.push({
        polygon: points, node: orig as TreeNode,
        fillColor: wasteColor(orig.waste_score, orig.size),
        glowColor: glowColor(orig.waste_score, orig.size),
        centroid: [cx, cy], area: polyArea(points),
        drillable: orig.is_directory && orig.children && orig.children.length > 0,
      });
    }
    return result;
  }

  function render() {
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr; canvas.height = height * dpr; ctx.scale(dpr, dpr);
    ctx.fillStyle = "#080c14"; ctx.fillRect(0, 0, width, height);
    if (cells.length === 0) {
      ctx.fillStyle = "rgba(255,255,255,0.25)"; ctx.font = "13px monospace";
      ctx.textAlign = "center"; ctx.fillText("No zones to display", width / 2, height / 2); return;
    }
    for (let i = 0; i < cells.length; i++) {
      const cell = cells[i]; if (cell.polygon.length < 3) continue;
      const hov = i === hoveredIndex;
      ctx.beginPath(); ctx.moveTo(cell.polygon[0][0], cell.polygon[0][1]);
      for (let j = 1; j < cell.polygon.length; j++) ctx.lineTo(cell.polygon[j][0], cell.polygon[j][1]);
      ctx.closePath();
      ctx.fillStyle = cell.fillColor; ctx.fill();
      if (!hov && cell.node.waste_score > 0.02) { ctx.fillStyle = cell.glowColor; ctx.fill(); }
      if (hov) {
        ctx.fillStyle = cell.drillable ? "rgba(56,189,248,0.12)" : "rgba(255,255,255,0.10)"; ctx.fill();
        ctx.strokeStyle = cell.drillable ? "rgba(56,189,248,0.6)" : "rgba(255,255,255,0.5)";
        ctx.lineWidth = 2.5; ctx.stroke();
      } else { ctx.strokeStyle = borderStyle(cell.node.waste_score, cell.node.size); ctx.lineWidth = 0.6; ctx.stroke(); }
      if (cell.drillable && !hov && cell.area > 3000) {
        ctx.save(); ctx.strokeStyle = "rgba(56,189,248,0.12)"; ctx.lineWidth = 0.4;
        ctx.setLineDash([3, 4]); ctx.stroke(); ctx.restore();
      }
      if (cell.area > 1200) {
        const maxDim = Math.sqrt(cell.area);
        const fs = Math.max(8, Math.min(14, maxDim / 7));
        ctx.font = `500 ${fs}px "SF Mono","Fira Code",monospace`;
        ctx.textAlign = "center"; ctx.textBaseline = "middle";
        let lbl = cell.node.name; const mw = maxDim * 0.75;
        while (ctx.measureText(lbl).width > mw && lbl.length > 4) lbl = lbl.slice(0, -2) + "\u2026";
        ctx.fillStyle = "rgba(0,0,0,0.75)"; ctx.fillText(lbl, cell.centroid[0] + 1, cell.centroid[1] + 1);
        const alpha = hov ? 1 : (visualIntensity(cell.node.waste_score, cell.node.size) > 0.3 ? 0.9 : 0.6);
        ctx.fillStyle = `rgba(255,255,255,${alpha})`; ctx.fillText(lbl, cell.centroid[0], cell.centroid[1]);
        if (cell.area > 6000) {
          ctx.font = `400 ${Math.max(7, fs - 3)}px "SF Mono",monospace`;
          ctx.fillStyle = `rgba(255,255,255,${hov ? 0.65 : 0.28})`;
          ctx.fillText(fmtB(cell.node.size), cell.centroid[0], cell.centroid[1] + fs);
        }
      }
    }
  }

  function fmtB(b: number): string {
    if (b >= 1073741824) return `${(b / 1073741824).toFixed(1)}G`;
    if (b >= 1048576) return `${(b / 1048576).toFixed(0)}M`;
    if (b >= 1024) return `${(b / 1024).toFixed(0)}K`;
    return `${b}B`;
  }
  function polyArea(pts: [number, number][]): number {
    let a = 0; for (let i = 0, n = pts.length; i < n; i++) { const j = (i + 1) % n; a += pts[i][0] * pts[j][1] - pts[j][0] * pts[i][1]; } return Math.abs(a / 2);
  }
  function pip(x: number, y: number, poly: [number, number][]): boolean {
    let inside = false; for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
      const [xi, yi] = poly[i], [xj, yj] = poly[j];
      if ((yi > y) !== (yj > y) && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) inside = !inside;
    } return inside;
  }

  function handleMouseMove(e: MouseEvent) {
    if (!canvas || cells.length === 0) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left, y = e.clientY - rect.top;
    let idx = -1;
    for (let i = 0; i < cells.length; i++) { if (pip(x, y, cells[i].polygon)) { idx = i; break; } }
    canvas.style.cursor = (idx >= 0 && cells[idx].drillable) ? "pointer" : "crosshair";
    if (idx !== hoveredIndex) { hoveredIndex = idx; render(); }
    onHover(idx >= 0 ? cells[idx].node : null, e.clientX, e.clientY);
  }
  function handleMouseLeave() {
    if (canvas) canvas.style.cursor = "crosshair";
    if (hoveredIndex !== -1) { hoveredIndex = -1; render(); }
    onHover(null, 0, 0);
  }
  function handleClick(e: MouseEvent) {
    if (!canvas || cells.length === 0) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left, y = e.clientY - rect.top;
    for (let i = 0; i < cells.length; i++) { if (pip(x, y, cells[i].polygon)) { onClick(cells[i].node); return; } }
  }

  function updateSize() {
    if (!container) return;
    const r = container.getBoundingClientRect();
    const nw = Math.floor(r.width), nh = Math.floor(r.height);
    if (nw > 0 && nh > 0 && (nw !== width || nh !== height)) { width = nw; height = nh; }
  }
  function doLayout() {
    if (!tree || width <= 0 || height <= 0 || !canvas) return;
    const v = ++layoutVersion;
    cells = computeLayout(tree, width, height);
    if (v === layoutVersion) {
      render();
      // Emit terrain layout so the Phase 2 simulation can build its chemoattractant map
      onCells?.(cells.map(c => ({
        polygon: c.polygon,
        path: c.node.path,
        waste_score: c.node.waste_score,
        centroid: c.centroid,
        area: c.area,
      })));
    }
  }

  onMount(() => {
    updateSize(); mounted = true; doLayout();
    const obs = new ResizeObserver(() => { updateSize(); doLayout(); });
    if (container) obs.observe(container);
    return () => obs.disconnect();
  });

  $effect(() => {
    if (!mounted) return;
    const _t = tree, _w = width, _h = height;
    if (_t && _w > 0 && _h > 0) requestAnimationFrame(doLayout);
  });
</script>

<div class="treemap-wrapper no-select" bind:this={container}>
  <canvas bind:this={canvas} style="width: {width}px; height: {height}px;"
    onmousemove={handleMouseMove} onmouseleave={handleMouseLeave} onclick={handleClick}></canvas>
  <div class="legend">
    <div class="legend-title">Terrain Health</div>
    <div class="legend-gradient"></div>
    <div class="legend-labels"><span>Clean</span><span>Moderate</span><span>Critical</span></div>
    <div class="legend-count">{cells.length} zones</div>
  </div>
</div>

<style>
  .treemap-wrapper { width: 100%; height: 100%; position: relative; overflow: hidden; }
  canvas { display: block; cursor: crosshair; }
  .legend {
    position: absolute; top: 12px; left: 12px;
    background: rgba(8,12,20,0.90); border: 1px solid rgba(30,41,59,0.8);
    border-radius: 8px; padding: 10px 14px; backdrop-filter: blur(12px); pointer-events: none;
  }
  .legend-title { font-size: 10px; text-transform: uppercase; letter-spacing: 0.8px; color: var(--text-muted); margin-bottom: 8px; font-weight: 600; }
  .legend-gradient { width: 140px; height: 10px; border-radius: 5px; background: linear-gradient(to right, #0a1e3c, #0e5060, #327340, #b48220, #e05a0f, #c81e1e); }
  .legend-labels { display: flex; justify-content: space-between; font-size: 9px; color: var(--text-muted); margin-top: 4px; }
  .legend-count { font-size: 9px; color: var(--text-muted); margin-top: 8px; text-align: center; font-family: var(--font-mono); }
</style>
