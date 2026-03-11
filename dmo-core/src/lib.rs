//! # DMO Core
//!
//! Core library for the Digital Maintenance Organism project.
//!
//! Provides filesystem scanning, waste classification, scoring, and storage.
//! This crate contains no UI code — it is the maintenance intelligence layer.
//!
//! ## Architecture
//!
//! The processing pipeline:
//! 1. **Scanner** — walks the filesystem, extracts metadata, skips protected paths
//! 2. **Classifier** — assigns each file a WasteCategory based on path patterns
//! 3. **Scorer** — computes waste_score using the weighted formula
//! 4. **Database** — stores results in SQLite for audit and learning
//!
//! ## Usage
//!
//! ```no_run
//! use dmo_core::{pipeline, types::ScanConfig};
//! use std::path::PathBuf;
//!
//! let config = ScanConfig {
//!     root: PathBuf::from("/Users/me"),
//!     max_depth: Some(6),
//!     ..Default::default()
//! };
//!
//! let (nodes, summary) = pipeline::run(&config).unwrap();
//! ```

pub mod classifier;
pub mod db;
pub mod pipeline;
pub mod scanner;
pub mod scorer;
pub mod types;

// Re-exports for convenience
pub use types::{FileNode, ScanConfig, ScanSummary, WasteCategory};
