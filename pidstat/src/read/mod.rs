use std::path::{Path, PathBuf};

use crate::process::{Process, ProcessStats};

pub mod linux;

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
    pub io: bool,
    pub mem: bool,
    pub cpu: bool,
}

pub struct Stats {
    pub process: Process,
    pub process_stats: ProcessStats,
}
