use dmo_core::pipeline;
use dmo_core::types::ScanConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

        log::info!(
            "Tree built: {} top-level children, total size {}",
            tree.children.len(),
            tree.size
        );

        let categories: Vec<CategoryDto> = summary
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

// ═══════════════════════════════════════════════════
// IPC Types
// ═══════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub file_count: usize,
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

// ═══════════════════════════════════════════════════
// TREE BUILDER — Simple recursive grouping
// ═══════════════════════════════════════════════════
//
// Strategy: For each file, compute its path relative to the scan root.
// Take the first component of that relative path as the "group key".
// Files sharing the same first component become children of a directory node.
// Recurse: within each group, strip the first component and repeat.
//
// This produces a proper nested tree at any depth.

fn build_tree(root: &Path, nodes: &[dmo_core::FileNode]) -> TreeNode {
    let root_str = root.to_string_lossy().to_string();

    // Collect non-protected files as simple structs
    let mut files: Vec<FileEntry> = Vec::new();
    for node in nodes {
        if node.category.is_protected() {
            continue;
        }
        // Compute path relative to root
        let full = node.path.to_string_lossy().to_string();
        let rel = full.strip_prefix(&root_str)
            .unwrap_or(&full)
            .trim_start_matches('/');

        if rel.is_empty() {
            continue;
        }

        files.push(FileEntry {
            rel_path: rel.to_string(),
            full_path: full,
            name: node.name.clone(),
            size: node.size_bytes,
            waste_score: node.waste_score,
            category: node.category.label().to_string(),
        });
    }

    log::info!("Building tree from {} non-protected files", files.len());

    // Build recursive tree
    let children = group_into_tree(&files, &root_str);

    // Compute root aggregates
    let total_size: u64 = children.iter().map(|c| c.size).sum();
    let total_files: usize = children.iter().map(|c| c.file_count).sum();
    let avg_score = if total_size > 0 {
        children.iter().map(|c| c.waste_score * c.size as f64).sum::<f64>() / total_size as f64
    } else {
        0.0
    };

    let root_name = root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root_str.clone());

    TreeNode {
        name: root_name,
        path: root_str,
        size: total_size,
        file_count: total_files,
        waste_score: avg_score,
        category: "Root".to_string(),
        is_directory: true,
        children,
    }
}

struct FileEntry {
    rel_path: String,    // path relative to current grouping root
    full_path: String,   // absolute path
    name: String,
    size: u64,
    waste_score: f64,
    category: String,
}

fn group_into_tree(files: &[FileEntry], parent_path: &str) -> Vec<TreeNode> {
    // Group by first path component
    let mut groups: HashMap<String, Vec<&FileEntry>> = HashMap::new();
    let mut direct_files: Vec<&FileEntry> = Vec::new();

    for f in files {
        if let Some(slash_pos) = f.rel_path.find('/') {
            let first_component = &f.rel_path[..slash_pos];
            groups.entry(first_component.to_string()).or_default().push(f);
        } else {
            // This file is directly in the current directory
            direct_files.push(f);
        }
    }

    let mut result: Vec<TreeNode> = Vec::new();

    // Create directory nodes for each group
    for (dir_name, group_files) in &groups {
        let dir_path = format!("{}/{}", parent_path, dir_name);

        // Recurse: strip the first component from each file's rel_path
        let sub_files: Vec<FileEntry> = group_files.iter().map(|f| {
            let slash_pos = f.rel_path.find('/').unwrap();
            FileEntry {
                rel_path: f.rel_path[slash_pos + 1..].to_string(),
                full_path: f.full_path.clone(),
                name: f.name.clone(),
                size: f.size,
                waste_score: f.waste_score,
                category: f.category.clone(),
            }
        }).collect();

        let children = group_into_tree(&sub_files, &dir_path);

        // Aggregate
        let total_size: u64 = children.iter().map(|c| c.size).sum();
        let file_count: usize = children.iter().map(|c| c.file_count).sum();
        let avg_score = if total_size > 0 {
            children.iter().map(|c| c.waste_score * c.size as f64).sum::<f64>() / total_size as f64
        } else {
            0.0
        };

        // Dominant category
        let mut cat_sizes: HashMap<&str, u64> = HashMap::new();
        for c in &children {
            *cat_sizes.entry(&c.category).or_insert(0) += c.size;
        }
        let dominant = cat_sizes.into_iter()
            .max_by_key(|(_, s)| *s)
            .map(|(c, _)| c.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        result.push(TreeNode {
            name: dir_name.clone(),
            path: dir_path,
            size: total_size,
            file_count,
            waste_score: avg_score,
            category: dominant,
            is_directory: true,
            children,
        });
    }

    // Add direct files as leaf nodes
    for f in &direct_files {
        result.push(TreeNode {
            name: f.name.clone(),
            path: f.full_path.clone(),
            size: f.size,
            file_count: 1,
            waste_score: f.waste_score,
            category: f.category.clone(),
            is_directory: false,
            children: vec![],
        });
    }

    // Sort by size descending
    result.sort_by(|a, b| b.size.cmp(&a.size));

    // Filter tiny nodes (< 50KB) to reduce noise — only at this level
    result.retain(|n| n.size >= 50 * 1024);

    result
}

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
