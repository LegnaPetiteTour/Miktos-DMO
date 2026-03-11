<script lang="ts">
  import { onMount, tick } from "svelte";
  import { hierarchy } from "d3-hierarchy";
  import { voronoiTreemap as voronoiTreemapFn } from "d3-voronoi-treemap";
  import type { TreeNode } from "./types";

  let {
    tree,
    onHover,
    onClick,
  }: {
    tree: TreeNode;
    onHover: (node: TreeNode | null, x: number, y: number) => void;
    onClick: (node: TreeNode) => void;
  } = $props();

  let canvas: HTMLCanvasElement | undefined = $state(undefined);
  let container: HTMLDivElement | undefined = $state(undefined);
  let width = $state(800);
  let height = $state(600);
  let cells: RenderedCell[] = $state([]);
  let hoveredIndex: number = $state(-1);
  let layoutError: string = $state("");
  let mounted = $state(false);
  let renderMode: string = $state("");  // "voronoi" | "fallback" | empty

  interface RenderedCell {
    polygon: [number, number][];
    node: TreeNode;
    fillColor: string;
    glowColor: string;
    centroid: [number, number];
    area: number;
  }

  // ═══════════════════════════════════════════════════
  // COLOR SYSTEM — Score + Size hybrid
  // ═══════════════════════════════════════════════════

  // The visual intensity combines waste_score (how likely it's waste)
  // with a size boost (large caches are visually important even at low scores).
  // An 8GB cache at score 0.05 should look warmer than a 2MB file at 0.05.
  function visualIntensity(rawScore: number, sizeBytes: number): number {
    const scorePart = Math.pow(Math.min(rawScore * 2.5, 1.0), 0.55);

    // Size boost: log-scaled, kicks in above 100MB
    const sizeMB = sizeBytes / (1024 * 1024);
    const sizeBoost = sizeMB > 100 ? Math.min(Math.log10(sizeMB / 100) * 0.15, 0.25) : 0;

    return Math.min(scorePart + sizeBoost, 1.0);
  }

  function wasteColor(rawScore: number, sizeBytes: number): string {
    const s = visualIntensity(rawScore, sizeBytes);
    let r: number, g: number, b: number;

    if (s < 0.20) {
      const t = s / 0.20;
      r = lerp(10, 14, t); g = lerp(30, 80, t); b = lerp(60, 95, t);
    } else if (s < 0.40) {
      const t = (s - 0.20) / 0.20;
      r = lerp(14, 50, t); g = lerp(80, 115, t); b = lerp(95, 80, t);
    } else if (s < 0.60) {
      const t = (s - 0.40) / 0.20;
      r = lerp(50, 185, t); g = lerp(115, 125, t); b = lerp(80, 25, t);
    } else if (s < 0.80) {
      const t = (s - 0.60) / 0.20;
      r = lerp(185, 225, t); g = lerp(125, 80, t); b = lerp(25, 10, t);
    } else {
      const t = (s - 0.80) / 0.20;
      r = lerp(225, 200, t); g = lerp(80, 25, t); b = lerp(10, 25, t);
    }
    return `rgb(${Math.round(r)},${Math.round(g)},${Math.round(b)})`;
  }

  function wasteGlow(rawScore: number, sizeBytes: number): string {
    const s = visualIntensity(rawScore, sizeBytes);
    if (s > 0.6) return "rgba(239, 68, 68, 0.30)";
    if (s > 0.4) return "rgba(245, 158, 11, 0.22)";
    if (s > 0.2) return "rgba(20, 184, 166, 0.15)";
    return "rgba(56, 189, 248, 0.08)";
  }

  function borderAlpha(rawScore: number, sizeBytes: number): string {
    const s = visualIntensity(rawScore, sizeBytes);
    if (s > 0.6) return "rgba(239, 68, 68, 0.45)";
    if (s > 0.4) return "rgba(245, 158, 11, 0.30)";
    if (s > 0.2) return "rgba(100, 200, 200, 0.18)";
    return "rgba(56, 189, 248, 0.08)";
  }

  function lerp(a: number, b: number, t: number): number {
    return a + (b - a) * t;
  }

  // ═══════════════════════════════════════════════════
  // LAYOUT
  // ═══════════════════════════════════════════════════

  function computeLayout(treeData: TreeNode, w: number, h: number): RenderedCell[] {
    layoutError = "";

    if (!treeData) { layoutError = "No data"; return []; }

    const items = (treeData.children || []).filter(c => c.size > 0);
    if (items.length === 0) {
      layoutError = `No visible children (${treeData.children?.length ?? 0} total)`;
      return [];
    }

    console.log(`[DMO] Layout: ${items.length} items, ${w}x${h}`);

    // Build a FLAT 2-level hierarchy: root → items as leaves.
    // We intentionally strip item.children so Voronoi doesn't recurse into
    // thousands of individual files (which would make iteration never converge).
    // Originals are kept in a map so drill-down onClick still receives the full node.
    const originalMap = new Map(items.map(n => [n.path, n]));
    const flatItems: any[] = items.map(c => ({
      name: c.name, path: c.path, size: c.size,
      waste_score: c.waste_score, category: c.category,
      is_directory: c.is_directory, file_count: c.file_count,
      children: undefined,
    }));
    const wrapper = { name: treeData.name, path: treeData.path, children: flatItems };

    const root = hierarchy(wrapper, (d: any) => d.children?.length ? d.children : null)
      .sum((d: any) => (!d.children?.length) ? Math.max(d.size || 0, 1) : 0)
      .sort((a: any, b: any) => (b.value || 0) - (a.value || 0));

    if (!root.value) { layoutError = "Zero total"; return []; }

    const pad = 6;
    const clip: [number, number][] = [
      [pad, pad], [w - pad, pad], [w - pad, h - pad], [pad, h - pad],
    ];

    try {
      voronoiTreemapFn()
        .clip(clip)
        .minWeightRatio(0.002)
        .maxIterationCount(300)
        .convergenceRatio(0.005)(root);

      const result = extractCells(root, originalMap);
      if (result.length > 0) {
        const avgPts = Math.round(result.reduce((s, c) => s + c.polygon.length, 0) / result.length);
        console.log(`[DMO] Voronoi OK: ${result.length} cells, avg ${avgPts} pts/cell`);
        renderMode = `voronoi (${avgPts}pts)`;
        return result;
      }
      console.warn("[DMO] Voronoi produced 0 cells — falling back");
    } catch (e) {
      console.warn("[DMO] Voronoi error, using fallback:", e);
    }

    renderMode = "fallback";
    return fallbackGrid(items, w, h, pad);
  }

  function extractCells(root: any, originalMap?: Map<string, TreeNode>): RenderedCell[] {
    const result: RenderedCell[] = [];
    for (const leaf of root.leaves()) {
      const poly = leaf.polygon;
      if (!poly || poly.length < 3) continue;

      const flat = leaf.data as any;
      // Restore original node so drill-down gets the full children list
      const node: TreeNode = originalMap?.get(flat.path) ?? flat;
      const points: [number, number][] = poly.map((p: any) => [p[0], p[1]]);

      let cx = 0, cy = 0;
      for (const [x, y] of points) { cx += x; cy += y; }
      cx /= points.length; cy /= points.length;

      result.push({
        polygon: points,
        node,
        fillColor: wasteColor(node.waste_score, node.size),
        glowColor: wasteGlow(node.waste_score, node.size),
        centroid: [cx, cy],
        area: polygonArea(points),
      });
    }
    return result;
  }

  function fallbackGrid(children: TreeNode[], w: number, h: number, pad: number): RenderedCell[] {
    const totalSize = children.reduce((s, c) => s + c.size, 0);
    if (totalSize === 0) return [];

    const result: RenderedCell[] = [];
    const usableW = w - pad * 2;
    const usableH = h - pad * 2;
    const cols = Math.ceil(Math.sqrt(children.length));
    const cellW = usableW / cols;
    let x = pad, y = pad, rowH = 0;

    for (const child of children) {
      const frac = child.size / totalSize;
      const h2 = Math.max(usableH * frac * cols, 20);

      if (x + cellW > w - pad + 1) { x = pad; y += rowH + 2; rowH = 0; }

      const x2 = x + cellW - 2, y2 = y + h2;
      rowH = Math.max(rowH, h2);

      const polygon: [number, number][] = [[x, y], [x2, y], [x2, y2], [x, y2]];
      result.push({
        polygon,
        node: child,
        fillColor: wasteColor(child.waste_score, child.size),
        glowColor: wasteGlow(child.waste_score, child.size),
        centroid: [(x + x2) / 2, (y + y2) / 2],
        area: (x2 - x) * (y2 - y),
      });
      x += cellW;
    }
    return result;
  }

  // ═══════════════════════════════════════════════════
  // RENDER
  // ═══════════════════════════════════════════════════

  function render() {
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    ctx.scale(dpr, dpr);

    ctx.fillStyle = "#080c14";
    ctx.fillRect(0, 0, width, height);

    if (cells.length === 0) {
      ctx.fillStyle = "rgba(255,255,255,0.25)";
      ctx.font = "13px monospace";
      ctx.textAlign = "center";
      ctx.fillText(layoutError || "No data to render", width / 2, height / 2);
      return;
    }

    for (let i = 0; i < cells.length; i++) {
      const cell = cells[i];
      if (cell.polygon.length < 3) continue;
      const isHovered = i === hoveredIndex;

      ctx.beginPath();
      ctx.moveTo(cell.polygon[0][0], cell.polygon[0][1]);
      for (let j = 1; j < cell.polygon.length; j++) {
        ctx.lineTo(cell.polygon[j][0], cell.polygon[j][1]);
      }
      ctx.closePath();

      // Fill
      ctx.fillStyle = cell.fillColor;
      ctx.fill();

      // Glow overlay for waste zones
      if (!isHovered && cell.node.waste_score > 0.02) {
        ctx.fillStyle = cell.glowColor;
        ctx.fill();
      }

      // Hover
      if (isHovered) {
        ctx.fillStyle = "rgba(255, 255, 255, 0.10)";
        ctx.fill();
        ctx.strokeStyle = "rgba(255, 255, 255, 0.5)";
        ctx.lineWidth = 2.5;
        ctx.stroke();
      } else {
        ctx.strokeStyle = borderAlpha(cell.node.waste_score, cell.node.size);
        ctx.lineWidth = 0.6;
        ctx.stroke();
      }

      // Directory indicator: thin inner border
      if (cell.node.is_directory && cell.node.children?.length > 0 && cell.area > 3000) {
        ctx.strokeStyle = "rgba(56, 189, 248, 0.12)";
        ctx.lineWidth = 0.4;
        ctx.setLineDash([3, 3]);
        ctx.stroke();
        ctx.setLineDash([]);
      }

      // Labels
      if (cell.area > 1200) {
        const maxDim = Math.sqrt(cell.area);
        const fontSize = Math.max(8, Math.min(14, maxDim / 7));

        ctx.font = `500 ${fontSize}px "SF Mono", "Fira Code", monospace`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";

        let displayLabel = cell.node.name;
        const maxWidth = maxDim * 0.75;
        while (ctx.measureText(displayLabel).width > maxWidth && displayLabel.length > 4) {
          displayLabel = displayLabel.slice(0, -2) + "\u2026";
        }

        // Shadow
        ctx.fillStyle = "rgba(0, 0, 0, 0.75)";
        ctx.fillText(displayLabel, cell.centroid[0] + 1, cell.centroid[1] + 1);

        // Text
        const alpha = isHovered ? 1.0 : (visualIntensity(cell.node.waste_score, cell.node.size) > 0.3 ? 0.9 : 0.6);
        ctx.fillStyle = `rgba(255, 255, 255, ${alpha})`;
        ctx.fillText(displayLabel, cell.centroid[0], cell.centroid[1]);

        // Size sublabel
        if (cell.area > 6000) {
          const sizeStr = fmtBytes(cell.node.size);
          const smallFont = Math.max(7, fontSize - 3);
          ctx.font = `400 ${smallFont}px "SF Mono", monospace`;
          ctx.fillStyle = `rgba(255, 255, 255, ${isHovered ? 0.65 : 0.28})`;
          ctx.fillText(sizeStr, cell.centroid[0], cell.centroid[1] + fontSize);
        }
      }
    }
  }

  function fmtBytes(b: number): string {
    if (b >= 1024 ** 3) return `${(b / 1024 ** 3).toFixed(1)}G`;
    if (b >= 1024 ** 2) return `${(b / 1024 ** 2).toFixed(0)}M`;
    if (b >= 1024) return `${(b / 1024).toFixed(0)}K`;
    return `${b}B`;
  }

  function polygonArea(pts: [number, number][]): number {
    let a = 0;
    for (let i = 0, n = pts.length; i < n; i++) {
      const j = (i + 1) % n;
      a += pts[i][0] * pts[j][1] - pts[j][0] * pts[i][1];
    }
    return Math.abs(a / 2);
  }

  // ═══════════════════════════════════════════════════
  // HIT DETECTION & EVENTS
  // ═══════════════════════════════════════════════════

  function pointInPoly(x: number, y: number, poly: [number, number][]): boolean {
    let inside = false;
    for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
      const [xi, yi] = poly[i], [xj, yj] = poly[j];
      if ((yi > y) !== (yj > y) && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi)
        inside = !inside;
    }
    return inside;
  }

  function handleMouseMove(e: MouseEvent) {
    if (!canvas || cells.length === 0) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left, y = e.clientY - rect.top;

    let idx = -1;
    for (let i = 0; i < cells.length; i++) {
      if (pointInPoly(x, y, cells[i].polygon)) { idx = i; break; }
    }

    if (idx !== hoveredIndex) { hoveredIndex = idx; render(); }
    onHover(idx >= 0 ? cells[idx].node : null, e.clientX, e.clientY);
  }

  function handleMouseLeave() {
    if (hoveredIndex !== -1) { hoveredIndex = -1; render(); }
    onHover(null, 0, 0);
  }

  function handleClick(e: MouseEvent) {
    if (!canvas || cells.length === 0) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left, y = e.clientY - rect.top;

    for (let i = 0; i < cells.length; i++) {
      if (pointInPoly(x, y, cells[i].polygon)) {
        onClick(cells[i].node);
        return;
      }
    }
  }

  // ═══════════════════════════════════════════════════
  // LIFECYCLE
  // ═══════════════════════════════════════════════════

  function updateSize() {
    if (!container) return;
    const rect = container.getBoundingClientRect();
    const nw = Math.floor(rect.width), nh = Math.floor(rect.height);
    if (nw > 0 && nh > 0 && (nw !== width || nh !== height)) {
      width = nw; height = nh;
    }
  }

  onMount(() => {
    updateSize();
    mounted = true;

    const obs = new ResizeObserver(() => {
      updateSize();
      if (tree && width > 0 && height > 0) {
        cells = computeLayout(tree, width, height);
        tick().then(() => render());
      }
    });
    if (container) obs.observe(container);
    return () => obs.disconnect();
  });

  $effect(() => {
    if (!mounted) return;
    const _t = tree; const _w = width; const _h = height;
    if (_t && _w > 0 && _h > 0) {
      cells = computeLayout(_t, _w, _h);
      tick().then(() => render());
    }
  });
