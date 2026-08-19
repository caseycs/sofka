# Performance benchmark: sofka and k9s

This document compares sofka 0.16.3 with k9s 0.51.0.
The test used a live Kubernetes cluster.
The results apply only to the test conditions in this document.
They do not prove the performance of all clusters or computers.

This document uses ASD-STE100 writing principles.
Product names and Kubernetes terms are technical names.
The document did not go through a formal ASD-STE100 compliance check.

## Result summary

| Measurement         | How the test measured it                                                |         sofka |         k9s | Observed result                  |
| ------------------- | ----------------------------------------------------------------------- | ------------: | ----------: | -------------------------------- |
| Time to status text | Five process runs; median time to raw PTY text `Running` or `Succeeded` |     **2.1 s** |       3.8 s | sofka used 44.7% less time       |
| Process RSS         | Median resident set size after a 5 s wait                               | **594.2 MiB** | 1,087.7 MiB | sofka RSS was 45.4% lower        |
| Version command     | 100 alternating warm runs; median completion time                       |   **6.07 ms** |    48.87 ms | sofka used 87.6% less time       |
| Help command        | 100 alternating warm runs; median completion time                       |   **6.26 ms** |    48.66 ms | sofka used 87.1% less time       |
| Release binary      | Size of the executable file                                             | **13.08 MiB** |  136.23 MiB | sofka used 90.4% less disk space |

In this test, sofka used less time and disk space.
sofka also had lower RSS.
The status-text result and the RSS result are the most useful results for TUI use.
The command results are end-to-end completion times.

## Test environment

The test used this environment:

| Item                  | Value                          |
| --------------------- | ------------------------------ |
| Computer              | Apple M3 Max                   |
| Operating system      | macOS, Darwin 25.6.0, arm64    |
| Terminal type         | `xterm-256color`               |
| Terminal size         | 180 columns by 50 rows         |
| sofka version         | 0.16.3                         |
| k9s version           | 0.51.0                         |
| Rust version          | 1.97.0                         |
| Kubernetes nodes      | 79                             |
| Kubernetes namespaces | 24                             |
| Kubernetes pods       | 1,825 to 1,853 during the test |

The pod quantity changed because the cluster was active.
This change is one cause of variation in the status-text results.

## Test controls

The test used these controls:

1. Each program used the same kubeconfig file and context.
2. Each program used a new process for each status-text test.
3. Each program used an isolated `XDG_CONFIG_HOME` directory.
4. Each program opened pods from all namespaces.
5. Each program used read-only mode.
6. Each program used the same terminal type and terminal size.
7. The test alternated sofka and k9s for the status-text and command measurements.
8. The test did not flush the operating-system file cache.

The isolated configuration prevented personal settings from changing the result.
It also made the test different from a normal user session.

## Build and start commands

The test built sofka with this command:

```sh
cargo build --release --locked
```

The release profile used thin link-time optimization.
The release profile also removed symbols from the binary.

The test started sofka with this command:

```sh
target/release/sofka --readonly -A pods
```

The test started k9s with this command:

```sh
k9s --readonly -A -c pods --splashless --logoless
```

The k9s command removed its splash screen and logo.
This control reduced work that was not necessary for the pod test.

The test invoked the installed k9s launcher.
This launcher ran a 416-byte shell script before it started `.k9s-wrapped`.
Thus, the k9s time results include the launcher cost.
The sofka time results include a direct executable start.

## Status-text time measurement

### Definition

Status-text time began when the test supervisor created the process.
It stopped when the log matcher found `Running` or `Succeeded` in raw PTY output.
The matcher did not rebuild the terminal screen.
The matcher did not parse a table cell.
Thus, this result measures the time to matching text from the pod view.
It does not measure the time to the first visible pod row.
It does not measure the time to load every pod.

The test ran each program in a real pseudo-terminal.
This method let each program use its normal TUI path.
The test used five starts for each program.
The test started the programs in alternating order.
The supervisor reported each duration to the nearest 0.1 seconds.

### Raw status-text time data

|      Trial |         sofka |           k9s |
| ---------: | ------------: | ------------: |
|          1 |         3.3 s |         3.7 s |
|          2 |         1.8 s |         3.8 s |
|          3 |         3.6 s |         5.9 s |
|          4 |         1.8 s |         6.0 s |
|          5 |         2.1 s |         3.7 s |
| **Median** |     **2.1 s** |     **3.8 s** |
|   **Mean** |    **2.52 s** |    **4.62 s** |
|  **Range** | **1.8-3.6 s** | **3.7-6.0 s** |

The sofka status-text median was 1.7 seconds lower.
The sofka status-text mean was 2.10 seconds lower.
The ratio of the status-text medians was 1.81.

These were new process starts.
They were not cold file-cache starts.
The operating system and the Kubernetes API could use cached data.

## Process memory measurement

### Definition

