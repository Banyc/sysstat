use std::time::Duration;

use clap::Parser;
use pidstat::{
    cpu::{CpuStatsHeaderDisplay, CpuStatsValueDisplay},
    io::{IoStatsHeaderDisplay, IoStatsValueDisplay},
    process::TidDisplayOption,
    read::{ProcId, ReadStatsOptions},
};

#[derive(Debug, Clone, Parser)]
struct Cli {
    #[clap(short, long)]
    pid: usize,
    #[clap(short('G'), long)]
    process_name: Option<usize>,
    /// Report I/O statistics (kernels 2.6.20 and later only).
    /// The following values may be displayed:
    ///
    /// UID    The real user identification number of the task
    ///         being monitored.
    ///
    /// USER   The name of the real user owning the task being
    ///         monitored.
    ///
    /// PID    The identification number of the task being
    ///         monitored.
    ///
    /// kB_rd/s
    ///         Number of kilobytes the task has caused to be read
    ///         from disk per second.
    ///
    /// kB_wr/s
    ///         Number of kilobytes the task has caused, or shall
    ///         cause to be written to disk per second.
    ///
    /// kB_ccwr/s
    ///         Number of kilobytes whose writing to disk has been
    ///         cancelled by the task. This may occur when the task
    ///         truncates some dirty pagecache. In this case, some
    ///         IO which another task has been accounted for will
    ///         not be happening.
    ///
    /// iodelay
    ///         Block I/O delay of the task being monitored,
    ///         measured in clock ticks. This metric includes the
    ///         delays spent waiting for sync block I/O completion
    ///         and for swapin block I/O completion.
    ///
    /// Command
    ///         The command name of the task.
    #[clap(short('d'), long)]
    io: bool,
    /// Report CPU utilization.
    ///
    /// When reporting statistics for individual tasks, the
    /// following values may be displayed:
    ///
    /// UID    The real user identification number of the task
    ///        being monitored.
    ///
    /// USER   The name of the real user owning the task being
    ///        monitored.
    ///
    /// PID    The identification number of the task being
    ///        monitored.
    ///
    /// %usr   Percentage of CPU used by the task while executing
    ///        at the user level (application), with or without
    ///        nice priority. Note that this field does NOT
    ///        include time spent running a virtual processor.
    ///
    /// %system
    ///        Percentage of CPU used by the task while executing
    ///        at the system level (kernel).
    ///
    /// %guest Percentage of CPU spent by the task in virtual
    ///        machine (running a virtual processor).
    ///
    /// %wait  Percentage of CPU spent by the task while waiting
    ///        to run.
    ///
    /// %CPU   Total percentage of CPU time used by the task. In
    ///        an SMP environment, the task's CPU usage will be
    ///        divided by the total number of CPU's if option -I
    ///        has been entered on the command line.
    ///
    /// CPU    Processor number to which the task is attached.
    ///
    /// Command
    ///        The command name of the task.
    ///
    /// When reporting global statistics for tasks and all their
    /// children, the following values may be displayed:
    ///
    /// UID    The real user identification number of the task
    ///        which is being monitored together with its
    ///        children.
    ///
    /// USER   The name of the real user owning the task which is
    ///        being monitored together with its children.
    ///
    /// PID    The identification number of the task which is
    ///        being monitored together with its children.
    ///
    /// usr-ms Total number of milliseconds spent by the task and
    ///        all its children while executing at the user level
    ///        (application), with or without nice priority, and
    ///        collected during the interval of time. Note that
    ///        this field does NOT include time spent running a
    ///        virtual processor.
    ///
    /// system-ms
    ///        Total number of milliseconds spent by the task and
    ///        all its children while executing at the system
    ///        level (kernel), and collected during the interval
    ///        of time.
    ///
    /// guest-ms
    ///        Total number of milliseconds spent by the task and
    ///        all its children in virtual machine (running a
    ///        virtual processor).
    ///
    /// Command
    ///        The command name of the task which is being
    ///        monitored together with its children.
    #[clap(short('u'), long)]
    cpu: bool,
    /// Specify the amount of time in seconds between each report
    #[clap(default_value = "1")]
    interval: u64,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let options = ReadStatsOptions {
        id: ProcId {
            pid: cli.pid,
            tid: None,
        },
        io: cli.io,
        disk: false,
        mem: false,
        cpu: cli.cpu,
    };
    let mut prev_process_stats = None;
    loop {
        if prev_process_stats.is_none() {
            let (_, p) = options.read().await.unwrap();
            prev_process_stats = Some(p);
        }
        tokio::time::sleep(Duration::from_secs(cli.interval)).await;
        let (process, curr_process_stats) = options.read().await.unwrap();

        if options.io {
            print!(
                "{}",
                IoStatsHeaderDisplay {
                    tid: TidDisplayOption::Pid
                }
            );
            print!(
                "{}",
                IoStatsValueDisplay {
                    tid: TidDisplayOption::Pid,
                    process: &process,
                    prev_stats: prev_process_stats.as_ref().unwrap().io.as_ref().unwrap(),
                    curr_stats: curr_process_stats.io.as_ref().unwrap(),
                }
            );
        }
        if options.cpu {
            print!(
                "{}",
                CpuStatsHeaderDisplay {
                    tid: TidDisplayOption::Pid
                }
            );
            print!(
                "{}",
                CpuStatsValueDisplay {
                    tid: TidDisplayOption::Pid,
                    process: &process,
                    prev_stats: prev_process_stats.as_ref().unwrap().cpu.as_ref().unwrap(),
                    curr_stats: curr_process_stats.cpu.as_ref().unwrap(),
                }
            );
        }

        prev_process_stats = Some(curr_process_stats);
    }
}
