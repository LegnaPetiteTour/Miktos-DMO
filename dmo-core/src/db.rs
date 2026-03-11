#[allow(unused_imports)]
use crate::types::{FileNode, ScanSummary, WasteCategory};
use rusqlite::{params, Connection, Result as SqlResult};
use std::path::{Path, PathBuf};

/// The DMO database — stores scan results, action logs, and learning data.
pub struct DmoDb {
    conn: Connection,
}

impl DmoDb {
    /// Open or create the database at the given path.
    pub fn open(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing).
    pub fn open_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Create tables if they don't exist.
    fn initialize(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS scan_snapshots (
                id              INTEGER PRIMARY KEY,
                root_path       TEXT NOT NULL,
                scanned_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
                total_files     INTEGER,
                total_dirs      INTEGER,
                total_size      INTEGER,
                waste_candidates INTEGER,
                waste_size      INTEGER,
                duration_ms     INTEGER
            );

            CREATE TABLE IF NOT EXISTS file_nodes (
                id              INTEGER PRIMARY KEY,
                scan_id         INTEGER NOT NULL,
                path            TEXT NOT NULL,
                name            TEXT NOT NULL,
                extension       TEXT,
                size_bytes      INTEGER NOT NULL,
                modified_at     TEXT,
                accessed_at     TEXT,
                is_symlink      INTEGER DEFAULT 0,
                depth           INTEGER,
                category        TEXT NOT NULL,
                waste_score     REAL NOT NULL,
                size_weight     REAL,
                age_weight      REAL,
                recency_score   REAL,
                FOREIGN KEY (scan_id) REFERENCES scan_snapshots(id)
            );

            CREATE TABLE IF NOT EXISTS action_log (
                id              INTEGER PRIMARY KEY,
                timestamp       DATETIME DEFAULT CURRENT_TIMESTAMP,
                path            TEXT NOT NULL,
                action_type     TEXT NOT NULL,
                category        TEXT NOT NULL,
                size_bytes      INTEGER,
                confidence      REAL,
                user_decision   TEXT,
                quarantine_path TEXT,
                sha256          TEXT,
                decision_time_ms INTEGER
            );

            CREATE INDEX IF NOT EXISTS idx_file_nodes_score ON file_nodes(waste_score DESC);
            CREATE INDEX IF NOT EXISTS idx_file_nodes_category ON file_nodes(category);
            CREATE INDEX IF NOT EXISTS idx_file_nodes_scan ON file_nodes(scan_id);
            "
        )?;
        Ok(())
    }

    /// Store a complete scan result. Returns the scan_id.
    pub fn store_scan(&self, summary: &ScanSummary, nodes: &[FileNode]) -> SqlResult<i64> {
        // Insert scan snapshot
        self.conn.execute(
            "INSERT INTO scan_snapshots (root_path, total_files, total_dirs, total_size, waste_candidates, waste_size, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                summary.root.to_string_lossy().to_string(),
                summary.total_files,
                summary.total_directories,
                summary.total_size_bytes,
                summary.waste_candidates,
                summary.waste_size_bytes,
                summary.scan_duration_ms,
            ],
        )?;

        let scan_id = self.conn.last_insert_rowid();

        // Insert file nodes in a transaction for performance
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO file_nodes (scan_id, path, name, extension, size_bytes, modified_at, accessed_at, is_symlink, depth, category, waste_score, size_weight, age_weight, recency_score)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)"
            )?;

            for node in nodes {
                if node.category.is_protected() {
                    continue; // Don't store protected files in the database
                }

                stmt.execute(params![
                    scan_id,
                    node.path.to_string_lossy().to_string(),
                    node.name,
                    node.extension,
                    node.size_bytes,
                    node.modified_at.map(|t| t.to_rfc3339()),
                    node.accessed_at.map(|t| t.to_rfc3339()),
                    node.is_symlink as i32,
                    node.depth,
                    format!("{:?}", node.category),
                    node.waste_score,
                    node.size_weight,
                    node.age_weight,
                    node.recency_score,
                ])?;
            }
        }
        tx.commit()?;

        Ok(scan_id)
    }

    /// Get the top N waste candidates from a scan, ordered by waste_score descending.
    pub fn top_waste(&self, scan_id: i64, limit: usize) -> SqlResult<Vec<StoredNode>> {
        let mut stmt = self.conn.prepare(
            "SELECT path, name, size_bytes, category, waste_score
             FROM file_nodes
             WHERE scan_id = ?1
             ORDER BY waste_score DESC
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![scan_id, limit], |row| {
            Ok(StoredNode {
                path: PathBuf::from(row.get::<_, String>(0)?),
                name: row.get(1)?,
                size_bytes: row.get(2)?,
                category: row.get(3)?,
                waste_score: row.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get scan count.
    pub fn scan_count(&self) -> SqlResult<i64> {
        self.conn.query_row("SELECT COUNT(*) FROM scan_snapshots", [], |row| row.get(0))
    }
}

/// A simplified node for query results.
#[derive(Debug)]
pub struct StoredNode {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub category: String,
    pub waste_score: f64,
}

// ═══════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ScanSummary;

    #[test]
    fn test_create_and_query() {
        let db = DmoDb::open_memory().unwrap();
        assert_eq!(db.scan_count().unwrap(), 0);

        let summary = ScanSummary {
            root: PathBuf::from("/test"),
            total_files: 100,
            total_directories: 10,
            total_size_bytes: 1_000_000,
            waste_candidates: 20,
            waste_size_bytes: 500_000,
            scan_duration_ms: 150,
            categories: vec![],
        };

        let mut node = FileNode::new(
            PathBuf::from("/test/cache/file.tmp"),
            50_000,
            None,
            None,
            false,
            false,
            2,
        );
        node.category = WasteCategory::TempFile;
        node.waste_score = 0.85;

        let scan_id = db.store_scan(&summary, &[node]).unwrap();
        assert_eq!(scan_id, 1);
        assert_eq!(db.scan_count().unwrap(), 1);

        let top = db.top_waste(scan_id, 10).unwrap();
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].name, "file.tmp");
        assert!((top[0].waste_score - 0.85).abs() < 0.001);
    }
}
