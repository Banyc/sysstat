use std::{num::NonZeroU32, path::Path, time::Instant};

use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncReadExt};

use crate::{
    cpu::CpuStats,
    io::IoStats,
    mem::MemStats,
    process::{Process, ProcessStats},
};

use super::{ProcId, ReadStatsOptions, Stats};

impl ReadStatsOptions {
    pub async fn read(&self) -> Result<Stats, ReadStatsError> {
        let now = Instant::now();
        let proc_stat = read_proc_stat(self.id).await?;
        let proc_status = read_proc_status(self.id).await?;
        let process = Process {
            uid: proc_status.uid,
            proc_id: self.id,
            command: proc_stat.command,
        };

        let mut cpu = None;
        if self.cpu {
            let clock_ticks_per_second = rustix::param::clock_ticks_per_second();
            let proc_sched = read_proc_sched(self.id).await?;
            let wait_time = clock_ticks_per_second * proc_sched.wait_time / 1_000_000_000;
            cpu = Some(CpuStats {
                user_time: proc_stat.utime.saturating_sub(proc_stat.guest_time),
                system_time: proc_stat.stime,
                guest_time: proc_stat.guest_time,
                wait_time,
                time: now,
                processor: proc_stat.processor,
                clock_ticks_per_second,
            })
        }
        let mut mem = None;
        if self.mem {
            let mem_info = read_proc_mem_info().await?;
            let page_size = rustix::param::page_size();
            mem = Some(MemStats {
                minflt: proc_stat.minflt,
                majflt: proc_stat.majflt,
                vsz: proc_stat.vsize / 1024,
                rss: proc_stat.rss * u64::try_from(page_size).expect("page_size") / 1024,
                tot_mem: mem_info.mem_total,
                time: now,
            })
        }
        let mut io = None;
        if self.io {
            let proc_io = read_proc_io(self.id).await?;
            io = Some(IoStats {
                read_bytes: proc_io.read_bytes,
                write_bytes: proc_io.write_bytes,
                cancelled_write_bytes: proc_io.cancelled_write_bytes,
                blkio_swapin_delays: proc_stat.delayacct_blkio_ticks,
                time: now,
            });
        }
        let process_stats = ProcessStats { cpu, mem, io };

        Ok(Stats {
            process,
            process_stats,
        })
    }
}

/// Ref: <https://man7.org/linux/man-pages/man5/proc.5.html>
#[derive(Debug, Clone)]
pub struct ProcStatus {
    /// Real, effective, saved set, and filesystem UIDs
    pub uid: usize,
    /// Number of threads in process containing this thread
    pub threads: usize,
    /// Number of voluntary context switches
    pub voluntary_ctxt_switches: usize,
    /// Number of involuntary context switches
    pub nonvoluntary_ctxt_switches: usize,
}
pub async fn read_proc_status(id: ProcId) -> Result<ProcStatus, ReadStatsError> {
    let path = id.path("status");
    let file = tokio::fs::File::options()
        .read(true)
        .open(path)
        .await
        .map_err(ReadStatsError::NoSuchProcess)?;

    let mut uid = None;
    let mut threads = None;
    let mut voluntary_ctxt_switches = None;
    let mut nonvoluntary_ctxt_switches = None;
    let buf = tokio::io::BufReader::new(file);
    let mut lines = buf.lines();
    while let Some(line) = lines.next_line().await.expect("UTF-8") {
        const UID: &str = "Uid:";
        if line.starts_with(UID) {
            let remaining = line.chars().skip(UID.len()).skip(1).collect::<String>();
            uid = Some(
                remaining
                    .trim_start()
                    .split_once('\t')
                    .expect("uid")
                    .0
                    .parse::<usize>()
                    .expect("uid"),
            );
        }
        const THREADS: &str = "Threads:";
        if line.starts_with(THREADS) {
            threads = Some(
                line.chars()
                    .skip(THREADS.len())
                    .collect::<String>()
                    .trim_start()
                    .parse::<usize>()
                    .expect("threads"),
            );
        }
        const VOLUNTARY_CTXT_SWITCHES: &str = "voluntary_ctxt_switches:";
        if line.starts_with(VOLUNTARY_CTXT_SWITCHES) {
            voluntary_ctxt_switches = Some(
                line.chars()
                    .skip(VOLUNTARY_CTXT_SWITCHES.len())
                    .collect::<String>()
                    .trim_start()
                    .parse::<usize>()
                    .expect("voluntary_ctxt_switches"),
            );
        }
        const NONVOLUNTARY_CTXT_SWITCHES: &str = "nonvoluntary_ctxt_switches:";
        if line.starts_with(NONVOLUNTARY_CTXT_SWITCHES) {
            nonvoluntary_ctxt_switches = Some(
                line.chars()
                    .skip(NONVOLUNTARY_CTXT_SWITCHES.len())
                    .collect::<String>()
                    .trim_start()
                    .parse::<usize>()
                    .expect("nonvoluntary_ctxt_switches"),
            );
        }
    }
    Ok(ProcStatus {
        uid: uid.expect("uid"),
        threads: threads.expect("threads"),
        voluntary_ctxt_switches: voluntary_ctxt_switches.expect("voluntary_ctxt_switches"),
        nonvoluntary_ctxt_switches: nonvoluntary_ctxt_switches.expect("nonvoluntary_ctxt_switches"),
    })
}

