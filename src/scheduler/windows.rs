use crate::scheduler::ScheduledTask;
use crate::Result;

/// Install a Windows scheduled task via the ITaskService COM interface.
pub fn install(_task: &ScheduledTask) -> Result<()> {
    // TODO: TaskScheduler COM (windows::Win32::System::TaskScheduler::ITaskService)
    Ok(())
}

pub fn remove(_name: &str) -> Result<()> {
    // TODO
    Ok(())
}

pub fn list() -> Result<Vec<ScheduledTask>> {
    // TODO
    Ok(Vec::new())
}
