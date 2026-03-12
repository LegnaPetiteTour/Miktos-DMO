// Types matching the Rust IPC structs in src-tauri/src/lib.rs

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
