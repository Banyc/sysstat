use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::process::{ComponentStats, ProcessId};

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
    pub components: ComponentOptions,
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentOptions {
    pub cpu: bool,
    pub mem: bool,
    pub io: bool,
}

pub struct Stats {
    pub id: ProcessId,
    pub components: ComponentStats,
}

#[derive(Debug, Error)]
pub enum ReadStatsError {
    #[error("No such process: {0}")]
    NoSuchProcess(#[source] std::io::Error),
}

#[derive(Debug, Clone, Copy)]
pub struct ReadTidOptions {
    pub tgid: usize,
}

pub async fn read_task_stats(
    pid: usize,
    components: ComponentOptions,
) -> Result<BTreeMap<usize, Stats>, ReadStatsError> {
    let mut task_stats = BTreeMap::new();
    let tid = ReadTidOptions { tgid: pid }.read_tid().await?;
    for tid in tid {
        let options = ReadStatsOptions {
            id: ProcId {
                pid,
                tid: Some(tid),
            },
            components,
        };
        let stats = options.read_stats().await?;
        task_stats.insert(tid, stats);
    }
    Ok(task_stats)
}

pub struct TaskGroupStats {
    pub pid: usize,
    pub process: Stats,
    pub task: BTreeMap<usize, Stats>,
}
pub async fn read_task_group_stats(
    pid: usize,
    components: ComponentOptions,
    task: bool,
) -> Result<TaskGroupStats, ReadStatsError> {
    let process_options = ReadStatsOptions {
        id: ProcId { pid, tid: None },
        components,
    };
    let process_stats = process_options.read_stats().await?;
    let mut task_stats = BTreeMap::new();
    if task {
        task_stats = read_task_stats(pid, components).await?;
    }
    Ok(TaskGroupStats {
        pid,
        process: process_stats,
        task: task_stats,
    })
}

pub struct ReadPidOptions<'a> {
    pub process_name: &'a str,
}
