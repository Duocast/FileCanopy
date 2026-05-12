use crate::scheduler::ScheduledTask;
use crate::Result;

/// Install a cron entry. We tag managed lines with `# filecanopy:<name>` so
/// they can be safely removed later.
pub fn install(_task: &ScheduledTask) -> Result<()> {
    // TODO: read current crontab, append tagged entry, write back
    Ok(())
}

pub fn remove(_name: &str) -> Result<()> {
    // TODO: filter out tagged lines and write back
    Ok(())
}

pub fn list() -> Result<Vec<ScheduledTask>> {
    // TODO: parse tagged lines out of crontab
    Ok(Vec::new())
}
