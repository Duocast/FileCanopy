# FileCanopy architecture

## Module layout

```
src/
├── main.rs              # binary entry point
├── lib.rs               # library root, public re-exports
├── error.rs             # crate-wide Error / Result
├── telemetry.rs         # tracing initialization
├── config.rs            # persistent user config (TOML)
│
├── cli/
│   ├── args.rs          # clap structs for every subcommand
│   └── commands.rs      # dispatch — thin glue to library modules
│
├── scanner/
│   ├── walker.rs        # parallel directory traversal (jwalk + rayon)
│   ├── metadata.rs      # FileEntry, EntryKind
│   └── line_counter.rs  # source-line counts for code-repo assessments
│
├── analysis/
│   ├── tree.rs          # hierarchical SizeTree built from a ScanReport
│   ├── largest.rs       # top-N "space hogs"
│   ├── hasher.rs        # blake3 / xxh3 fingerprints
│   └── duplicates.rs    # size-→prefix-→full-content duplicate detection
│
├── visualization/
│   └── treemap.rs       # squarified treemap layout + SVG/PNG render
│
├── reports/
│   ├── pdf.rs           # printpdf
│   ├── excel.rs         # rust_xlsxwriter
│   ├── html.rs          # askama template (templates/report.html)
│   ├── csv.rs           # csv crate
│   └── json.rs          # serde_json (intermediate format)
│
├── history/
│   ├── snapshot.rs      # Snapshot type
│   ├── store.rs         # SQLite-backed SnapshotStore
│   └── compare.rs       # DiffReport between two snapshots
│
├── dedup/               # apply DedupStrategy to a DuplicatesReport
│
├── scheduler/
│   ├── linux.rs         # crontab tagged with `# filecanopy:<name>`
│   └── windows.rs       # ITaskService COM via the windows crate
│
└── platform/
    ├── linux.rs         # stat(2)-based metrics
    └── windows.rs       # Win32 file-info APIs
```

## Data flow

```
scanner::walker::scan  ─►  ScanReport (flat list of FileEntry)
                             │
              ┌──────────────┼──────────────┬──────────────┐
              ▼              ▼              ▼              ▼
       analysis::tree   analysis::largest  analysis::    history::
       (hierarchy)      (top-N)            duplicates    snapshot
              │              │              │              │
              ▼              ▼              ▼              ▼
       visualization::      reports::*    dedup::apply  history::compare
       treemap
```

Every long-running operation streams progress through `indicatif` and emits
structured events via `tracing`.

## Target environments

- **Rust toolchain:** pinned to 1.95.0 via `rust-toolchain.toml`.
- **Linux:** `x86_64-unknown-linux-gnu`, glibc.
- **Windows:** `x86_64-pc-windows-msvc`.

CI builds and tests on both via `.github/workflows/ci.yml`.
