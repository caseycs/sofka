# Performance benchmark: sofka and k9s

This document compares sofka 0.16.3 with k9s 0.51.0.
The test used a live Kubernetes cluster.
The results apply only to the test conditions in this document.
They do not prove the performance of all clusters or computers.

This document uses ASD-STE100 writing principles.
Product names and Kubernetes terms are technical names.
The document did not go through a formal ASD-STE100 compliance check.

## Result summary

| Measurement                       | How the test measured it                             |         sofka |        k9s | Observed result                     |
| --------------------------------- | ---------------------------------------------------- | ------------: | ---------: | ----------------------------------- |
| Reach visible count of 1,700 pods | Five tmux runs; median visible-screen time           |   **2.032 s** |    3.901 s | sofka used 47.9% less time          |
| Apply exact pod filter            | 20 tmux runs; median `Enter`-to-row time             |  **12.28 ms** |   25.36 ms | sofka used 51.6% less time          |
| Clear pod filter                  | 20 tmux runs; median `Escape`-to-count-restored time |  **11.88 ms** |  602.40 ms | sofka used 98.0% less time          |
| Move selected row                 | 40 tmux runs; median visible-screen time             |  **12.42 ms** |   12.77 ms | The result was at the harness floor |
| First StatefulSets open           | Five fresh TUI runs; median visible-screen time      |  **76.53 ms** |  363.25 ms | sofka used 78.9% less time          |
| Process RSS                       | 58 samples from one loaded TUI per program           | **497.0 MiB** |  983.3 MiB | sofka RSS was 49.5% lower           |
| Version command                   | 100 alternating warm runs; median completion time    |   **6.07 ms** |   48.87 ms | sofka used 87.6% less time          |
| Help command                      | 100 alternating warm runs; median completion time    |   **6.26 ms** |   48.66 ms | sofka used 87.1% less time          |
| Release binary                    | Size of the executable file                          | **13.08 MiB** | 136.23 MiB | sofka used 90.4% less disk space    |

The tmux tests measured the visible TUI screen.
The command results are end-to-end completion times.
The test did not send a mutating command to the cluster.

## Test environment

The test used this environment:

| Item                  | Value                           |
| --------------------- | ------------------------------- |
| Computer              | Apple M3 Max                    |
| Operating system      | macOS, Darwin 25.6.0, arm64     |
| Terminal multiplexer  | tmux 3.7b                       |
| Terminal type         | `xterm-256color`                |
| Terminal size         | 180 columns by 50 rows          |
| sofka version         | 0.16.3                          |
| k9s version           | 0.51.0                          |
| Rust version          | 1.97.0                          |
| Kubernetes nodes      | 79                              |
| Kubernetes namespaces | 24                              |
| Kubernetes pods       | 1,825 to 1,858 during all tests |

The pod quantity changed because the cluster was active.
This change is one cause of variation in the results.

## Test controls

The test used these controls:

1. Each program used the same kubeconfig file and context.
2. Each program used a new process when the method required a fresh TUI.
3. Each program used an isolated `XDG_CONFIG_HOME` directory.
4. Each program opened pods from all namespaces.
5. Each program used read-only mode.
6. Each program used the same terminal type and terminal size.
7. tmux used an isolated server and no user configuration.
8. The test alternated sofka and k9s for paired process runs.
9. The test did not flush the operating-system file cache.
10. The test used only navigation, filter, and exit keys.
11. The test did not create, patch, scale, restart, delete, exec, or port-forward a resource.

The isolated configuration prevented personal settings from changing the result.
It also made the test different from a normal user session.

## tmux TUI measurement

### Safety

The tmux test used the live production cluster in read-only mode.
It used Kubernetes list, get, watch, and metrics requests.
It did not generate Kubernetes events for the benchmark.
It did not run a load generator.

The test used `/`, `Enter`, `Escape`, `j`, `k`, and resource commands.
It used `Ctrl-C` only to stop the local TUI process.

The pod count was 1,856 before the tmux test.
The pod count was 1,847 after the tmux test.
The cluster was active during the test.
The benchmark sent no mutating command that could cause this count change.

### Harness

The harness used an isolated tmux server.
It used `/dev/null` as the tmux configuration file.
It set the pane size to 180 columns by 50 rows.
It started both programs with `--readonly`.

Python `time.perf_counter()` supplied the time values.
`tmux send-keys` supplied all input.
`tmux capture-pane` supplied the visible screen.
The harness polled the screen after each input.

