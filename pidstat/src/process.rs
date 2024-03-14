use core::fmt;

use common::value::{int_stat_color, item_name_color, normal_color, zero_int_stat_color};

use crate::{cpu::CpuStats, io::IoStats, mem::MemStats, read::ProcId};

#[derive(Debug, Clone)]
pub struct ProcessId {
    pub uid: usize,
    pub proc_id: ProcId,
    pub command: String,
    // pub delay_asum_count: usize,
}

#[derive(Debug, Clone)]
pub struct ComponentStats {
    pub cpu: Option<CpuStats>,
    pub mem: Option<MemStats>,
    pub io: Option<IoStats>,
}

pub struct IdHeaderDisplay {
    pub tid: TidDisplayOption,
}
impl fmt::Display for IdHeaderDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "   UID")?;
        match self.tid {
            TidDisplayOption::Tid => write!(f, "      TGID       TID"),
            TidDisplayOption::Pid => write!(f, "       PID"),
        }?;
        Ok(())
    }
}

pub struct IdValueDisplay<'a> {
    pub process: &'a ProcessId,
    pub tid: TidDisplayOption,
}
impl<'a> fmt::Display for IdValueDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = item_name_color();
        let end = normal_color();
        write!(f, "{start} {uid:5}{end}", uid = self.process.uid)?;
        write!(f, "{start}")?;
        match self.tid {
            TidDisplayOption::Tid => match self.process.proc_id.tid {
                Some(tid) => write!(f, "         - {tid:9}")?,
                None => write!(f, " {tgid:9}         -", tgid = self.process.proc_id.pid)?,
            },
            TidDisplayOption::Pid => write!(f, " {pid:9}", pid = self.process.proc_id.pid)?,
        }
        write!(f, "{end}")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TidDisplayOption {
    Tid,
    Pid,
}

pub struct CommandDisplay<'a> {
    pub process: &'a ProcessId,
}
impl<'a> fmt::Display for CommandDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.process.proc_id.tid {
            Some(_) => write!(
                f,
                "{start}  |__{value}{end}",
                start = zero_int_stat_color(),
                value = self.process.command,
                end = normal_color()
            )?,
            None => write!(
                f,
                "{start}  {value}{end}",
                start = int_stat_color(),
                value = self.process.command,
                end = normal_color()
            )?,
        }
        Ok(())
    }
}
