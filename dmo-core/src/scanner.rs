use crate::types::{FileNode, ScanConfig};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use walkdir::WalkDir;

/// Convert a SystemTime to chrono DateTime<Utc>, returning None on failure.
fn system_time_to_utc(st: SystemTime) -> Option<DateTime<Utc>> {
    st.duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .map(|d| DateTime::from_timestamp(d.as_secs() as i64, d.subsec_nanos()).unwrap_or_default())
}

/// Check if a path is inside any of the denylisted directories.
fn is_denylisted(path: &Path, denylist: &[std::path::PathBuf]) -> bool {
    for denied in denylist {
        if path.starts_with(denied) {
            return true;
        }
    }
    false
}

/// Detect if a macOS .app bundle — these look like directories but are atomic.
/// We detect them by checking if the path has a .app extension component.
fn is_inside_app_bundle(path: &Path) -> bool {
    for component in path.components() {
        if let Some(s) = component.as_os_str().to_str() {
            if s.ends_with(".app") {
                return true;
            }
        }
    }
    false
}

/// Detect iCloud placeholder files (macOS).
/// These have the extension .icloud and a name starting with '.'
fn is_icloud_placeholder(path: &Path) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') && name.ends_with(".icloud") {
            return true;
        }
    }
    false
}

/// Scan a filesystem tree and return raw FileNodes with metadata.
/// No classification or scoring happens here — that's for the classifier and scorer.
pub fn scan(config: &ScanConfig) -> Result<Vec<FileNode>, ScanError> {
    let root = &config.root;
    if !root.exists() {
        return Err(ScanError::RootNotFound(root.display().to_string()));
    }

    let mut walker = WalkDir::new(root)
        .follow_links(config.follow_symlinks)
        .min_depth(1); // skip the root directory itself

    if let Some(max_depth) = config.max_depth {
        walker = walker.max_depth(max_depth);
    }

    let mut nodes = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for entry in walker {
        match entry {
            Ok(entry) => {
                let path = entry.path().to_path_buf();

                // Skip denylisted paths
                if is_denylisted(&path, &config.denylist) {
                    continue;
                }

                // Skip iCloud placeholders — they are cloud stubs, not real files
                if is_icloud_placeholder(&path) {
                    continue;
                }

                // Skip contents of .app bundles — they are atomic packages
                if is_inside_app_bundle(&path) {
                    continue;
                }

                let is_symlink = entry.path_is_symlink();
                let is_dir = entry.file_type().is_dir();
                let depth = entry.depth();

                // Get file metadata (follow symlinks = false to get link metadata)
                let metadata = match fs::symlink_metadata(&path) {
                    Ok(m) => m,
                    Err(e) => {
                        errors.push(format!("metadata error: {}: {}", path.display(), e));
                        continue;
                    }
                };

                // Skip directories — we only score files
                if is_dir {
                    continue;
                }

                let size_bytes = metadata.len();
                let modified_at = metadata.modified().ok().and_then(system_time_to_utc);
                let accessed_at = metadata.accessed().ok().and_then(system_time_to_utc);

                let node = FileNode::new(
                    path,
                    size_bytes,
                    modified_at,
                    accessed_at,
                    is_dir,
                    is_symlink,
                    depth,
                );

                nodes.push(node);
            }
            Err(e) => {
                errors.push(format!("walk error: {}", e));
            }
        }
    }

    if !errors.is_empty() {
        log::warn!("Scan completed with {} errors", errors.len());
        for e in errors.iter().take(10) {
            log::debug!("  {}", e);
        }
    }

    Ok(nodes)
}

// ─── Errors ───

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("Root directory not found: {0}")]
    RootNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Walk error: {0}")]
    Walk(#[from] walkdir::Error),
}
