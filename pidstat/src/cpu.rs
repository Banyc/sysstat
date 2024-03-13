use core::fmt;
use std::time::Instant;

use common::{
    change_per_second,
    value::{item_name_color, normal_color, PercentageColorStatsDisplay, PercentageDisplayLimit},
};
use strict_num::PositiveF64;

use crate::process::{CommandDisplay, IdHeaderDisplay, IdValueDisplay, Process, TidDisplayOption};

#[derive(Debug, Clone)]
pub struct CpuStats {
    /// In ticks
    ///
    /// Not including guest time
    pub user_time: u64,
    /// In ticks
    pub system_time: u64,
    /// In ticks
    pub guest_time: u64,
    /// In ticks
    pub wait_time: u64,
    pub time: Instant,
    pub processor: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CpuStatsHeaderDisplay {
    pub tid: TidDisplayOption,
}
impl fmt::Display for CpuStatsHeaderDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", IdHeaderDisplay { tid: self.tid })?;
        writeln!(f, "    %usr %system  %guest   %wait    %CPU   CPU  Command")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CpuStatsValueDisplay<'a> {
    pub tid: TidDisplayOption,
    pub process: &'a Process,
    pub prev_stats: &'a CpuStats,
    pub curr_stats: &'a CpuStats,
}
impl<'a> fmt::Display for CpuStatsValueDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = IdValueDisplay {
            process: self.process,
            tid: self.tid,
        };
        write!(f, "{}", display)?;

        let interval = self.curr_stats.time - self.prev_stats.time;
        let clock_ticks_per_second = rustix::param::clock_ticks_per_second();

        let usr = change_per_second(
            self.prev_stats.user_time.into(),
            self.curr_stats.user_time.into(),
            interval,
        )
        .unwrap()
        .get()
            / clock_ticks_per_second as f64;
        let usr = PositiveF64::new(usr).unwrap();

        let system = change_per_second(
            self.prev_stats.system_time.into(),
            self.curr_stats.system_time.into(),
            interval,
        )
        .unwrap()
        .get()
            / clock_ticks_per_second as f64;
        let system = PositiveF64::new(system).unwrap();

        let guest = change_per_second(
            self.prev_stats.guest_time.into(),
            self.curr_stats.guest_time.into(),
            interval,
        )
        .unwrap()
        .get()
            / clock_ticks_per_second as f64;
        let guest = PositiveF64::new(guest).unwrap();

        let wait = change_per_second(
            self.prev_stats.wait_time.into(),
            self.curr_stats.wait_time.into(),
            interval,
        )
        .unwrap()
        .get()
            / clock_ticks_per_second as f64;
        let wait = PositiveF64::new(wait).unwrap();

        let cpu = change_per_second(
            (self.prev_stats.user_time + self.prev_stats.system_time + self.prev_stats.wait_time)
                .into(),
            (self.curr_stats.user_time + self.curr_stats.system_time + self.curr_stats.wait_time)
                .into(),
            interval,
        )
        .unwrap()
        .get()
            / clock_ticks_per_second as f64;
        let cpu = PositiveF64::new(cpu).unwrap();

        let display = PercentageColorStatsDisplay {
            values: &[usr, system, guest, wait, cpu],
            width: 7,
            decimals: 2,
            limit: PercentageDisplayLimit::ExtremeHigh,
        };
        write!(f, "{}", display)?;

        if let Some(processor) = self.curr_stats.processor {
            write!(
                f,
                "{start}   {value:3}{end}",
                start = item_name_color(),
                value = processor,
                end = normal_color()
            )?;
        } else {
            write!(
                f,
                "{start}   {value:3}{end}",
                start = item_name_color(),
                value = '-',
                end = normal_color()
            )?;
        }

        let display = CommandDisplay {
            process: self.process,
        };
        writeln!(f, "{}", display)?;

        Ok(())
    }
}
