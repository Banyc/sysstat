use core::fmt;
use std::time::Instant;

use common::{
    change_per_second,
    value::{FloatColorStatsDisplay, FloatDisplayPostfix, U64ColorStatsDisplay},
};

use crate::process::{CommandDisplay, IdHeaderDisplay, IdValueDisplay, Process, TidDisplayOption};

#[derive(Debug, Clone)]
pub struct IoStats {
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub cancelled_write_bytes: u64,
    pub blkio_swapin_delays: u64,
    pub time: Instant,
}

#[derive(Debug, Clone)]
pub struct IoStatsHeaderDisplay {
    pub tid: TidDisplayOption,
}
impl fmt::Display for IoStatsHeaderDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", IdHeaderDisplay { tid: self.tid })?;
        writeln!(f, "   kB_rd/s   kB_wr/s kB_ccwr/s iodelay  Command")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct IoStatsValueDisplay<'a> {
    pub tid: TidDisplayOption,
    // pub average_stats: bool,
    pub process: &'a Process,
    pub prev_stats: &'a IoStats,
    pub curr_stats: &'a IoStats,
}
impl<'a> fmt::Display for IoStatsValueDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = IdValueDisplay {
            process: self.process,
            tid: self.tid,
        };
        write!(f, "{}", display)?;

        let interval = self.curr_stats.time - self.prev_stats.time;

        let r_bytes = change_per_second(
            self.prev_stats.read_bytes.into(),
            self.curr_stats.read_bytes.into(),
            interval,
        )
        .unwrap();
        let w_bytes = change_per_second(
            self.prev_stats.write_bytes.into(),
            self.curr_stats.write_bytes.into(),
            interval,
        )
        .unwrap();
        let c_bytes = change_per_second(
            self.prev_stats.cancelled_write_bytes.into(),
            self.curr_stats.cancelled_write_bytes.into(),
            interval,
        )
        .unwrap();

        let display = FloatColorStatsDisplay {
            values: &[r_bytes, w_bytes, c_bytes],
            width: 9,
            postfix: FloatDisplayPostfix::Decimals(2),
        };
        write!(f, "{}", display)?;

        let io_delay = self.curr_stats.blkio_swapin_delays - self.prev_stats.blkio_swapin_delays;
        let display = U64ColorStatsDisplay {
            values: &[io_delay],
            width: 7,
            unit: None,
        };
        write!(f, "{}", display)?;

        let display = CommandDisplay {
            process: self.process,
        };
        writeln!(f, "{}", display)?;

        Ok(())
    }
}