/// Ref: <https://man7.org/linux/man-pages/man5/proc.5.html>
#[derive(Debug, Clone)]
pub struct ProcStat {
    pub command: String,
    pub state: ProcState,
    /// The PID of the parent of this process
    pub ppid: u32,
    /// The process group ID of the process
    pub pgrp: u32,
    /// The session ID of the process
    pub session: u32,
    /// The controlling terminal of the process
    pub tty_nr: u32,
    /// The ID of the foreground process group of the controlling terminal of the process
    pub tpgid: Option<u32>,
    /// The kernel flags word of the process.
    /// For bit meanings, see the `PF_*` defines in the Linux kernel source file `include/linux/sched.h`.
    /// Details depend on the kernel version.
    pub flags: u32,
    /// The number of minor faults the process has made which have not required loading a memory page from disk
    pub minflt: u64,
    /// The number of minor faults that the process's waited-for children have made
    pub cminflt: u64,
    /// The number of major faults the process has made which have required loading a memory page from disk
    pub majflt: u64,
    /// The number of major faults that the process's waited-for children have made
    pub cmajflt: u64,
    /// Amount of time that this process has been scheduled in user mode, measured in clock ticks (divide by `sysconf(_SC_CLK_TCK)`).
    /// This includes guest time, `guest_time` (time spent running a virtual CPU, see below), so that applications that are not aware of the guest time field do not lose that time from their calculations.
    pub utime: u64,
    /// Amount of time that this process has been scheduled in kernel mode, measured in clock ticks (divide by `sysconf(_SC_CLK_TCK)`)
    pub stime: u64,
    /// Amount of time that this process's waited-for children have been scheduled in user mode, measured in clock ticks (divide by `sysconf(_SC_CLK_TCK)`). (See also times(2).)
    /// This includes guest time, `cguest_time` (time spent running a virtual CPU, see below).
    pub cutime: Option<u64>,
    /// Amount of time that this process's waited-for children have been scheduled in kernel mode, measured in clock ticks (divide by sysconf(_SC_CLK_TCK))
    pub cstime: Option<u64>,
    /// For processes running a real-time scheduling policy (`policy` below; see `sched_setscheduler(2)`), this is the negated scheduling priority, minus one; that is, a number in the range -2 to -100, corresponding to real-time priorities 1 to 99.
    /// For processes running under a non-real-time scheduling policy, this is the raw nice value (`setpriority(2)`) as represented in the kernel.
    /// The kernel stores nice values as numbers in the range 0 (high) to 39 (low), corresponding to the user-visible nice range of -20 to 19.
    pub priority: i64,
    /// The nice value (see `setpriority(2)`), a value in the range 19 (low priority) to -20 (high priority)
    pub nice: i64,
    /// Number of threads in this process
    pub num_threads: u64,
    /// The time the process started after system boot
    pub starttime: u64,
    /// Virtual memory size in bytes
    pub vsize: u64,
    /// Resident Set Size: number of pages the process has in real memory.
    /// This is just the pages which count toward text, data, or stack space.
    /// This does not include pages which have not been demand-loaded in, or which are swapped out.
    /// This value is inaccurate; see `/proc/pid/statm` below.
    pub rss: u64,
    /// Current soft limit in bytes on the rss of the process; see the description of `RLIMIT_RSS` in `getrlimit(2)`
    pub rsslim: u64,
    /// The address above which program text can run
    pub startcode: Option<NonZeroU32>,
    /// The address below which program text can run
    pub endcode: Option<NonZeroU32>,
    /// The address of the start (i.e., bottom) of the stack
    pub startstack: Option<NonZeroU32>,
    /// The current value of ESP (stack pointer), as found in the kernel stack page for the process
    pub kstkesp: Option<NonZeroU32>,
    /// The current EIP (instruction pointer)
    pub kstkeip: Option<NonZeroU32>,
    /// This is the "channel" in which the process is waiting.
    /// It is the address of a location in the kernel where the process is sleeping.
    /// The corresponding symbolic name can be found in `/proc/pid/wchan`.
    pub wchan: Option<NonZeroU32>,
    /// Signal to be sent to parent when we die
    pub exit_signal: Option<u32>,
    /// CPU number last executed on
    pub processor: Option<u32>,
    /// Real-time scheduling priority, a number in the range 1 to 99 for processes scheduled under a real-time policy, or 0, for non-real-time processes (see `sched_setscheduler(2)`)
    pub rt_priority: u32,
    /// Scheduling policy (see `sched_setscheduler(2)`).
    /// Decode using the `SCHED_*` constants in `linux/sched.h`.
    pub policy: u32,
    /// Aggregated block I/O delays, measured in clock ticks (centiseconds)
    pub delayacct_blkio_ticks: u64,
    /// Guest time of the process (time spent running a virtual CPU for a guest operating system), measured in clock ticks (divide by `sysconf(_SC_CLK_TCK)`)
    pub guest_time: u64,
    /// Guest time of the process's children, measured in clock ticks (divide by `sysconf(_SC_CLK_TCK)`)
    pub cguest_time: Option<u64>,
    /// Address above which program initialized and uninitialized (BSS) data are placed
    pub start_data: Option<NonZeroU32>,
    /// Address below which program initialized and uninitialized (BSS) data are placed
    pub end_data: Option<NonZeroU32>,
    /// Address above which program heap can be expanded with `brk(2)`
    pub start_brk: Option<NonZeroU32>,
    /// Address above which program command-line arguments (`argv`) are placed
    pub arg_start: Option<NonZeroU32>,
    /// Address below program command-line arguments (`argv`) are placed
    pub arg_end: Option<NonZeroU32>,
    /// Address above which program environment is placed
    pub env_start: Option<NonZeroU32>,
    /// Address below which program environment is placed
    pub env_end: Option<NonZeroU32>,
    /// The thread's exit status in the form reported by `waitpid(2)`
    pub exit_code: Option<NonZeroU32>,
}
pub async fn read_proc_stat(id: ProcId) -> Result<ProcStat, ReadStatsError> {
    let path = id.path("stat");
    let mut file = tokio::fs::File::options()
        .read(true)
        .open(path)
        .await
        .map_err(ReadStatsError::NoSuchProcess)?;
    let mut text = String::new();
    file.read_to_string(&mut text).await.expect("UTF-8");

    let command_start = text.find('(').expect("(") + 1;
    let command_end = text.find(')').expect(")");
    let command_len = command_end - command_start;
    let command = text
        .chars()
        .skip(command_start)
        .take(command_len)
        .collect::<String>();

    let remaining = text.chars().skip(command_end + 2).collect::<String>();
    let mut items = remaining.split(' ');

    let state = items.next().expect("state");
    let state = match state {
        "R" => ProcState::Running,
        "S" => ProcState::Sleeping,
        "D" => ProcState::Waiting,
        "Z" => ProcState::Zombie,
        "T" => ProcState::Stopped,
        "X" => ProcState::Dead,
        "I" => ProcState::Idle,
        _ => panic!("unknown state"),
    };
    let ppid = items.next().expect("ppid").parse::<u32>().expect("ppid");
    let pgrp = items.next().expect("pgrp").parse::<u32>().expect("pgrp");
    let session = items
        .next()
        .expect("session")
        .parse::<u32>()
        .expect("session");
    let tty_nr = items
        .next()
        .expect("tty_nr")
        .parse::<u32>()
        .expect("tty_nr");
    let tpgid = items.next().expect("tpgid").parse::<u32>().ok();
    let flags = items.next().expect("flags").parse::<u32>().expect("flags");
    let minflt = items
        .next()
        .expect("minflt")
        .parse::<u64>()
        .expect("minflt");
    let cminflt = items
        .next()
        .expect("cminflt")
        .parse::<u64>()
        .expect("cminflt");
    let majflt = items
        .next()
        .expect("majflt")
        .parse::<u64>()
        .expect("majflt");
    let cmajflt = items
        .next()
        .expect("cmajflt")
        .parse::<u64>()
        .expect("cmajflt");
    let utime = items.next().expect("utime").parse::<u64>().expect("utime");
    let stime = items.next().expect("stime").parse::<u64>().expect("stime");
    let cutime = items.next().expect("cutime").parse::<u64>().ok();
    let cstime = items.next().expect("cstime").parse::<u64>().ok();
    let priority = items
        .next()
        .expect("priority")
        .parse::<i64>()
        .expect("priority");
    let nice = items.next().expect("nice").parse::<i64>().expect("nice");
    let num_threads = items
        .next()
        .expect("num_threads")
        .parse::<u64>()
        .expect("num_threads");
    let _itrealvalue = items
        .next()
        .expect("itrealvalue")
        .parse::<u64>()
        .expect("itrealvalue");
    let starttime = items
        .next()
        .expect("starttime")
        .parse::<u64>()
        .expect("starttime");
    let vsize = items.next().expect("vsize").parse::<u64>().expect("vsize");
    let rss = items.next().expect("rss").parse::<u64>().expect("rss");
    let rsslim = items
        .next()
        .expect("rsslim")
        .parse::<u64>()
        .expect("rsslim");
    let startcode = items.next().expect("startcode").parse::<NonZeroU32>().ok();
    let endcode = items.next().expect("endcode").parse::<NonZeroU32>().ok();
    let startstack = items.next().expect("startstack").parse::<NonZeroU32>().ok();
    let kstkesp = items.next().expect("kstkesp").parse::<NonZeroU32>().ok();
    let kstkeip = items.next().expect("kstkeip").parse::<NonZeroU32>().ok();
    let _signal = items
        .next()
        .expect("signal")
        .parse::<u64>()
        .expect("signal");
    let _blocked = items
        .next()
        .expect("blocked")
        .parse::<u64>()
        .expect("blocked");
    let _sigignore = items
        .next()
        .expect("sigignore")
        .parse::<u64>()
        .expect("sigignore");
    let _sigcatch = items
        .next()
        .expect("sigcatch")
        .parse::<u64>()
        .expect("sigcatch");
    let wchan = items.next().expect("wchan").parse::<NonZeroU32>().ok();
    let _nswap = items.next().expect("nswap").parse::<u64>().expect("nswap");
    let _cnswap = items
        .next()
        .expect("cnswap")
        .parse::<u64>()
        .expect("cnswap");
    let exit_signal = items.next().expect("exit_signal").parse::<u32>().ok();
    let processor = items.next().expect("processor").parse::<u32>().ok();
    let rt_priority = items
        .next()
        .expect("rt_priority")
        .parse::<u32>()
        .expect("rt_priority");
    let policy = items
        .next()
        .expect("policy")
        .parse::<u32>()
        .expect("policy");
    let delayacct_blkio_ticks = items
        .next()
        .expect("delayacct_blkio_ticks")
        .parse::<u64>()
        .expect("delayacct_blkio_ticks");
    let guest_time = items
        .next()
        .expect("guest_time")
        .parse::<u64>()
        .expect("guest_time");
    let cguest_time = items.next().expect("cguest_time").parse::<u64>().ok();
    let start_data = items.next().expect("start_data").parse::<NonZeroU32>().ok();
    let end_data = items.next().expect("end_data").parse::<NonZeroU32>().ok();
    let start_brk = items.next().expect("start_brk").parse::<NonZeroU32>().ok();
    let arg_start = items.next().expect("arg_start").parse::<NonZeroU32>().ok();
    let arg_end = items.next().expect("arg_end").parse::<NonZeroU32>().ok();
    let env_start = items.next().expect("env_start").parse::<NonZeroU32>().ok();
    let env_end = items.next().expect("env_end").parse::<NonZeroU32>().ok();
    let exit_code = items.next().expect("exit_code").parse::<NonZeroU32>().ok();

    Ok(ProcStat {
        command,
        state,
        ppid,
        pgrp,
        session,
        tty_nr,
        tpgid,
        flags,
        minflt,
        cminflt,
        majflt,
        cmajflt,
        utime,
        stime,
        cutime,
        cstime,
        priority,
        nice,
        num_threads,
        starttime,
        vsize,
        rss,
        rsslim,
        startcode,
        endcode,
        startstack,
        kstkesp,
        kstkeip,
        wchan,
        exit_signal,
        processor,
        rt_priority,
        policy,
        delayacct_blkio_ticks,
        guest_time,
        cguest_time,
        start_data,
        end_data,
        start_brk,
        arg_start,
        arg_end,
        env_start,
        env_end,
        exit_code,
    })
}

