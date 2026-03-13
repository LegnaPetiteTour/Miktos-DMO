import type { TerrainCell } from "../types";

/**
 * Rasterizes terrain cell waste scores into a flat Float32Array
 * suitable for upload as a WebGL2 RGBA32F texture (R channel = concentration).
 *
 * The polygon coordinates are in CSS-pixel canvas space (0..width, 0..height),
 * matching the coordinate system used by Treemap.svelte's Voronoi layout.
 * The overlay WebGL canvas must have the same CSS-pixel dimensions.
 *
 * @param cells   Terrain cells emitted by Treemap.svelte via onCells()
 * @param width   Canvas CSS-pixel width  (== overlayCanvas.width)
 * @param height  Canvas CSS-pixel height (== overlayCanvas.height)
 * @returns       Float32Array of length width * height; each value in [0, 1]
 */
export function buildChemoattractantTexture(
  cells: TerrainCell[],
  width: number,
  height: number,
): Float32Array {
  const data = new Float32Array(width * height); // all zeros

  for (const cell of cells) {
    if (cell.polygon.length < 3) continue;
    const score = cell.waste_score;
    if (score <= 0) continue;

    // Bounding box of polygon (clamped to texture bounds)
    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    for (const [x, y] of cell.polygon) {
      if (x < minX) minX = x;
      if (x > maxX) maxX = x;
      if (y < minY) minY = y;
      if (y > maxY) maxY = y;
    }

    const x0 = Math.max(0, Math.floor(minX));
    const x1 = Math.min(width - 1, Math.ceil(maxX));
    const y0 = Math.max(0, Math.floor(minY));
    const y1 = Math.min(height - 1, Math.ceil(maxY));

    // Point-in-polygon test for each pixel in bounding box
    for (let py = y0; py <= y1; py++) {
      for (let px = x0; px <= x1; px++) {
        if (pip(px + 0.5, py + 0.5, cell.polygon)) {
          const idx = py * width + px;
          // Take the maximum when cells overlap (boundary rounding)
          if (score > data[idx]) data[idx] = score;
        }
      }
    }
  }

  return data;
}

/** Ray-casting point-in-polygon test. */
function pip(x: number, y: number, poly: [number, number][]): boolean {
  let inside = false;
  const n = poly.length;
  for (let i = 0, j = n - 1; i < n; j = i++) {
    const [xi, yi] = poly[i];
    const [xj, yj] = poly[j];
    if ((yi > y) !== (yj > y) && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) {
      inside = !inside;
    }
  }
  return inside;
}
