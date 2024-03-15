use core::fmt;
use std::time::Instant;

use common::value::{MemoryUnit, U64ColorStatsDisplay};

use crate::process::{
    CommandDisplay, IdHeaderDisplay, IdValueDisplay, ProcessId, TidDisplayOption,
};

#[derive(Debug, Clone)]
pub struct StackStats {
    /// The amount of memory in kilobytes reserved for the task as stack, but not necessarily used
    pub stk_size: u64,
    /// The amount of memory in kilobytes used as stack, referenced by the task
    pub stk_ref: u64,
    pub time: Instant,
}

#[derive(Debug, Clone)]
pub struct StackStatsHeaderDisplay {
    pub tid: TidDisplayOption,
}
impl fmt::Display for StackStatsHeaderDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", IdHeaderDisplay { tid: self.tid })?;
        writeln!(f, " StkSize  StkRef  Command")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StackStatsValueDisplay<'a> {
    pub tid: TidDisplayOption,
    pub id: &'a ProcessId,
    pub curr_stats: &'a StackStats,
}
impl<'a> fmt::Display for StackStatsValueDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = IdValueDisplay {
            process: self.id,
            tid: self.tid,
        };
        write!(f, "{}", display)?;

        let display = U64ColorStatsDisplay {
            values: &[self.curr_stats.stk_size, self.curr_stats.stk_ref],
            width: 7,
            unit: Some(MemoryUnit::Kilobytes),
        };
        write!(f, "{}", display)?;

        let display = CommandDisplay { process: self.id };
        writeln!(f, "{}", display)?;

        Ok(())
    }
}
