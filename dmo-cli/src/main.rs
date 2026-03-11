use clap::Parser;
use dmo_core::db::DmoDb;
use dmo_core::pipeline;
use dmo_core::types::ScanConfig;
use std::path::PathBuf;

/// DMO — Digital Maintenance Organism
///
/// Filesystem scanner and waste scorer. Phase 0: CLI-only, no UI.
/// Scans a directory, classifies files into waste categories,
/// computes waste_score, stores results in SQLite, and prints a report.
#[derive(Parser, Debug)]
#[command(name = "dmo", version, about)]
struct Args {
    /// Directory to scan
    #[arg(short, long, default_value = ".")]
    path: PathBuf,

    /// Maximum directory depth to scan
    #[arg(short = 'd', long, default_value = "8")]
    max_depth: usize,

    /// Number of top waste candidates to display
    #[arg(short = 'n', long, default_value = "25")]
    top: usize,

    /// Path to the SQLite database file
    #[arg(long, default_value = "dmo_scan.db")]
    db: PathBuf,

    /// Output format: text, json
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Show all categories in summary (including zero-score)
    #[arg(long)]
    all_categories: bool,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format_timestamp(None)
        .init();

    let args = Args::parse();

    // Resolve path
    let root = if args.path.is_relative() {
        std::env::current_dir().unwrap().join(&args.path)
    } else {
        args.path.clone()
    };

    if !root.exists() {
        eprintln!("Error: path does not exist: {}", root.display());
        std::process::exit(1);
    }

    eprintln!(
        "\n  DMO Scanner v{}\n  Scanning: {}\n  Max depth: {}\n",
        env!("CARGO_PKG_VERSION"),
        root.display(),
        args.max_depth,
    );

    // Configure scan
    let config = ScanConfig {
        root: root.clone(),
        max_depth: Some(args.max_depth),
        follow_symlinks: false,
        denylist: default_denylist(),
    };

    // Run pipeline
    let (nodes, summary) = match pipeline::run(&config) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Scan failed: {}", e);
            std::process::exit(1);
        }
    };

    // Store in database
    let db = match DmoDb::open(&args.db) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Database error: {}", e);
            std::process::exit(1);
        }
    };

    let scan_id = match db.store_scan(&summary, &nodes) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to store scan: {}", e);
            std::process::exit(1);
        }
    };

    // Output
    match args.format.as_str() {
        "json" => print_json(&summary, &nodes, args.top),
        _ => print_text(&summary, &nodes, args.top, scan_id, &args.db),
    }
}

fn print_text(summary: &dmo_core::ScanSummary, nodes: &[dmo_core::FileNode], top: usize, scan_id: i64, db_path: &PathBuf) {
    println!("═══════════════════════════════════════════════════════════");
    println!("  SCAN SUMMARY");
    println!("═══════════════════════════════════════════════════════════");
    println!("  Root:             {}", summary.root.display());
    println!("  Total files:      {}", summary.total_files);
    println!("  Total size:       {}", format_bytes(summary.total_size_bytes));
    println!("  Waste candidates: {}", summary.waste_candidates);
    println!("  Waste size:       {}", format_bytes(summary.waste_size_bytes));
    println!("  Scan time:        {}ms", summary.scan_duration_ms);
    println!("  Scan ID:          {}", scan_id);
    println!("  Database:         {}", db_path.display());
    println!();

    // Category breakdown
    if !summary.categories.is_empty() {
        println!("  CATEGORY BREAKDOWN");
        println!("  {:<30} {:>8} {:>12} {:>8}", "Category", "Files", "Size", "Avg Score");
        println!("  {}", "─".repeat(62));
        for cat in &summary.categories {
            println!(
                "  {:<30} {:>8} {:>12} {:>8.4}",
                cat.category.label(),
                cat.file_count,
                format_bytes(cat.total_bytes),
                cat.avg_waste_score,
            );
        }
        println!();
    }

    // Top waste candidates
    let display_nodes: Vec<&dmo_core::FileNode> = nodes
        .iter()
        .filter(|n| n.waste_score > 0.0)
        .take(top)
        .collect();

    if display_nodes.is_empty() {
        println!("  No waste candidates found. The filesystem looks clean.");
    } else {
        println!("  TOP {} WASTE CANDIDATES", display_nodes.len());
        println!("  {:<6} {:<24} {:>10} {}", "Score", "Category", "Size", "Path");
        println!("  {}", "─".repeat(80));

        for node in &display_nodes {
            let path_display = abbreviate_path(&node.path, 60);
            println!(
                "  {:<6.4} {:<24} {:>10} {}",
                node.waste_score,
                node.category.label(),
                format_bytes(node.size_bytes),
                path_display,
            );
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════════════");
}

fn print_json(summary: &dmo_core::ScanSummary, nodes: &[dmo_core::FileNode], top: usize) {
    let top_nodes: Vec<&dmo_core::FileNode> = nodes
        .iter()
        .filter(|n| n.waste_score > 0.0)
        .take(top)
        .collect();

    let output = serde_json::json!({
        "summary": summary,
        "top_waste": top_nodes,
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

// ─── Formatting helpers ───

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn abbreviate_path(path: &std::path::Path, max_len: usize) -> String {
    let s = path.to_string_lossy().to_string();
    if s.len() <= max_len {
        return s;
    }
    // Replace home directory with ~
    if let Ok(home) = std::env::var("HOME") {
        let abbreviated = s.replace(&home, "~");
        if abbreviated.len() <= max_len {
            return abbreviated;
        }
        // Still too long — truncate from the left
        let trimmed = &abbreviated[abbreviated.len() - max_len + 3..];
        return format!("...{}", trimmed);
    }
    let trimmed = &s[s.len() - max_len + 3..];
    format!("...{}", trimmed)
}

/// Default denylist for macOS — paths the DMO should never scan.
fn default_denylist() -> Vec<PathBuf> {
    let mut list = vec![
        PathBuf::from("/System"),
        PathBuf::from("/usr"),
        PathBuf::from("/bin"),
        PathBuf::from("/sbin"),
        PathBuf::from("/Applications"),
    ];

    // Add home-directory protected paths
    if let Ok(home) = std::env::var("HOME") {
        let h = PathBuf::from(&home);
        list.push(h.join("Documents"));
        list.push(h.join("Desktop"));
        list.push(h.join("Pictures"));
        list.push(h.join("Music"));
        list.push(h.join("Movies"));
        list.push(h.join("Applications"));
        // Keychains and credentials
        list.push(h.join("Library/Keychains"));
        list.push(h.join(".ssh"));
        list.push(h.join(".gnupg"));
    }

    list
}