</script>

<div class="treemap-wrapper no-select" bind:this={container}>
  <canvas
    bind:this={canvas}
    style="width: {width}px; height: {height}px;"
    onmousemove={handleMouseMove}
    onmouseleave={handleMouseLeave}
    onclick={handleClick}
  ></canvas>

  <div class="legend">
    <div class="legend-title">Terrain Health</div>
    <div class="legend-gradient"></div>
    <div class="legend-labels">
      <span>Clean</span>
      <span>Moderate</span>
      <span>Critical</span>
    </div>
    <div class="legend-count">{cells.length} zones{renderMode ? ` · ${renderMode}` : ""}</div>
  </div>
</div>

<style>
  .treemap-wrapper { width: 100%; height: 100%; position: relative; overflow: hidden; }
  canvas { display: block; cursor: crosshair; }
  .legend {
    position: absolute; top: 12px; right: 12px;
    background: rgba(8, 12, 20, 0.90); border: 1px solid rgba(30, 41, 59, 0.8);
    border-radius: 8px; padding: 10px 14px; backdrop-filter: blur(12px); pointer-events: none;
  }
  .legend-title { font-size: 10px; text-transform: uppercase; letter-spacing: 0.8px; color: var(--text-muted); margin-bottom: 8px; font-weight: 600; }
  .legend-gradient { width: 140px; height: 10px; border-radius: 5px; background: linear-gradient(to right, #0a1e3c, #0e5060, #327340, #b48220, #e05a0f, #c81e1e); }
  .legend-labels { display: flex; justify-content: space-between; font-size: 9px; color: var(--text-muted); margin-top: 4px; }
  .legend-count { font-size: 9px; color: var(--text-muted); margin-top: 8px; text-align: center; font-family: var(--font-mono); }
</style>
