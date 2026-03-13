// Types matching the Rust IPC structs in src-tauri/src/lib.rs

// A single cell of the Voronoi terrain layout.
// Exported so the Phase 2 Physarum simulation can read the filesystem→screen mapping.
export interface TerrainCell {
  /** Polygon vertices in canvas coordinates */
  polygon: [number, number][];
  /** Absolute filesystem path of the node this cell represents */
  path: string;
  /** waste_score [0, 1] — used as chemoattractant concentration in Phase 2 */
  waste_score: number;
  /** Centroid of the polygon in canvas coordinates */
  centroid: [number, number];
  /** Pixel area of the polygon (for label / importance thresholds) */
  area: number;
}

export interface TreeNode {
  name: string;
  path: string;
  size: number;
  file_count: number;
  waste_score: number;
  category: string;
  is_directory: boolean;
  children: TreeNode[];
}

export interface ScanResult {
  /** Row ID in ~/.dmo/history.db; -1 if persistence failed */
  scan_id: number;
  tree: TreeNode;
  summary: ScanSummary;
}

export interface ScanSummary {
  root: string;
  total_files: number;
  total_size: number;
  waste_candidates: number;
  waste_size: number;
  scan_time_ms: number;
  categories: CategoryInfo[];
}

export interface CategoryInfo {
  name: string;
  file_count: number;
  total_bytes: number;
  avg_score: number;
}