Resident set size (RSS) is the resident memory mapped into a process.
RSS can include pages that processes share.
It does not show the memory that only one process owns.

The test used one separate steady process for each program.
The test waited five seconds after the status-text match.
For each sample, it ran `/bin/ps -o rss= -p PID`.
macOS reports this RSS field in kibibytes.
The test divided the value by 1,024 to get mebibytes.
It collected 58 samples for each program during 15 seconds.
The target sample interval was 0.25 seconds.

### Memory data

| Statistic   |         sofka |         k9s |
| ----------- | ------------: | ----------: |
| Median RSS  | **594.2 MiB** | 1,087.7 MiB |
| Minimum RSS | **591.9 MiB** | 1,086.6 MiB |
| Maximum RSS | **601.1 MiB** | 1,088.5 MiB |

The sofka median RSS was 493.5 MiB lower.
The sofka median RSS was 45.4% lower.
The k9s median RSS was 1.83 times the sofka median RSS.

The samples show the change in one sofka process and one k9s process.
They do not show the difference between many process starts.

## Command completion measurement

The test measured two commands that do not connect to the cluster.
Python `time.perf_counter_ns()` supplied the start and stop times.
The timer started immediately before `subprocess.run` created the process.
The timer stopped after the process returned.
The test sent standard output and standard error to the null device.
The test ran each command 100 times.
It alternated the programs after each command.
It did not use a separate warm-up run.
The file cache was warm because the test had started both programs before this test.
The 95th percentile used linear interpolation at rank `(n - 1) * 0.95`.

The version commands were:

```sh
target/release/sofka --version
k9s version --short
```

The help commands were:

```sh
target/release/sofka --help
k9s --help
```

### Version command data

| Statistic       |       sofka |      k9s |
| --------------- | ----------: | -------: |
| Median          | **6.07 ms** | 48.87 ms |
| Mean            | **6.07 ms** | 48.56 ms |
| 95th percentile | **6.81 ms** | 51.20 ms |
| Minimum         | **5.17 ms** | 44.06 ms |
| Maximum         | **7.24 ms** | 52.45 ms |

### Help command data

| Statistic       |       sofka |      k9s |
| --------------- | ----------: | -------: |
| Median          | **6.26 ms** | 48.66 ms |
| Mean            | **6.24 ms** | 48.75 ms |
| 95th percentile | **6.90 ms** | 51.67 ms |
| Minimum         | **5.59 ms** | 44.88 ms |
| Maximum         | **7.17 ms** | 52.99 ms |

These commands do not measure Kubernetes API work.
They do not measure redraw time or key response time.
The k9s measurements include its installed shell launcher.
The commands also produce different quantities of output.
Thus, the results show end-to-end command completion time.

## Binary size measurement

The test used the file size in bytes.
It converted bytes to mebibytes with 1 MiB equal to 1,048,576 bytes.

| File                   |       Bytes |       MiB |
| ---------------------- | ----------: | --------: |
| `target/release/sofka` |  13,712,928 | **13.08** |
| k9s executable         | 142,844,032 |    136.23 |

The k9s Nix package used a 416-byte shell wrapper.
The test did not use the wrapper size as the k9s binary size.
It used the size of the `.k9s-wrapped` executable.
The k9s executable was 10.42 times larger than the sofka executable.

## What the test did not measure

The test did not measure these items:

- CPU use under a fixed event rate.
- Redraw latency during a large watch event burst.
- Key response time.
- Kubernetes API request quantity.
- Network data quantity.
- Memory use during a long session.
- Memory use for other resource types.
- Cold start after a computer restart.
- Garbage collector pause time.
- Performance on Linux or Windows.

The test collected CPU samples.
The live event rate changed during the samples.
Thus, the CPU data did not give a valid comparison.
This document does not include that data.

## Limits of the result

This benchmark used one computer and one cluster.
The cluster workload changed during the test.
The two programs can request different Kubernetes data.
The programs can also process that data in different ways.

The status-text matcher found the first matching text in raw PTY output.
It did not verify that the text was in a table cell.
It did not confirm that each program had loaded all pods.
Thus, status-text time is not full-load time.

The k9s time results include its installed shell launcher.
The sofka time results do not include an equivalent launcher.
Thus, the time results compare the installed user commands.

The memory test used one process for each program.
A larger sample of process starts can give a better memory estimate.
A fixed Kubernetes event replay can give a better CPU estimate.

Do not use these results as a general performance guarantee.
Use them as one measured comparison for the specified versions and environment.

## Recommended next tests

Use these tests to make the comparison stronger:

1. Replay a fixed Kubernetes watch event stream.
2. Measure 10 independent memory runs for each program.
3. Measure the time until each program has all pod rows.
4. Measure redraw latency during a fixed event burst.
5. Measure key response time with 100, 1,000, and 10,000 objects.
6. Run the same tests on Linux.
7. Save the test harness and raw data in the repository.