#[derive(Debug, Clone, Copy)]
pub enum ProcState {
    Running,
    /// Sleeping in an interruptible wait
    Sleeping,
    /// Waiting in uninterruptible disk sleep
    Waiting,
    Zombie,
    /// Stopped (on a signal)
    Stopped,
    TracingStop,
    Dead,
    Idle,
}

/// Ref: <https://man7.org/linux/man-pages/man5/proc.5.html>
#[derive(Debug, Clone, Copy)]
pub struct ProcIo {
    /// Attempt to count the number of bytes which this process really did cause to be fetched from the storage layer.
    /// This is accurate for block-backed filesystems.
    pub read_bytes: u64,
    /// Attempt to count the number of bytes which this process caused to be sent to the storage layer
    pub write_bytes: u64,
    /// The big inaccuracy here is truncate.
    /// If a process writes 1 MB to a file and then deletes the file, it will in fact perform no writeout.
    /// But it will have been accounted as having caused 1 MB of write.
    /// In other words: this field represents the number of bytes which this process caused to not happen, by truncating pagecache.
    /// A task can cause "negative" I/O too.
    /// If this task truncates some dirty pagecache, some I/O which another task has been accounted for (in its write_bytes) will not be happening.
    pub cancelled_write_bytes: u64,
}
pub async fn read_proc_io(id: ProcId) -> Result<ProcIo, ReadStatsError> {
    let path = id.path("io");
    let file = tokio::fs::File::options()
        .read(true)
        .open(path)
        .await
        .map_err(ReadStatsError::NoSuchProcess)?;
    let buf = tokio::io::BufReader::new(file);
    let mut lines = buf.lines();
    let mut read_bytes = None;
    let mut write_bytes = None;
    let mut cancelled_write_bytes = None;
    while let Some(line) = lines.next_line().await.expect("UTF-8") {
        const READ_BYTES: &str = "read_bytes: ";
        if line.starts_with(READ_BYTES) {
            read_bytes = Some(
                line.chars()
                    .skip(READ_BYTES.len())
                    .collect::<String>()
                    .parse::<u64>()
                    .expect("read_bytes"),
            );
        }
        const WRITE_BYTES: &str = "write_bytes: ";
        if line.starts_with(WRITE_BYTES) {
            write_bytes = Some(
                line.chars()
                    .skip(WRITE_BYTES.len())
                    .collect::<String>()
                    .parse::<u64>()
                    .expect("write_bytes"),
            );
        }
        const CANCELLED_WRITE_BYTES: &str = "cancelled_write_bytes: ";
        if line.starts_with(CANCELLED_WRITE_BYTES) {
            cancelled_write_bytes = Some(
                line.chars()
                    .skip(CANCELLED_WRITE_BYTES.len())
                    .collect::<String>()
                    .parse::<u64>()
                    .expect("cancelled_write_bytes"),
            );
        }
    }
    let stats = ProcIo {
        read_bytes: read_bytes.expect("read_bytes"),
        write_bytes: write_bytes.expect("write_bytes"),
        cancelled_write_bytes: cancelled_write_bytes.expect("cancelled_write_bytes"),
    };
    Ok(stats)
}

