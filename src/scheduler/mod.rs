//! Cross-platform installation of recurring filecanopy tasks.
//!
//! On Linux/macOS we shell out to `crontab`. On Windows we use the
//! ITaskService COM API via the `windows` crate.

#[cfg(unix)]
pub mod linux;
#[cfg(windows)]
pub mod windows;

use crate::Result;

#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub name: String,
    pub cron: String,
    pub command: String,
}

pub fn install(_task: &ScheduledTask) -> Result<()> {
    #[cfg(unix)]
    {
        return linux::install(_task);
    }
    #[cfg(windows)]
    {
        return windows::install(_task);
    }
    #[allow(unreachable_code)]
    Err(crate::Error::Scheduler("unsupported platform".into()))
}

pub fn remove(_name: &str) -> Result<()> {
    #[cfg(unix)]
    {
        return linux::remove(_name);
    }
    #[cfg(windows)]
    {
        return windows::remove(_name);
    }
    #[allow(unreachable_code)]
    Err(crate::Error::Scheduler("unsupported platform".into()))
}

pub fn list() -> Result<Vec<ScheduledTask>> {
    #[cfg(unix)]
    {
        return linux::list();
    }
    #[cfg(windows)]
    {
        return windows::list();
    }
    #[allow(unreachable_code)]
    Ok(Vec::new())
}
