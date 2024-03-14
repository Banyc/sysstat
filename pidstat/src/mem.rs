use core::fmt;
use std::time::Instant;

use common::{
    change_per_second,
    value::{
        FloatColorStatsDisplay, FloatDisplayPostfix, MemoryUnit, PercentageColorStatsDisplay,
        PercentageDisplayLimit, U64ColorStatsDisplay,
    },
};
use strict_num::PositiveF64;

use crate::process::{
    CommandDisplay, IdHeaderDisplay, IdValueDisplay, ProcessId, TidDisplayOption,
};

#[derive(Debug, Clone)]
pub struct MemStats {
    pub minflt: u64,
    pub majflt: u64,
    /// In kB
    pub vsz: u64,
    /// In kB
    pub rss: u64,
    /// In kB
    pub tot_mem: u64,
    pub time: Instant,
}

#[derive(Debug, Clone)]
pub struct MemStatsHeaderDisplay {
    pub tid: TidDisplayOption,
}
impl fmt::Display for MemStatsHeaderDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", IdHeaderDisplay { tid: self.tid })?;
        writeln!(f, "  minflt/s  majflt/s     VSZ     RSS   %MEM  Command")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MemStatsValueDisplay<'a> {
    pub tid: TidDisplayOption,
    pub id: &'a ProcessId,
    pub prev_stats: &'a MemStats,
    pub curr_stats: &'a MemStats,
}
impl<'a> fmt::Display for MemStatsValueDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = IdValueDisplay {
            process: self.id,
            tid: self.tid,
        };
        write!(f, "{}", display)?;

        let interval = self.curr_stats.time - self.prev_stats.time;

        let minflt = change_per_second(
            self.prev_stats.minflt.into(),
            self.curr_stats.minflt.into(),
            interval,
        )
        .expect("minflt");
        let majflt = change_per_second(
            self.prev_stats.majflt.into(),
            self.curr_stats.majflt.into(),
            interval,
        )
        .expect("majflt");
        let display = FloatColorStatsDisplay {
            values: &[minflt, majflt],
            width: 9,
            postfix: FloatDisplayPostfix::Decimals(2),
        };
        write!(f, "{}", display)?;

        let display = U64ColorStatsDisplay {
            values: &[self.curr_stats.vsz, self.curr_stats.rss],
            width: 7,
            unit: Some(MemoryUnit::Kilobytes),
        };
        write!(f, "{}", display)?;

        let mem = PositiveF64::new(self.curr_stats.rss as f64 / self.curr_stats.tot_mem as f64)
            .expect("mem");
        let display = PercentageColorStatsDisplay {
            values: &[mem],
            width: 6,
            decimals: 2,
            limit: PercentageDisplayLimit::ExtremeHigh,
        };
        write!(f, "{}", display)?;

        let display = CommandDisplay { process: self.id };
        writeln!(f, "{}", display)?;

        Ok(())
    }
}
