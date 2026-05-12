# FileCanopy

A cross-platform disk-space analyzer written in Rust.

## Features

- One-click visualization of storage allocation down to the file level
- Treemap chart of used disk space (SVG / PNG)
- PDF reports of disk-space usage
- Top-N largest files / "space hog" detection
- Duplicate-file search with optional dedupe (delete, hardlink, symlink)
- Snapshot + diff to compare disk usage over time
- Command-line scriptable, with OS-native task-scheduler integration
- Exports: PDF, Excel, HTML, CSV, JSON
- Source-line counting for code-repo assessments (monolithic-file detection)

## Toolchain

- Rust **1.95.0** (pinned in `rust-toolchain.toml`)
- Targets: `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`

## Build

```sh
cargo build --release
```

## Usage

```sh
filecanopy scan /path/to/dir --output report.json
filecanopy treemap /path/to/dir --output treemap.svg
filecanopy top /path/to/dir -n 50
filecanopy duplicates /path/to/dir --min-size 1048576
filecanopy line-count /path/to/repo --ext rs --ext ts
filecanopy snapshot /path/to/dir --label nightly
filecanopy compare nightly latest
filecanopy export report.json --output report.pdf
filecanopy schedule install --cron "0 2 * * *" --command "scan /data"
```

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the module layout.
