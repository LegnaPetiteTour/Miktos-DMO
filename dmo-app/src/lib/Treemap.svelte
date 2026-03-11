<script lang="ts">
  import { onMount } from "svelte";
  import { hierarchy } from "d3-hierarchy";
  import { voronoiTreemap } from "d3-voronoi-treemap";
  import type { TreeNode } from "./types";

  // ─── Props ───
  let {
    tree,
    onHover,
    onClick,
  }: {
    tree: TreeNode;
    onHover: (node: TreeNode | null, x: number, y: number) => void;
    onClick: (node: TreeNode) => void;
  } = $props();

  // ─── State ───
  let canvas: HTMLCanvasElement | undefined = $state(undefined);
  let container: HTMLDivElement | undefined = $state(undefined);
  let width = $state(800);
  let height = $state(600);
  let cells: RenderedCell[] = $state([]);
  let hoveredIndex: number = $state(-1);
  let prevHoveredIndex: number = $state(-1);

  interface RenderedCell {
    polygon: [number, number][];
    node: TreeNode;
    fillColor: string;
    glowColor: string;
    centroid: [number, number];
    area: number;
  }

  // ═══════════════════════════════════════════════════
  // COLOR SYSTEM — Stretched for visual clarity
  // ═══════════════════════════════════════════════════

  // The waste_score formula produces small numbers (product of four 0–1 values).
  // Real-world scores cluster in 0.0–0.3 range. We need a perceptual curve
  // that makes differences in this range visible.
  function perceptualScore(raw: number): number {
    // Power curve: sqrt stretches the low end
    // Then scale so 0.3 raw → ~0.7 perceptual
    return Math.pow(Math.min(raw * 2.5, 1.0), 0.55);
  }

  function wasteColor(rawScore: number): string {
    const s = perceptualScore(rawScore);

    // 5-stop gradient:
    // 0.00 → deep ocean blue      rgb(12, 35, 68)
    // 0.25 → cool teal             rgb(15, 90, 100)
    // 0.50 → warm amber            rgb(180, 130, 30)
    // 0.75 → hot orange            rgb(220, 90, 15)
    // 1.00 → critical red          rgb(200, 30, 30)

    let r: number, g: number, b: number;

    if (s < 0.25) {
      const t = s / 0.25;
      r = lerp(12, 15, t);
      g = lerp(35, 90, t);
      b = lerp(68, 100, t);
    } else if (s < 0.50) {
      const t = (s - 0.25) / 0.25;
      r = lerp(15, 180, t);
      g = lerp(90, 130, t);
      b = lerp(100, 30, t);
    } else if (s < 0.75) {
      const t = (s - 0.50) / 0.25;
      r = lerp(180, 220, t);
      g = lerp(130, 90, t);
      b = lerp(30, 15, t);
    } else {
      const t = (s - 0.75) / 0.25;
      r = lerp(220, 200, t);
      g = lerp(90, 30, t);
      b = lerp(15, 30, t);
    }

    return `rgb(${Math.round(r)},${Math.round(g)},${Math.round(b)})`;
  }

  function wasteGlow(rawScore: number): string {
    const s = perceptualScore(rawScore);
    if (s > 0.6) return "rgba(239, 68, 68, 0.35)";
    if (s > 0.4) return "rgba(245, 158, 11, 0.30)";
    if (s > 0.2) return "rgba(20, 184, 166, 0.20)";
    return "rgba(56, 189, 248, 0.12)";
  }

  function borderAlpha(rawScore: number): string {
    const s = perceptualScore(rawScore);
    if (s > 0.6) return "rgba(239, 68, 68, 0.50)";
    if (s > 0.4) return "rgba(245, 158, 11, 0.35)";
    if (s > 0.2) return "rgba(100, 200, 200, 0.20)";
    return "rgba(56, 189, 248, 0.10)";
  }

  function lerp(a: number, b: number, t: number): number {
    return a + (b - a) * t;
  }

  // ═══════════════════════════════════════════════════
  // LAYOUT
  // ═══════════════════════════════════════════════════

  function computeLayout(treeData: TreeNode, w: number, h: number): RenderedCell[] {
    if (!treeData.children || treeData.children.length === 0) return [];

    const root = hierarchy(treeData, (d: any) => d.children)
      .sum((d: any) => (d.children && d.children.length > 0 ? 0 : d.size))
      .sort((a: any, b: any) => (b.value || 0) - (a.value || 0));

    const pad = 6;
    const clip: [number, number][] = [
      [pad, pad],
      [w - pad, pad],
      [w - pad, h - pad],
      [pad, h - pad],
    ];

    try {
      const layout = voronoiTreemap()
        .clip(clip)
        .minWeightRatio(0.005)
        .maxIterationCount(100);
      layout(root);
    } catch (e) {
      console.warn("Voronoi layout error:", e);
      return [];
    }

    const result: RenderedCell[] = [];
    const leaves = root.leaves();

    for (const leaf of leaves) {
      const poly = (leaf as any).polygon;
      if (!poly || poly.length < 3) continue;

      const data = leaf.data as TreeNode;
      const points: [number, number][] = poly.map((p: number[]) => [p[0], p[1]]);

      let cx = 0, cy = 0;
      for (const [x, y] of points) { cx += x; cy += y; }
      cx /= points.length;
      cy /= points.length;

      const area = polygonArea(points);

      result.push({
        polygon: points,
        node: data,
        fillColor: wasteColor(data.waste_score),
        glowColor: wasteGlow(data.waste_score),
        centroid: [cx, cy],
        area,
      });
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

    // Background
    ctx.fillStyle = "#080c14";
    ctx.fillRect(0, 0, width, height);

    // Draw cells
    for (let i = 0; i < cells.length; i++) {
      const cell = cells[i];
      if (cell.polygon.length < 3) continue;

      const isHovered = i === hoveredIndex;

      // ── Fill ──
      ctx.beginPath();
      ctx.moveTo(cell.polygon[0][0], cell.polygon[0][1]);
      for (let j = 1; j < cell.polygon.length; j++) {
        ctx.lineTo(cell.polygon[j][0], cell.polygon[j][1]);
      }
      ctx.closePath();

      ctx.fillStyle = cell.fillColor;
      ctx.fill();

      // ── Hover glow overlay ──
      if (isHovered) {
        ctx.fillStyle = "rgba(255, 255, 255, 0.08)";
        ctx.fill();
        ctx.strokeStyle = "rgba(255, 255, 255, 0.45)";
        ctx.lineWidth = 2.0;
        ctx.stroke();
      } else {
        // Normal border
        ctx.strokeStyle = borderAlpha(cell.node.waste_score);
        ctx.lineWidth = 0.7;
        ctx.stroke();
      }

      // ── Subtle inner glow for high-waste zones ──
      if (cell.node.waste_score > 0.08 && !isHovered) {
        ctx.fillStyle = cell.glowColor;
        ctx.fill();
      }

      // ── Label ──
      if (cell.area > 1500) {
        const label = cell.node.name;
        const maxDim = Math.sqrt(cell.area);
        const fontSize = Math.max(8, Math.min(14, maxDim / 7));

        ctx.font = `500 ${fontSize}px "SF Mono", "Fira Code", "Cascadia Code", monospace`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";

        const maxWidth = maxDim * 0.75;
        let displayLabel = label;
        while (ctx.measureText(displayLabel).width > maxWidth && displayLabel.length > 4) {
          displayLabel = displayLabel.slice(0, -2) + "\u2026";
        }

        // Text shadow for readability
        ctx.fillStyle = "rgba(0, 0, 0, 0.7)";
        ctx.fillText(displayLabel, cell.centroid[0] + 0.8, cell.centroid[1] + 0.8);

        // Text
        const textAlpha = isHovered ? 1.0 : (cell.node.waste_score > 0.15 ? 0.85 : 0.55);
        ctx.fillStyle = `rgba(255, 255, 255, ${textAlpha})`;
        ctx.fillText(displayLabel, cell.centroid[0], cell.centroid[1]);

        // Size label below name for larger cells
        if (cell.area > 8000) {
          const sizeStr = formatBytesShort(cell.node.size);
          const smallSize = Math.max(7, fontSize - 3);
          ctx.font = `400 ${smallSize}px "SF Mono", monospace`;
          ctx.fillStyle = `rgba(255, 255, 255, ${isHovered ? 0.7 : 0.3})`;
          ctx.fillText(sizeStr, cell.centroid[0], cell.centroid[1] + fontSize * 0.9);
        }
      }
    }
  }

  function formatBytesShort(bytes: number): string {
    if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(1)}G`;
    if (bytes >= 1024 ** 2) return `${(bytes / 1024 ** 2).toFixed(0)}M`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)}K`;
    return `${bytes}B`;
  }

  function polygonArea(points: [number, number][]): number {
    let area = 0;
    const n = points.length;
    for (let i = 0; i < n; i++) {
      const j = (i + 1) % n;
      area += points[i][0] * points[j][1];
      area -= points[j][0] * points[i][1];
    }
    return Math.abs(area / 2);
  }

  // ═══════════════════════════════════════════════════
  // HIT DETECTION
  // ═══════════════════════════════════════════════════

  function pointInPolygon(x: number, y: number, poly: [number, number][]): boolean {
    let inside = false;
    for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
      const xi = poly[i][0], yi = poly[i][1];
      const xj = poly[j][0], yj = poly[j][1];
      if ((yi > y) !== (yj > y) && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) {
        inside = !inside;
      }
    }
    return inside;
  }

  function findCellAt(x: number, y: number): number {
    for (let i = 0; i < cells.length; i++) {
      if (pointInPolygon(x, y, cells[i].polygon)) return i;
    }
    return -1;
  }

  function handleMouseMove(e: MouseEvent) {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const idx = findCellAt(x, y);
    hoveredIndex = idx;

    if (idx >= 0) {
      onHover(cells[idx].node, e.clientX, e.clientY);
    } else {
      onHover(null, 0, 0);
    }
  }

  function handleMouseLeave() {
    hoveredIndex = -1;
    onHover(null, 0, 0);
  }

  function handleClick(e: MouseEvent) {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const idx = findCellAt(x, y);
    if (idx >= 0) {
      onClick(cells[idx].node);
    }
  }

  // ═══════════════════════════════════════════════════
  // RESIZE
  // ═══════════════════════════════════════════════════

  function updateSize() {
    if (!container) return;
    const rect = container.getBoundingClientRect();
    width = Math.floor(rect.width);
    height = Math.floor(rect.height);
  }

  onMount(() => {
    updateSize();
    const observer = new ResizeObserver(() => updateSize());
    if (container) observer.observe(container);
    return () => observer.disconnect();
  });

  // ─── Reactivity ───
  $effect(() => {
    if (tree && width > 0 && height > 0) {
      cells = computeLayout(tree, width, height);
    }
  });

  // Re-render on cells change or hover change
  $effect(() => {
    // Access both to track them
    const _cells = cells;
    const _hover = hoveredIndex;
    if (_cells.length > 0 && canvas) {
      render();
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

  <!-- Legend -->
  <div class="legend">
    <div class="legend-title">Terrain Health</div>
    <div class="legend-bar">
      <div class="legend-gradient"></div>
      <div class="legend-labels">
        <span>Clean</span>
        <span>Moderate</span>
        <span>Critical</span>
      </div>
    </div>
    <div class="legend-count">{cells.length} zones</div>
  </div>
</div>

<style>
  .treemap-wrapper {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
  }
  canvas {
    display: block;
    cursor: crosshair;
  }

  .legend {
    position: absolute;
    top: 12px;
    right: 12px;
    background: rgba(8, 12, 20, 0.88);
    border: 1px solid rgba(30, 41, 59, 0.8);
    border-radius: 8px;
    padding: 10px 14px;
    backdrop-filter: blur(12px);
    pointer-events: none;
  }
  .legend-title {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--text-muted);
    margin-bottom: 8px;
    font-weight: 600;
  }
  .legend-gradient {
    width: 140px;
    height: 10px;
    border-radius: 5px;
    background: linear-gradient(to right, #0c2344, #0f5a64, #b48220, #dc5a0f, #c81e1e);
  }
  .legend-labels {
    display: flex;
    justify-content: space-between;
    font-size: 9px;
    color: var(--text-muted);
    margin-top: 4px;
  }
  .legend-count {
    font-size: 9px;
    color: var(--text-muted);
    margin-top: 8px;
    text-align: center;
    font-family: var(--font-mono);
  }
</style>
