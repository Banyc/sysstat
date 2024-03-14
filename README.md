# sysstat

Modified sysstat.

## pidstat

Usage:

```bash
pidstat --cpu --mem --io -p 1
```

- Print statistics about CPU, memory, and I/O for the PID 1.

```bash
pidstat -urd -G systemd
```

- Print statistics about CPU, memory, and I/O for processes whose names contain "systemd".

Learn more from `pidstat --help`.
