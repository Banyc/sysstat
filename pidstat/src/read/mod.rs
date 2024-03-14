use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::process::{Process, ProcessStats};

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;

#[derive(Debug, Clone, Copy)]
pub struct ProcId {
    /// Or TGID if it's in the context of threads instead of processes
    pub pid: usize,
    pub tid: Option<usize>,
}
impl ProcId {
    pub fn path(&self, section: &str) -> PathBuf {
        let pid_path = Path::new("/proc").join(self.pid.to_string());
        let task_path = match self.tid {
            Some(tid) => pid_path.join("task").join(tid.to_string()),
            None => pid_path,
        };
        task_path.join(section)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReadStatsOptions {
    pub id: ProcId,
    pub cpu: bool,
    pub mem: bool,
    pub io: bool,
}

pub struct Stats {
    pub process: Process,
    pub process_stats: ProcessStats,
}

#[derive(Debug, Error)]
pub enum ReadStatsError {
    #[error("No such process: {0}")]
    NoSuchProcess(#[source] std::io::Error),
}
