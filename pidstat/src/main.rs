use std::{
    collections::{btree_map, BTreeMap},
    time::Duration,
};

use clap::Parser;
use pidstat::{
    read::{read_task_group_stats, ComponentOptions, ReadPidOptions, TaskGroupStats},
    TaskGroupStatsDisplay,
};

#[derive(Debug, Clone, Parser)]
struct Cli {
    #[clap(short, long)]
    pid: Option<usize>,
    #[clap(short('G'), long)]
    process_name: Option<String>,
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
    #[clap(short('u'), long)]
    cpu: bool,
    /// Report page faults and memory utilization.
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
    /// minflt/s
    ///        Total number of minor faults the task has made per
    ///        second, those which have not required loading a
    ///        memory page from disk.
    ///
    /// majflt/s
    ///        Total number of major faults the task has made per
    ///        second, those which have required loading a memory
    ///        page from disk.
    ///
    /// VSZ    Virtual Size: The virtual memory usage of entire
    ///        task in kilobytes.
    ///
    /// RSS    Resident Set Size: The non-swapped physical memory
    ///        used by the task in kilobytes.
    ///
    /// %MEM   The tasks's currently used share of available
    ///        physical memory.
    ///
    /// Command
    ///        The command name of the task.
    #[clap(short('r'), long)]
    mem: bool,
    /// Report task switching activity (kernels 2.6.23 and later
    /// only).  The following values may be displayed:
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
    /// cswch/s
    ///        Total number of voluntary context switches the task
    ///        made per second.  A voluntary context switch occurs
    ///        when a task blocks because it requires a resource
    ///        that is unavailable.
    ///
    /// nvcswch/s
    ///        Total number of non voluntary context switches the
    ///        task made per second.  An involuntary context
    ///        switch takes place when a task executes for the
    ///        duration of its time slice and then is forced to
    ///        relinquish the processor.
    ///
    /// Command
    ///        The command name of the task.
    #[clap(short('w'), long)]
    ctx_switch: bool,
    #[clap(short('t'), long)]
    task: bool,
    /// Specify the amount of time in seconds between each report
    #[clap(default_value = "1")]
    interval: u64,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let components = ComponentOptions {
        cpu: cli.cpu,
        mem: cli.mem,
        io: cli.io,
        ctx_switch: cli.ctx_switch,
    };

    let mut prev_stats = BTreeMap::<usize, TaskGroupStats>::new();

    loop {
        let pid = match (cli.pid, &cli.process_name) {
            (None, None) => panic!("Provide either `pid` or `process-name`"),
            (None, Some(process_name)) => ReadPidOptions { process_name }.read_pid().await,
            (Some(pid), None) => vec![pid],
            (Some(pid), Some(process_name)) => ReadPidOptions { process_name }
                .read_pid()
                .await
                .into_iter()
                .filter(|p| *p == pid)
                .collect::<Vec<_>>(),
        };

        for &p in &pid {
            if let btree_map::Entry::Vacant(e) = prev_stats.entry(p) {
                let Ok(s) = read_task_group_stats(p, components, cli.task).await else {
                    continue;
                };
                e.insert(s);
            }
        }
        tokio::time::sleep(Duration::from_secs(cli.interval)).await;
        for &p in &pid {
            let Ok(stats) = read_task_group_stats(p, components, cli.task).await else {
                prev_stats.remove(&p);
                continue;
            };
            let display = TaskGroupStatsDisplay {
                prev_stats: prev_stats.get(&p).unwrap(),
                curr_stats: &stats,
            };
            print!("{display}");
            prev_stats.insert(p, stats);
        }
    }
}
