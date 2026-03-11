use crate::types::FileNode;
use chrono::Utc;

/// Compute waste_score for a single FileNode using the formula from the research document:
///
///   waste_score(f) = size_weight(f) × age_weight(f) × type_risk(f) × (1 − recency_score(f))
///
/// waste_score is a heuristic priority rank [0.0, 1.0], not a calibrated probability.
/// It orders chemoattractant intensity and proposal priority.
pub fn score(node: &mut FileNode) {
    let type_risk = node.category.type_risk();

    // Protected categories: score is always 0.0, no computation needed
    if type_risk == 0.0 {
        node.waste_score = 0.0;
        node.size_weight = 0.0;
        node.age_weight = 0.0;
        node.recency_score = 1.0;
        return;
    }

    // ── size_weight: log10(size_bytes / 1024), normalized to [0, 1] ──
    // Files < 1KB get 0.0. We cap at ~10GB (log10(10_000_000) ≈ 7) for normalization.
    let size_kb = (node.size_bytes as f64) / 1024.0;
    let size_weight = if size_kb > 1.0 {
        (size_kb.log10() / 7.0).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // ── age_weight: days_since_mtime / 365, capped at 1.0 ──
    let now = Utc::now();
    let age_weight = node
        .modified_at
        .map(|mtime| {
            let days = (now - mtime).num_days().max(0) as f64;
            (days / 365.0).min(1.0)
        })
        .unwrap_or(0.5); // Unknown age: moderate assumption

    // ── recency_score: exp(−days_since_access / 30) ──
    // Recently accessed files are less likely waste. Exponential decay.
    let recency_score = node
        .accessed_at
        .map(|atime| {
            let days = (now - atime).num_days().max(0) as f64;
            (-days / 30.0).exp()
        })
        .unwrap_or(0.3); // Unknown access time: assume moderately stale

    // ── Composite score ──
    let waste_score = size_weight * age_weight * type_risk * (1.0 - recency_score);

    // Store all components for transparency / debugging
    node.size_weight = size_weight;
    node.age_weight = age_weight;
    node.recency_score = recency_score;
    node.waste_score = waste_score.clamp(0.0, 1.0);
}

/// Score all nodes in a batch.
pub fn score_all(nodes: &mut [FileNode]) {
    for node in nodes.iter_mut() {
        score(node);
    }
}

// ═══════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FileNode, WasteCategory};
    use chrono::{Duration, Utc};
    use std::path::PathBuf;

    fn make_node(size: u64, days_old: i64, days_since_access: i64, category: WasteCategory) -> FileNode {
        let now = Utc::now();
        let mut node = FileNode::new(
            PathBuf::from("/test/file.tmp"),
            size,
            Some(now - Duration::days(days_old)),
            Some(now - Duration::days(days_since_access)),
            false,
            false,
            1,
        );
        node.category = category;
        node
    }

    #[test]
    fn test_protected_always_zero() {
        let mut node = make_node(1_000_000, 365, 365, WasteCategory::UserDocument);
        score(&mut node);
        assert_eq!(node.waste_score, 0.0);
    }

    #[test]
    fn test_large_old_cache_scores_high() {
        let mut node = make_node(100_000_000, 180, 90, WasteCategory::ApplicationCache);
        score(&mut node);
        assert!(node.waste_score > 0.1, "Large old cache should score high, got {}", node.waste_score);
    }

    #[test]
    fn test_recently_accessed_scores_lower() {
        let mut old = make_node(10_000_000, 60, 60, WasteCategory::ApplicationCache);
        let mut recent = make_node(10_000_000, 60, 1, WasteCategory::ApplicationCache);
        score(&mut old);
        score(&mut recent);
        assert!(
            old.waste_score > recent.waste_score,
            "Old access ({}) should score higher than recent access ({})",
            old.waste_score,
            recent.waste_score
        );
    }

    #[test]
    fn test_tiny_file_scores_low() {
        let mut node = make_node(100, 365, 365, WasteCategory::TempFile);
        score(&mut node);
        assert!(node.waste_score < 0.05, "Tiny file should score very low, got {}", node.waste_score);
    }
}
