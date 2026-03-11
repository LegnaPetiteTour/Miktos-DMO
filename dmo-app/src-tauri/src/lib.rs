use dmo_core::pipeline;
use dmo_core::types::ScanConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─── App entry point (called by main.rs) ───
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_filesystem,
            commands::get_home_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running DMO application");
}

// ─── Tauri Commands ───
// Must live in a submodule to avoid #[macro_export] collision in crate root.
pub mod commands {
    use super::*;

    #[tauri::command]
    pub fn scan_filesystem(path: String, max_depth: usize) -> Result<ScanResult, String> {
        let root = PathBuf::from(&path);
        if !root.exists() {
            return Err(format!("Path does not exist: {}", path));
        }

        let config = ScanConfig {
            root: root.clone(),
            max_depth: Some(max_depth),
            follow_symlinks: false,
            denylist: default_denylist(),
        };

        let (nodes, summary) = pipeline::run(&config).map_err(|e| e.to_string())?;

        let tree = build_tree(&root, &nodes);

        let categories = summary
            .categories
            .iter()
            .map(|c| CategoryDto {
                name: c.category.label().to_string(),
                file_count: c.file_count,
                total_bytes: c.total_bytes,
                avg_score: c.avg_waste_score,
            })
            .collect();

        Ok(ScanResult {
            tree,
            summary: ScanSummaryDto {
                root: path,
                total_files: summary.total_files,
                total_size: summary.total_size_bytes,
                waste_candidates: summary.waste_candidates,
                waste_size: summary.waste_size_bytes,
                scan_time_ms: summary.scan_duration_ms,
                categories,
            },
        })
    }

    #[tauri::command]
    pub fn get_home_dir() -> Result<String, String> {
        std::env::var("HOME").map_err(|_| "Could not determine home directory".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub waste_score: f64,
    pub category: String,
    pub is_directory: bool,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub tree: TreeNode,
    pub summary: ScanSummaryDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummaryDto {
    pub root: String,
    pub total_files: usize,
    pub total_size: u64,
    pub waste_candidates: usize,
    pub waste_size: u64,
    pub scan_time_ms: u64,
    pub categories: Vec<CategoryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryDto {
    pub name: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub avg_score: f64,
}

// ─── IPC Types ───
// These are the data structures passed between Rust and the Svelte frontend.
// They must be serializable and focused on what the visualization needs.
// Converts the flat list of scored FileNodes into a hierarchical tree
// suitable for the Voronoi treemap visualization.

fn build_tree(root: &PathBuf, nodes: &[dmo_core::FileNode]) -> TreeNode {
    use std::collections::HashMap;

    let root_str = root.to_string_lossy().to_string();
    let mut dir_map: HashMap<String, Vec<TreeNode>> = HashMap::new();

    // Group files by their parent directory
    for node in nodes {
        if node.category.is_protected() {
            continue; // Protected files are invisible to the organism
        }

        let parent = node
            .path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let leaf = TreeNode {
            name: node.name.clone(),
            path: node.path.to_string_lossy().to_string(),
            size: node.size_bytes,
            waste_score: node.waste_score,
            category: node.category.label().to_string(),
            is_directory: false,
            children: vec![],
        };

        dir_map.entry(parent).or_default().push(leaf);
    }

    // Aggregate files into directory-level nodes for the treemap.
    // Each directory becomes a node whose size is the sum of its children,
    // and whose waste_score is the weighted average.
    let mut aggregated: HashMap<String, TreeNode> = HashMap::new();

    for (dir_path, files) in &dir_map {
        let total_size: u64 = files.iter().map(|f| f.size).sum();
        let weighted_score: f64 = if total_size > 0 {
            files.iter().map(|f| f.waste_score * f.size as f64).sum::<f64>() / total_size as f64
        } else {
            0.0
        };

        let dir_name = PathBuf::from(dir_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| dir_path.clone());

        // Determine the dominant category for this directory
        let mut cat_sizes: HashMap<String, u64> = HashMap::new();
        for f in files {
            *cat_sizes.entry(f.category.clone()).or_insert(0) += f.size;
        }
        let dominant_cat = cat_sizes
            .into_iter()
            .max_by_key(|(_, s)| *s)
            .map(|(c, _)| c)
            .unwrap_or_else(|| "Unknown".to_string());

        aggregated.insert(
            dir_path.clone(),
            TreeNode {
                name: dir_name,
                path: dir_path.clone(),
                size: total_size,
                waste_score: weighted_score,
                category: dominant_cat,
                is_directory: true,
                children: vec![], // Will be populated below
            },
        );
    }

    // Build the top-level tree: find directories that are direct children of root
    // For treemap visualization, we use a flat-ish structure (depth 1 directories)
    // to avoid deeply nested Voronoi which performs poorly.
    let mut top_children: Vec<TreeNode> = Vec::new();

    for (dir_path, node) in &aggregated {
        // Check if this directory is a "top-level" relative to root
        let rel = dir_path.strip_prefix(&root_str).unwrap_or(dir_path);
        let rel_path = PathBuf::from(rel);
        let depth = rel_path.components().count();

        if depth <= 2 {
            // Include as a direct child of the root
            top_children.push(node.clone());
        } else {
            // Find the nearest top-level ancestor and add this node's size to it
            // (aggregation for deeply nested directories)
            let components: Vec<_> = rel_path.components().take(2).collect();
            if components.len() >= 2 {
                let ancestor = format!(
                    "{}/{}",
                    root_str,
                    components
                        .iter()
                        .map(|c| c.as_os_str().to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join("/")
                );
                if let Some(parent_node) = top_children.iter_mut().find(|n| n.path == ancestor) {
                    parent_node.size += node.size;
                    // Update weighted score
                    let total = parent_node.size;
                    if total > 0 {
                        parent_node.waste_score = (parent_node.waste_score
                            * (total - node.size) as f64
                            + node.waste_score * node.size as f64)
                            / total as f64;
                    }
                }
            }
        }
    }

    // Sort by size descending for better treemap layout
    top_children.sort_by(|a, b| b.size.cmp(&a.size));

    // Filter out tiny directories (< 1MB) to reduce visual noise
    let min_size = 1024 * 1024; // 1MB
    top_children.retain(|n| n.size >= min_size);

    let total_size: u64 = top_children.iter().map(|n| n.size).sum();
    let avg_score: f64 = if total_size > 0 {
        top_children
            .iter()
            .map(|n| n.waste_score * n.size as f64)
            .sum::<f64>()
            / total_size as f64
    } else {
        0.0
    };

    let root_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root_str.clone());

    TreeNode {
        name: root_name,
        path: root_str,
        size: total_size,
        waste_score: avg_score,
        category: "Root".to_string(),
        is_directory: true,
        children: top_children,
    }
}

/// Default denylist for macOS.
fn default_denylist() -> Vec<PathBuf> {
    let mut list = vec![
        PathBuf::from("/System"),
        PathBuf::from("/usr"),
        PathBuf::from("/bin"),
        PathBuf::from("/sbin"),
        PathBuf::from("/Applications"),
    ];

    if let Ok(home) = std::env::var("HOME") {
        let h = PathBuf::from(&home);
        list.push(h.join("Documents"));
        list.push(h.join("Desktop"));
        list.push(h.join("Pictures"));
        list.push(h.join("Music"));
        list.push(h.join("Movies"));
        list.push(h.join("Applications"));
        list.push(h.join("Library/Keychains"));
        list.push(h.join(".ssh"));
        list.push(h.join(".gnupg"));
    }

    list
}
