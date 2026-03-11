use crate::types::{FileNode, WasteCategory};
use std::path::Path;

/// Classify a FileNode into a WasteCategory based on path patterns,
/// file extensions, and parent directory names.
///
/// The classifier applies rules in priority order:
/// 1. Protected categories (denylist) — checked first, always wins
/// 2. High-confidence waste patterns (cache dirs, temp dirs, build outputs)
/// 3. Medium-confidence patterns (logs, stale downloads)
/// 4. Fallback to Unknown
pub fn classify(node: &mut FileNode) {
    let path = &node.path;
    let path_str = path.to_string_lossy().to_lowercase();

    // ── Phase 1: Protected categories (permanent denylist) ──
    // These emit zero chemoattractant. The organism cannot perceive them.

    if is_user_document(path) {
        node.category = WasteCategory::UserDocument;
        return;
    }

    if is_system_file(&path_str) {
        node.category = WasteCategory::SystemFile;
        return;
    }

    if is_database(path, node.extension.as_deref()) {
        node.category = WasteCategory::Database;
        return;
    }

    if is_cloud_sync(&path_str) {
        node.category = WasteCategory::CloudSync;
        return;
    }

    if is_developer_working_tree(&path_str, &node.name) {
        node.category = WasteCategory::DeveloperWorkingTree;
        return;
    }

    // ── Phase 2: Waste categories (ordered by confidence) ──

    if is_browser_cache(&path_str) {
        node.category = WasteCategory::BrowserCache;
        return;
    }

    if is_package_manager_cache(&path_str) {
        node.category = WasteCategory::PackageManagerCache;
        return;
    }

    if is_application_cache(&path_str) {
        node.category = WasteCategory::ApplicationCache;
        return;
    }

    if is_build_artifact(&path_str, node.extension.as_deref()) {
        node.category = WasteCategory::BuildArtifact;
        return;
    }

    if is_temp_file(&path_str, node.extension.as_deref(), &node.name) {
        node.category = WasteCategory::TempFile;
        return;
    }

    if is_system_log(&path_str, node.extension.as_deref()) {
        node.category = WasteCategory::SystemLog;
        return;
    }

    if is_stale_download(&path_str) {
        node.category = WasteCategory::StaleDownload;
        return;
    }

    // ── Fallback ──
    node.category = WasteCategory::Unknown;
}

/// Classify all nodes in a batch.
pub fn classify_all(nodes: &mut [FileNode]) {
    for node in nodes.iter_mut() {
        classify(node);
    }
}

// ═══════════════════════════════════════════════════
// Detection functions
// ═══════════════════════════════════════════════════

fn is_user_document(path: &Path) -> bool {
    let home = dirs_hint();
    let protected_dirs = [
        "documents", "desktop", "pictures", "music", "movies",
    ];
    if let Some(home) = home {
        for dir in &protected_dirs {
            let protected = home.join(dir);
            let protected_str = protected.to_string_lossy().to_lowercase();
            let path_str = path.to_string_lossy().to_lowercase();
            if path_str.starts_with(&protected_str) {
                return true;
            }
        }
    }
    false
}

fn is_system_file(path_str: &str) -> bool {
    path_str.starts_with("/system/")
        || path_str.starts_with("/usr/")
        || path_str.starts_with("/bin/")
        || path_str.starts_with("/sbin/")
        || (path_str.starts_with("/library/") && !path_str.contains("/caches/"))
}

fn is_database(_path: &Path, ext: Option<&str>) -> bool {
    matches!(
        ext,
        Some("db" | "sqlite" | "sqlite3" | "realm" | "sqlite-wal" | "sqlite-shm")
    )
}

fn is_cloud_sync(path_str: &str) -> bool {
    // iCloud Drive, Dropbox, Google Drive, OneDrive
    path_str.contains("/mobile documents/")         // iCloud
        || path_str.contains("/icloud drive/")
        || path_str.contains("/dropbox/")
        || path_str.contains("/google drive/")
        || path_str.contains("/onedrive/")
}

fn is_developer_working_tree(path_str: &str, name: &str) -> bool {
    // .git directories, IDE project files
    name == ".git"
        || name == ".gitignore"
        || name == ".gitmodules"
        || path_str.contains("/.git/")
        || name == ".idea"
        || path_str.contains("/.idea/")
        || name == ".vscode"
        || path_str.contains("/.vscode/")
}

fn is_browser_cache(path_str: &str) -> bool {
    // Chrome, Safari, Firefox cache directories
    (path_str.contains("/google/chrome/") && path_str.contains("/cache"))
        || (path_str.contains("/chromium/") && path_str.contains("/cache"))
        || (path_str.contains("/mozilla/firefox/") && path_str.contains("/cache"))
        || (path_str.contains("/safari/") && path_str.contains("/cache"))
        || (path_str.contains("/brave") && path_str.contains("/cache"))
        || (path_str.contains("/microsoft edge/") && path_str.contains("/cache"))
        || path_str.contains("/caches/com.google.chrome")
        || path_str.contains("/caches/com.apple.safari")
        || path_str.contains("/caches/com.brave.browser")
        || path_str.contains("/caches/org.mozilla.firefox")
}

