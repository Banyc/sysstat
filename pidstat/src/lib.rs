use core::fmt;

use cpu::CpuStatsValueDisplay;
use ctx_switch::{CtxSwitchStatsHeaderDisplay, CtxSwitchStatsValueDisplay};
use io::{IoStatsHeaderDisplay, IoStatsValueDisplay};
use mem::{MemStatsHeaderDisplay, MemStatsValueDisplay};
use process::TidDisplayOption;
use read::TaskGroupStats;
use stack::{StackStatsHeaderDisplay, StackStatsValueDisplay};

use crate::cpu::CpuStatsHeaderDisplay;

pub mod cpu;
pub mod ctx_switch;
pub mod io;
pub mod mem;
pub mod process;
pub mod read;
pub mod stack;

pub struct TaskGroupStatsDisplay<'a> {
    pub prev_stats: &'a TaskGroupStats,
    pub curr_stats: &'a TaskGroupStats,
}
impl<'a> fmt::Display for TaskGroupStatsDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tid_display_option = if self.curr_stats.task.is_empty() {
            TidDisplayOption::Pid
        } else {
            TidDisplayOption::Tid
        };

        if self.curr_stats.process.components.cpu.is_some() {
            let header = CpuStatsHeaderDisplay {
                tid: tid_display_option,
            };
            write!(f, "{header}")?;
            let process = CpuStatsValueDisplay {
                tid: tid_display_option,
                id: &self.curr_stats.process.id,
                prev_stats: self.prev_stats.process.components.cpu.as_ref().unwrap(),
                curr_stats: self.curr_stats.process.components.cpu.as_ref().unwrap(),
            };
            write!(f, "{process}")?;
            for (tid, stats) in &self.curr_stats.task {
                let Some(prev_stats) = self.prev_stats.task.get(tid) else {
                    continue;
                };
                let task = CpuStatsValueDisplay {
                    tid: tid_display_option,
                    id: &stats.id,
                    prev_stats: prev_stats.components.cpu.as_ref().unwrap(),
                    curr_stats: stats.components.cpu.as_ref().unwrap(),
                };
                write!(f, "{task}")?;
            }
        }
        if self.curr_stats.process.components.mem.is_some() {
            let header = MemStatsHeaderDisplay {
                tid: tid_display_option,
            };
            write!(f, "{header}")?;
            let process = MemStatsValueDisplay {
                tid: tid_display_option,
                id: &self.curr_stats.process.id,
                prev_stats: self.prev_stats.process.components.mem.as_ref().unwrap(),
                curr_stats: self.curr_stats.process.components.mem.as_ref().unwrap(),
            };
            write!(f, "{process}")?;
            for (tid, stats) in &self.curr_stats.task {
                let Some(prev_stats) = self.prev_stats.task.get(tid) else {
                    continue;
                };
                let task = MemStatsValueDisplay {
                    tid: tid_display_option,
                    id: &stats.id,
                    prev_stats: prev_stats.components.mem.as_ref().unwrap(),
                    curr_stats: stats.components.mem.as_ref().unwrap(),
                };
                write!(f, "{task}")?;
            }
        }
        if self.curr_stats.process.components.stack.is_some() {
            let header = StackStatsHeaderDisplay {
                tid: tid_display_option,
            };
            write!(f, "{header}")?;
            let process = StackStatsValueDisplay {
                tid: tid_display_option,
                id: &self.curr_stats.process.id,
                curr_stats: self.curr_stats.process.components.stack.as_ref().unwrap(),
            };
            write!(f, "{process}")?;
            for stats in self.curr_stats.task.values() {
                let task = StackStatsValueDisplay {
                    tid: tid_display_option,
                    id: &stats.id,
                    curr_stats: stats.components.stack.as_ref().unwrap(),
                };
                write!(f, "{task}")?;
            }
        }
        if self.curr_stats.process.components.io.is_some() {
            let header = IoStatsHeaderDisplay {
                tid: tid_display_option,
            };
            write!(f, "{header}")?;
            let process = IoStatsValueDisplay {
                tid: tid_display_option,
                id: &self.curr_stats.process.id,
                prev_stats: self.prev_stats.process.components.io.as_ref().unwrap(),
                curr_stats: self.curr_stats.process.components.io.as_ref().unwrap(),
            };
            write!(f, "{process}")?;
            for (tid, stats) in &self.curr_stats.task {
                let Some(prev_stats) = self.prev_stats.task.get(tid) else {
                    continue;
                };
                let task = IoStatsValueDisplay {
                    tid: tid_display_option,
                    id: &stats.id,
                    prev_stats: prev_stats.components.io.as_ref().unwrap(),
                    curr_stats: stats.components.io.as_ref().unwrap(),
                };
                write!(f, "{task}")?;
            }
        }
        if self.curr_stats.process.components.ctx_switch.is_some() {
            let header = CtxSwitchStatsHeaderDisplay {
                tid: tid_display_option,
            };
            write!(f, "{header}")?;
            let process = CtxSwitchStatsValueDisplay {
                tid: tid_display_option,
                id: &self.curr_stats.process.id,
                prev_stats: self
                    .prev_stats
                    .process
                    .components
                    .ctx_switch
                    .as_ref()
                    .unwrap(),
                curr_stats: self
                    .curr_stats
                    .process
                    .components
                    .ctx_switch
                    .as_ref()
                    .unwrap(),
            };
            write!(f, "{process}")?;
            for (tid, stats) in &self.curr_stats.task {
                let Some(prev_stats) = self.prev_stats.task.get(tid) else {
                    continue;
                };
                let task = CtxSwitchStatsValueDisplay {
                    tid: tid_display_option,
                    id: &stats.id,
                    prev_stats: prev_stats.components.ctx_switch.as_ref().unwrap(),
                    curr_stats: stats.components.ctx_switch.as_ref().unwrap(),
                };
                write!(f, "{task}")?;
            }
        }

        Ok(())
    }
}
