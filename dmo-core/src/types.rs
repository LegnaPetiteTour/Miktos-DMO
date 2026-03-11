use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─── Waste Categories ───
// Each file is classified into exactly one category.
// Categories with risk 0.00 are on the permanent denylist — the organism cannot perceive them.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WasteCategory {
    ApplicationCache,
    PackageManagerCache,
    BrowserCache,
    BuildArtifact,
    SystemLog,
    TempFile,
    StaleDownload,
    // Protected categories (risk = 0.00, organism never touches)
    UserDocument,
    AppBundle,
    SystemFile,
    Database,
    CloudSync,
    DeveloperWorkingTree,
    // Fallback
    Unknown,
}

impl WasteCategory {
    /// Initial type_risk value per the research document (Section 6.3).
    /// These are conservative starting priors, not calibrated outputs.
    pub fn type_risk(&self) -> f64 {
        match self {
            Self::ApplicationCache => 0.95,
            Self::PackageManagerCache => 0.95,
            Self::BrowserCache => 0.90,
            Self::BuildArtifact => 0.85,
            Self::SystemLog => 0.80,
            Self::TempFile => 0.90,
            Self::StaleDownload => 0.60,
            // Protected — zero chemoattractant
            Self::UserDocument => 0.00,
            Self::AppBundle => 0.00,
            Self::SystemFile => 0.00,
            Self::Database => 0.00,
            Self::CloudSync => 0.00,
            Self::DeveloperWorkingTree => 0.00,
            Self::Unknown => 0.10,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::ApplicationCache => "Application Cache",
            Self::PackageManagerCache => "Package Manager Cache",
            Self::BrowserCache => "Browser Cache",
            Self::BuildArtifact => "Build Artifact",
            Self::SystemLog => "System Log",
            Self::TempFile => "Temporary File",
            Self::StaleDownload => "Stale Download",
            Self::UserDocument => "User Document",
            Self::AppBundle => "App Bundle",
            Self::SystemFile => "System File",
            Self::Database => "Database",
            Self::CloudSync => "Cloud Sync",
            Self::DeveloperWorkingTree => "Dev Working Tree",
            Self::Unknown => "Unknown",
        }
    }

    /// Whether this category is on the permanent denylist.
    pub fn is_protected(&self) -> bool {
        self.type_risk() == 0.00
    }
}

impl std::fmt::Display for WasteCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ─── File Node ───
// Represents a single file observed during a filesystem scan, with all metadata
// needed for classification and scoring.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
    pub accessed_at: Option<DateTime<Utc>>,
    pub is_directory: bool,
    pub is_symlink: bool,
    pub depth: usize,
    // Classification (set by classifier)
    pub category: WasteCategory,
    // Scoring (set by scorer)
    pub waste_score: f64,
    // Component scores for transparency
    pub size_weight: f64,
    pub age_weight: f64,
    pub recency_score: f64,
}

impl FileNode {
    /// Create a new FileNode with raw metadata, unclassified and unscored.
    pub fn new(
        path: PathBuf,
        size_bytes: u64,
        modified_at: Option<DateTime<Utc>>,
        accessed_at: Option<DateTime<Utc>>,
        is_directory: bool,
        is_symlink: bool,
        depth: usize,
    ) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase());

        Self {
            path,
            name,
            extension,
            size_bytes,
            modified_at,
            accessed_at,
            is_directory,
            is_symlink,
            depth,
            category: WasteCategory::Unknown,
            waste_score: 0.0,
            size_weight: 0.0,
            age_weight: 0.0,
            recency_score: 0.0,
        }
    }
}

// ─── Scan Configuration ───

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub root: PathBuf,
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
    pub denylist: Vec<PathBuf>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            max_depth: None,
            follow_symlinks: false, // Safety: never follow symlinks by default
            denylist: Vec::new(),
        }
    }
}

// ─── Scan Summary ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub root: PathBuf,
    pub total_files: usize,
    pub total_directories: usize,
    pub total_size_bytes: u64,
    pub waste_candidates: usize,
    pub waste_size_bytes: u64,
    pub scan_duration_ms: u64,
    pub categories: Vec<CategorySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category: WasteCategory,
    pub file_count: usize,
    pub total_bytes: u64,
    pub avg_waste_score: f64,
}