/// Ref: <https://docs.kernel.org/scheduler/sched-stats.html>
#[derive(Debug, Clone, Copy)]
pub struct ProcSched {
    /// time spent on the cpu (in nanoseconds)
    pub cpu_time: u64,
    /// time spent waiting on a runqueue (in nanoseconds)
    pub wait_time: u64,
    /// # of timeslices run on this cpu
    pub timeslices: u64,
}
pub async fn read_proc_sched(id: ProcId) -> Result<ProcSched, ReadStatsError> {
    let path = id.path("schedstat");
    let mut file = tokio::fs::File::options()
        .read(true)
        .open(path)
        .await
        .map_err(ReadStatsError::NoSuchProcess)?;
    let mut text = String::new();
    file.read_to_string(&mut text).await.expect("UTF-8");
    drop(file);

    let mut items = text.split_whitespace();

    let cpu_time = items.next().expect("cpu_time").parse().expect("cpu_time");
    let wait_time = items.next().expect("wait_time").parse().expect("wait_time");
    let timeslices = items
        .next()
        .expect("timeslices")
        .parse()
        .expect("timeslices");

    Ok(ProcSched {
        cpu_time,
        wait_time,
        timeslices,
    })
}

/// Ref: <https://man7.org/linux/man-pages/man5/proc.5.html>
#[derive(Debug, Clone, Copy)]
pub struct ProcMemInfo {
    pub mem_total: u64,
}
pub async fn read_proc_mem_info() -> Result<ProcMemInfo, ReadStatsError> {
    let path = Path::new("/proc/meminfo");
    let file = tokio::fs::File::options()
        .read(true)
        .open(path)
        .await
        .map_err(ReadStatsError::NoSuchProcess)?;
    let buf = tokio::io::BufReader::new(file);
    let mut lines = buf.lines();
    let mut mem_total = None;
    while let Some(line) = lines.next_line().await.expect("UTF-8") {
        const MEM_TOTAL: &str = "MemTotal:";
        if line.starts_with(MEM_TOTAL) {
            let remaining = line.chars().skip(MEM_TOTAL.len()).collect::<String>();
            mem_total = Some(
                remaining
                    .split_whitespace()
                    .next()
                    .expect("mem_total")
                    .parse()
                    .expect("mem_total"),
            );
        }
    }

    Ok(ProcMemInfo {
        mem_total: mem_total.expect("mem_total"),
    })
}

#[derive(Debug, Error)]
pub enum ReadStatsError {
    #[error("No such process: {0}")]
    NoSuchProcess(#[source] std::io::Error),
}
