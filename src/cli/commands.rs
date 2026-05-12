//! Subcommand dispatch — thin glue that wires CLI args to library modules.

use crate::cli::args::{
    Cli, Command, CompareArgs, DedupArgs, DuplicatesArgs, ExportArgs, LineCountArgs, ScanArgs,
    ScheduleAction, ScheduleArgs, SnapshotArgs, TopArgs, TreemapArgs,
};
use crate::Result;

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Scan(args) => run_scan(args),
        Command::Treemap(args) => run_treemap(args),
        Command::Top(args) => run_top(args),
        Command::Duplicates(args) => run_duplicates(args),
        Command::Dedup(args) => run_dedup(args),
        Command::Snapshot(args) => run_snapshot(args),
        Command::Compare(args) => run_compare(args),
        Command::LineCount(args) => run_line_count(args),
        Command::Schedule(args) => run_schedule(args),
        Command::Export(args) => run_export(args),
    }
}

fn run_scan(_args: ScanArgs) -> Result<()> {
    // TODO: scanner::walker::scan(...) -> analysis::tree::build(...)
    Ok(())
}

fn run_treemap(_args: TreemapArgs) -> Result<()> {
    // TODO: visualization::treemap::render(...)
    Ok(())
}

fn run_top(_args: TopArgs) -> Result<()> {
    // TODO: analysis::largest::top(...)
    Ok(())
}

fn run_duplicates(_args: DuplicatesArgs) -> Result<()> {
    // TODO: analysis::duplicates::find(...)
    Ok(())
}

fn run_dedup(_args: DedupArgs) -> Result<()> {
    // TODO: dedup::apply(...)
    Ok(())
}

fn run_snapshot(_args: SnapshotArgs) -> Result<()> {
    // TODO: history::snapshot::take(...)
    Ok(())
}

fn run_compare(_args: CompareArgs) -> Result<()> {
    // TODO: history::compare::diff(...)
    Ok(())
}

fn run_line_count(_args: LineCountArgs) -> Result<()> {
    // TODO: scanner::line_counter::count(...)
    Ok(())
}

fn run_schedule(args: ScheduleArgs) -> Result<()> {
    match args.action {
        ScheduleAction::Install { .. } => { /* TODO */ }
        ScheduleAction::Remove { .. } => { /* TODO */ }
        ScheduleAction::List => { /* TODO */ }
    }
    Ok(())
}

fn run_export(_args: ExportArgs) -> Result<()> {
    // TODO: reports::<format>::export(...)
    Ok(())
}