fn is_package_manager_cache(path_str: &str) -> bool {
    // npm, pip, cargo, yarn, pnpm, homebrew, cocoapods
    path_str.contains("/.npm/_cacache")
        || path_str.contains("/.npm/_logs")
        || path_str.contains("/pip/cache/")
        || path_str.contains("/.cache/pip/")
        || path_str.contains("/.cargo/registry/cache/")
        || path_str.contains("/.cargo/registry/src/")
        || path_str.contains("/.yarn/cache/")
        || path_str.contains("/.pnpm-store/")
        || path_str.contains("/homebrew/cache/")
        || path_str.contains("/caches/homebrew/")
        || path_str.contains("/caches/cocoapods/")
        || path_str.contains("/library/caches/com.apple.dt.xcode/")
}

fn is_application_cache(path_str: &str) -> bool {
    // macOS ~/Library/Caches/* and other application cache directories
    // Excludes browser and package manager caches (already matched above)
    path_str.contains("/library/caches/")
        || path_str.contains("/cache/")
        && !path_str.contains("/library/caches/com.apple.dt.xcode/")
}

fn is_build_artifact(path_str: &str, ext: Option<&str>) -> bool {
    // Xcode DerivedData, cargo target, webpack dist, node_modules
    path_str.contains("/deriveddata/")
        || path_str.contains("/build/intermediates/")
        || (path_str.contains("/target/debug/") && path_str.contains("/.cargo/"))
        || (path_str.contains("/target/release/") && path_str.contains("/.cargo/"))
        || path_str.contains("/node_modules/")
        || path_str.contains("/.next/cache/")
        || path_str.contains("/dist/")
            && matches!(ext, Some("js" | "map" | "css"))
        || matches!(ext, Some("o" | "obj" | "dSYM"))
}

fn is_temp_file(path_str: &str, ext: Option<&str>, name: &str) -> bool {
    path_str.starts_with("/tmp/")
        || path_str.starts_with("/var/tmp/")
        || path_str.contains("/tmp/")
        || matches!(ext, Some("tmp" | "temp" | "swp" | "swo"))
        || name.starts_with("._")
        || name == ".DS_Store"
        || name == "Thumbs.db"
        || name.starts_with('~') && !name.starts_with("~/")
}

fn is_system_log(path_str: &str, ext: Option<&str>) -> bool {
    (path_str.contains("/logs/") || path_str.contains("/log/"))
        && matches!(ext, Some("log" | "log.gz" | "log.1" | "log.2" | "old"))
        || (ext == Some("log") && path_str.contains("/library/"))
}

fn is_stale_download(path_str: &str) -> bool {
    if let Some(home) = dirs_hint() {
        let downloads = home.join("downloads");
        let downloads_str = downloads.to_string_lossy().to_lowercase();
        if path_str.starts_with(&*downloads_str) {
            return true;
        }
    }
    false
}

/// Get the home directory hint without pulling in a full dependency.
fn dirs_hint() -> Option<std::path::PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(std::path::PathBuf::from)
}

// ═══════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileNode;
    use std::path::PathBuf;

    fn make_node(path: &str) -> FileNode {
        FileNode::new(
            PathBuf::from(path),
            1024,
            None,
            None,
            false,
            false,
            1,
        )
    }

    #[test]
    fn test_browser_cache() {
        let mut node = make_node("/Users/test/Library/Caches/com.google.Chrome/Default/Cache/data");
        classify(&mut node);
        assert_eq!(node.category, WasteCategory::BrowserCache);
    }

    #[test]
    fn test_npm_cache() {
        let mut node = make_node("/Users/test/.npm/_cacache/content-v2/sha512/abc");
        classify(&mut node);
        assert_eq!(node.category, WasteCategory::PackageManagerCache);
    }

    #[test]
    fn test_temp_file() {
        let mut node = make_node("/tmp/com.apple.installer/something.tmp");
        classify(&mut node);
        assert_eq!(node.category, WasteCategory::TempFile);
    }

    #[test]
    fn test_ds_store() {
        let mut node = make_node("/Users/test/Projects/.DS_Store");
        classify(&mut node);
        assert_eq!(node.category, WasteCategory::TempFile);
    }

    #[test]
    fn test_database_protected() {
        let mut node = make_node("/Users/test/app/data.sqlite");
        classify(&mut node);
        assert_eq!(node.category, WasteCategory::Database);
        assert!(node.category.is_protected());
    }

    #[test]
    fn test_documents_protected() {
        std::env::set_var("HOME", "/Users/test");
        let mut node = make_node("/Users/test/Documents/important.pdf");
        classify(&mut node);
        assert_eq!(node.category, WasteCategory::UserDocument);
        assert!(node.category.is_protected());
    }
}
