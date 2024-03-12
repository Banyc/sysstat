use std::time::Duration;

use clap::Parser;
use pidstat::{
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
        cpu: false,
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
                    prev_stats: &prev_process_stats.unwrap().io.unwrap(),
                    curr_stats: curr_process_stats.io.as_ref().unwrap(),
                }
            );
        }
        prev_process_stats = Some(curr_process_stats);
    }
}
