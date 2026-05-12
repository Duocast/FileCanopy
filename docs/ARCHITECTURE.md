# FileCanopy architecture

FileCanopy is a desktop GUI built on **iced 0.14**. The application code is
split into a thin UI layer that owns *all* mutable state and a set of
pure library modules that do the actual filesystem and analysis work.

## Module layout

```
src/
├── main.rs              # launches the iced application
├── lib.rs               # library root, public re-exports
├── error.rs             # crate-wide Error / Result
├── telemetry.rs         # tracing initialization
├── config.rs            # persistent user config (TOML)
│
├── ui/                  # iced 0.14 GUI
│   ├── mod.rs           # iced::application(...) entry
│   ├── app.rs           # App state + update reducer + view dispatcher
│   ├── message.rs       # flat Message enum for every UI event
│   ├── theme.rs         # light/dark theme selection
│   ├── tasks.rs         # iced::Task wrappers around blocking library calls
│   ├── views/
│   │   ├── sidebar.rs       # left-nav switcher between screens
│   │   ├── dashboard.rs     # summary tiles + quick actions
│   │   ├── scan.rs          # folder picker, progress bar, cancel
│   │   ├── treemap.rs       # treemap canvas + zoom controls
│   │   ├── largest.rs       # top-N "space hogs"
│   │   ├── duplicates.rs    # duplicate groups + dedupe strategy picker
│   │   ├── line_count.rs    # source-line counts by extension
│   │   ├── history.rs       # snapshots + diff viewer
│   │   ├── reports.rs       # export to PDF / Excel / HTML / CSV / JSON
│   │   ├── scheduler.rs     # install/remove OS scheduled tasks
│   │   └── settings.rs      # theme, follow_symlinks, ignore globs
│   └── widgets/
│       └── treemap_canvas.rs   # canvas::Program rendering the treemap
│
├── scanner/             # parallel directory traversal (jwalk + rayon)
│   ├── walker.rs
│   ├── metadata.rs
│   └── line_counter.rs
│
├── analysis/
│   ├── tree.rs          # hierarchical SizeTree
│   ├── largest.rs       # top-N
│   ├── hasher.rs        # blake3 / xxh3 + HashAlgo enum
│   └── duplicates.rs    # size → prefix-hash → full-content
│
├── visualization/
│   └── treemap.rs       # squarified treemap layout (pure data)
│
├── reports/             # ExportFormat enum + per-format exporters
│   ├── pdf.rs · excel.rs · html.rs · csv.rs · json.rs
│
├── history/             # SQLite-backed Snapshot store + DiffReport
│   ├── snapshot.rs · store.rs · compare.rs
│
├── dedup/               # DedupStrategy + apply() (delete/hardlink/symlink)
│
├── scheduler/           # cron (Linux) / ITaskService (Windows)
│   ├── linux.rs · windows.rs
│
└── platform/            # Win32 / POSIX-specific filesystem metrics
    ├── linux.rs · windows.rs
```

## UI architecture (Elm-style)

```
   ┌──────────────────────┐
   │      App state       │  <── App in ui/app.rs
   └─────────┬────────────┘
             │
       view  │  Element<Message>
             ▼
   ┌──────────────────────┐    user interaction
   │ rendered UI (iced)   │ ─────────────────────► Message
   └──────────────────────┘
                              update(state, msg) -> Task<Message>
                              (state mutated in place; Task may emit more messages)
```

- `Message` is intentionally flat — easier to thread through `Task` and async
  results than nested per-screen sub-enums.
- All filesystem work happens off the UI thread inside `ui::tasks::*`, which
  wraps blocking library calls in `tokio::task::spawn_blocking` and returns
  an `iced::Task<Message>`.
- The treemap is a custom `canvas::Program` so it can render thousands of
  tiles without going through the retained widget tree.

## Data flow

```
scanner::walker::scan  ─►  ScanReport  (held by App as Arc<ScanReport>)
                             │
              ┌──────────────┼──────────────┬──────────────┐
              ▼              ▼              ▼              ▼
       analysis::tree   analysis::largest  analysis::    history::
       (hierarchy)      (top-N)            duplicates    snapshot
              │              │              │              │
              ▼              ▼              ▼              ▼
       ui::widgets::      views::largest   views::      history::compare
       treemap_canvas                      duplicates
                                                │
                                                ▼
                                          dedup::apply
```

Reports consume an `Arc<ScanReport>` and write to disk via the chosen
`ExportFormat` (PDF / Excel / HTML / CSV / JSON).

## Target environments

- **Rust toolchain:** pinned to 1.95.0 via `rust-toolchain.toml`.
- **Linux:** `x86_64-unknown-linux-gnu`, glibc, wgpu (Vulkan/GL).
- **Windows:** `x86_64-pc-windows-msvc`, wgpu (D3D12).

CI builds on both targets via `.github/workflows/ci.yml`. The Windows binary
uses `#![windows_subsystem = "windows"]` in release builds so no console
window appears.
