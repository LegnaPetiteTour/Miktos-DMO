use crate::classifier;
use crate::scanner;
use crate::scorer;
use crate::types::{CategorySummary, FileNode, ScanConfig, ScanSummary, WasteCategory};
use std::collections::HashMap;
use std::time::Instant;

/// Run the full scan → classify → score pipeline.
///
/// Returns the processed nodes and a summary.
pub fn run(config: &ScanConfig) -> Result<(Vec<FileNode>, ScanSummary), PipelineError> {
    let start = Instant::now();

    // Phase 1: Scan
    let mut nodes = scanner::scan(config)?;
    let total_files = nodes.len();

    // Phase 2: Classify
    classifier::classify_all(&mut nodes);

    // Phase 3: Score
    scorer::score_all(&mut nodes);

    // Build summary
    let elapsed = start.elapsed();
    let summary = build_summary(config, &nodes, total_files, elapsed.as_millis() as u64);

    // Sort by waste_score descending
    nodes.sort_by(|a, b| b.waste_score.partial_cmp(&a.waste_score).unwrap_or(std::cmp::Ordering::Equal));

    Ok((nodes, summary))
}

fn build_summary(
    config: &ScanConfig,
    nodes: &[FileNode],
    total_files: usize,
    duration_ms: u64,
) -> ScanSummary {
    let total_size_bytes: u64 = nodes.iter().map(|n| n.size_bytes).sum();
    let waste_nodes: Vec<&FileNode> = nodes.iter().filter(|n| n.waste_score > 0.0).collect();
    let waste_size_bytes: u64 = waste_nodes.iter().map(|n| n.size_bytes).sum();

    // Category breakdown
    let mut cat_map: HashMap<WasteCategory, (usize, u64, f64)> = HashMap::new();
    for node in nodes {
        if node.category.is_protected() {
            continue;
        }
        let entry = cat_map.entry(node.category).or_insert((0, 0, 0.0));
        entry.0 += 1;
        entry.1 += node.size_bytes;
        entry.2 += node.waste_score;
    }

    let mut categories: Vec<CategorySummary> = cat_map
        .into_iter()
        .map(|(category, (count, bytes, total_score))| CategorySummary {
            category,
            file_count: count,
            total_bytes: bytes,
            avg_waste_score: if count > 0 { total_score / count as f64 } else { 0.0 },
        })
        .collect();

    // Sort by total bytes descending
    categories.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));

    ScanSummary {
        root: config.root.clone(),
        total_files,
        total_directories: 0, // We skip directories in the scanner
        total_size_bytes,
        waste_candidates: waste_nodes.len(),
        waste_size_bytes,
        scan_duration_ms: duration_ms,
        categories,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Scan error: {0}")]
    Scan(#[from] scanner::ScanError),
}