One `capture-pane` operation had a median cost of 6.26 ms.
Its 95th percentile cost was 7.17 ms.
The observed input-to-screen floor was near 12 ms.
Results near this floor do not show a useful product difference.

### Large pod view

The timer started before tmux sent the program command.
The timer stopped when the visible count was at least 1,700.
The screen also had to contain at least 20 pod status rows.
The cluster had approximately 1,850 pods during these runs.

The test used five runs for each program.
It alternated one k9s run and one sofka run.

|               Trial |       sofka |          k9s |
| ------------------: | ----------: | -----------: |
|                   1 |     4.130 s |      3.901 s |
|                   2 |     7.820 s |     14.369 s |
|                   3 |     1.980 s |      3.706 s |
|                   4 |     1.744 s |     10.290 s |
|                   5 |     2.032 s |      3.740 s |
|          **Median** | **2.032 s** |  **3.901 s** |
|            **Mean** | **3.541 s** |  **7.201 s** |
| **95th percentile** | **7.082 s** | **13.553 s** |

The live cluster caused high variation.
The median is more useful than the mean for this small sample.
This test does not show the time to an exact final count.

### Filter and row movement

The filter test used one loaded pod view for each program.
The exact filter text was `soketi-soketi-prod-5999547bbb-xf75k`.
The timer started immediately before the harness sent `Enter`.
The timer stopped when the expected pod row was visible.

The clear timer started immediately before the harness sent `Escape`.
It stopped when the visible pod count was at least 1,700 again.

The row test used 20 `j` inputs and 20 `k` inputs.
The timer stopped when the screen with color data changed.
In the table, p95 means the 95th percentile.

| Action             | Samples | sofka median |    sofka p95 | k9s median |     k9s p95 |
| ------------------ | ------: | -----------: | -----------: | ---------: | ----------: |
| Apply exact filter |      20 | **12.28 ms** | **15.29 ms** |   25.36 ms | 1,035.07 ms |
| Clear filter       |      20 | **11.88 ms** | **14.33 ms** |  602.40 ms | 1,101.14 ms |
| Move selected row  |      40 | **12.42 ms** | **13.51 ms** |   12.77 ms |    17.12 ms |

The row movement medians were near the harness floor.
Thus, this method did not find a useful median difference for row movement.
Background screen updates can also affect the row test.

### First resource open in a fresh TUI

Each run started a new TUI in the pod view.
StatefulSets had not been open in that process.
The harness entered `:statefulsets`.
The timer started immediately before it sent `Enter`.
The timer stopped when the visible StatefulSets count was at least 18.

| Statistic       |         sofka |       k9s |
| --------------- | ------------: | --------: |
| Runs            |             5 |         5 |
| Median          |  **76.53 ms** | 363.25 ms |
| Mean            | **118.28 ms** | 372.19 ms |
| 95th percentile | **199.19 ms** | 394.50 ms |
| Minimum         |  **60.85 ms** | 353.57 ms |
| Maximum         | **206.09 ms** | 395.79 ms |

The result includes tmux input and screen-capture cost.
It also includes live Kubernetes API response time.

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

The test used one loaded pod TUI for each program.
It collected the samples after the interaction tests.
For each sample, it ran `/bin/ps -o rss= -p PID`.
macOS reports this RSS field in kibibytes.
The test divided the value by 1,024 to get mebibytes.
It collected 58 samples for each program during 15 seconds.
The target sample interval was 0.25 seconds.

### Memory data

| Statistic   |         sofka |       k9s |
| ----------- | ------------: | --------: |
| Median RSS  | **497.0 MiB** | 983.3 MiB |
| Minimum RSS | **497.0 MiB** | 983.2 MiB |
| Maximum RSS | **498.3 MiB** | 983.3 MiB |

The sofka median RSS was 486.3 MiB lower.
The sofka median RSS was 49.5% lower.
The k9s median RSS was 1.98 times the sofka median RSS.

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
- Kubernetes API request quantity.
- Network data quantity.
- Memory use during a long session.
- Memory use for other resource types.
- Cold start after a computer restart.
- Garbage collector pause time.
- Performance on Linux or Windows.
- Exact time to the final pod count.
- Sustained scroll frame rate.

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

The tmux large-view test stopped at a visible count of 1,700.
It did not wait for an exact final pod count.
The tmux screen polling added observation delay.
Background screen updates can affect the row movement result.

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
5. Measure key response with a lower-overhead harness and fixed object sets.
6. Run the same tests on Linux.
7. Save the test harness and raw data in the repository.
