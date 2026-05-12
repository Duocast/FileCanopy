# FileCanopy

<p align="center">
  <img src="Assets/FileCanopyLogov1-transparent.png" alt="FileCanopy logo" width="320">
</p>

A cross-platform GUI disk-space analyzer written in Rust with **iced 0.14**.

## Features

- One-click scan of any folder with a live progress view
- Treemap visualization (custom `canvas::Program`) of disk usage
- Top-N "space hog" finder
- Duplicate-file search with in-app dedupe (dry-run / delete / hardlink / symlink)
- Snapshot + diff view to compare disk usage over time
- Source-line counting for code-repo assessments (monolithic-file flagging)
- Exports: PDF, Excel, HTML, CSV, JSON
- Built-in scheduler that installs cron entries (Linux) or Task Scheduler
  entries (Windows) for recurring background scans

## Toolchain

- Rust **1.95.0** (pinned in `rust-toolchain.toml`)
- iced **0.14.0** (wgpu backend)
- Targets: `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`

## Build & run

```sh
cargo run --release
```

The release binary on Windows is built with `windows_subsystem = "windows"`,
so it launches without a console window.

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the module layout and a
brief tour of the Elm-style UI architecture.
