use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "filecanopy",
    version,
    about = "Disk space analyzer, treemap visualizer, duplicate finder, and report generator.",
    long_about = None,
)]
pub struct Cli {
    /// Path to a custom config file (overrides default location).
    #[arg(long, global = true, env = "FILECANOPY_CONFIG")]
    pub config: Option<PathBuf>,

    /// Increase logging verbosity (-v, -vv, -vvv).
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress all non-error output.
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Scan a directory tree and produce a size report.
    Scan(ScanArgs),

    /// Render a treemap chart of disk usage.
    Treemap(TreemapArgs),

    /// List the largest files (top "space hogs").
    Top(TopArgs),

    /// Find duplicate files by content hash.
    Duplicates(DuplicatesArgs),

    /// Deduplicate previously-detected duplicates.
    Dedup(DedupArgs),

    /// Take a snapshot for historical comparison.
    Snapshot(SnapshotArgs),

    /// Compare two snapshots and show what changed.
    Compare(CompareArgs),

    /// Count lines in source files (for code-repo assessments).
    LineCount(LineCountArgs),

    /// Install or remove a scheduled-task entry (cron / Task Scheduler).
    Schedule(ScheduleArgs),

    /// Export the most recent scan result in a chosen format.
    Export(ExportArgs),
}

#[derive(Debug, clap::Args)]
pub struct ScanArgs {
    /// One or more directory roots to scan.
    #[arg(required = true)]
    pub roots: Vec<PathBuf>,

    /// Maximum depth to recurse (unlimited if omitted).
    #[arg(long)]
    pub depth: Option<usize>,

    /// Follow symbolic links during traversal.
    #[arg(long)]
    pub follow_symlinks: bool,

    /// Glob patterns to exclude.
    #[arg(long = "exclude", value_name = "GLOB")]
    pub excludes: Vec<String>,

    /// Number of worker threads (defaults to physical CPUs).
    #[arg(long)]
    pub threads: Option<usize>,

    /// Write the result to a file (format inferred from extension).
    #[arg(long, short)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, clap::Args)]
pub struct TreemapArgs {
    pub root: PathBuf,
    /// Output SVG or PNG path.
    #[arg(long, short)]
    pub output: PathBuf,
    /// Maximum tiles to draw.
    #[arg(long, default_value_t = 500)]
    pub max_tiles: usize,
    #[arg(long, default_value_t = 1600)]
    pub width: u32,
    #[arg(long, default_value_t = 1000)]
    pub height: u32,
}

#[derive(Debug, clap::Args)]
pub struct TopArgs {
    pub root: PathBuf,
    /// Number of entries to show.
    #[arg(short = 'n', long, default_value_t = 50)]
    pub limit: usize,
    /// Only consider files larger than this many bytes.
    #[arg(long)]
    pub min_size: Option<u64>,
}

#[derive(Debug, clap::Args)]
pub struct DuplicatesArgs {
    pub roots: Vec<PathBuf>,
    /// Minimum file size to consider (skip empty / tiny files).
    #[arg(long, default_value_t = 1024)]
    pub min_size: u64,
    /// Hashing algorithm.
    #[arg(long, value_enum, default_value_t = HashAlgo::Blake3)]
    pub algo: HashAlgo,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HashAlgo {
    Blake3,
    Xxh3,
}

#[derive(Debug, clap::Args)]
pub struct DedupArgs {
    /// Path to a duplicates report produced by `filecanopy duplicates`.
    pub report: PathBuf,
    /// Resolution strategy.
    #[arg(long, value_enum, default_value_t = DedupStrategy::DryRun)]
    pub strategy: DedupStrategy,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DedupStrategy {
    DryRun,
    Delete,
    Hardlink,
    Symlink,
}

#[derive(Debug, clap::Args)]
pub struct SnapshotArgs {
    pub root: PathBuf,
    /// Optional label for this snapshot.
    #[arg(long)]
    pub label: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct CompareArgs {
    pub before: String,
    pub after: String,
    /// Only show entries whose size changed by at least this many bytes.
    #[arg(long, default_value_t = 0)]
    pub min_delta: u64,
}

#[derive(Debug, clap::Args)]
pub struct LineCountArgs {
    pub root: PathBuf,
    /// File extensions to include (e.g. rs, py, ts).
    #[arg(long, value_name = "EXT")]
    pub ext: Vec<String>,
    /// Sort threshold for "monolithic file" flagging.
    #[arg(long, default_value_t = 1000)]
    pub monolith_threshold: usize,
}

#[derive(Debug, clap::Args)]
pub struct ScheduleArgs {
    #[command(subcommand)]
    pub action: ScheduleAction,
}

#[derive(Debug, Subcommand)]
pub enum ScheduleAction {
    /// Install a recurring task on the host OS scheduler.
    Install {
        /// Cron expression (Linux) or task interval description.
        #[arg(long)]
        cron: String,
        /// Command to run on schedule (passed to filecanopy).
        #[arg(long)]
        command: String,
        /// Unique task name.
        #[arg(long, default_value = "filecanopy")]
        name: String,
    },
    /// Remove a previously-installed task.
    Remove {
        #[arg(long, default_value = "filecanopy")]
        name: String,
    },
    /// List installed filecanopy tasks.
    List,
}

#[derive(Debug, clap::Args)]
pub struct ExportArgs {
    /// Path to the scan result to export.
    pub input: PathBuf,
    /// Destination file (extension inferred unless `--format` set).
    #[arg(long, short)]
    pub output: PathBuf,
    #[arg(long, value_enum)]
    pub format: Option<ExportFormat>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportFormat {
    Pdf,
    Excel,
    Html,
    Csv,
    Json,
}
