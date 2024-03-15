use core::fmt;
use std::time::Instant;

use common::{
    change_per_second,
    value::{FloatColorStatsDisplay, FloatDisplayPostfix},
};

use crate::process::{
    CommandDisplay, IdHeaderDisplay, IdValueDisplay, ProcessId, TidDisplayOption,
};

#[derive(Debug, Clone)]
pub struct CtxSwitchStats {
    /// voluntary_ctxt_switches
    pub nvcsw: u64,
    /// nonvoluntary_ctxt_switches
    pub nivcsw: u64,
    pub time: Instant,
}

#[derive(Debug, Clone)]
pub struct CtxSwitchStatsHeaderDisplay {
    pub tid: TidDisplayOption,
}
impl fmt::Display for CtxSwitchStatsHeaderDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", IdHeaderDisplay { tid: self.tid })?;
        writeln!(f, "   cswch/s nvcswch/s  Command")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CtxSwitchStatsValueDisplay<'a> {
    pub tid: TidDisplayOption,
    pub id: &'a ProcessId,
    pub prev_stats: &'a CtxSwitchStats,
    pub curr_stats: &'a CtxSwitchStats,
}
impl<'a> fmt::Display for CtxSwitchStatsValueDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = IdValueDisplay {
            process: self.id,
            tid: self.tid,
        };
        write!(f, "{}", display)?;

        let interval = self.curr_stats.time - self.prev_stats.time;

        let nvcsw = change_per_second(
            self.prev_stats.nvcsw.into(),
            self.curr_stats.nvcsw.into(),
            interval,
        )
        .expect("nvcsw");
        let nivcsw = change_per_second(
            self.prev_stats.nivcsw.into(),
            self.curr_stats.nivcsw.into(),
            interval,
        )
        .expect("nivcsw");

        let display = FloatColorStatsDisplay {
            values: &[nvcsw, nivcsw],
            width: 9,
            postfix: FloatDisplayPostfix::Decimals(2),
        };
        write!(f, "{}", display)?;

        let display = CommandDisplay { process: self.id };
        writeln!(f, "{}", display)?;

        Ok(())
    }
}
