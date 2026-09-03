//! Baselines for the paths the optimization plan targets.
//!
//! Run with `cargo bench --features bench`.
//!
//! Each group isolates one optimized hot path so changes can be evaluated
//! independently:
//!
//! - `rows_cache`  -> 2.1 (watch event followed by the redraw query)
//! - `filter`      -> 3.1 (uncached cell extraction per keystroke)
//! - `cells`       -> 2.2 (`pod_summary` 3x, `helm::decode` 5x)
//! - `metadata`    -> 3.3 (typed field lookup vs whole-meta serialization)
//! - `log_filter`  -> 4.1 (O(n*m) substring scan)
//! - `log_wrap`    -> 2.3 / 4.2 (full-buffer re-measure per frame)

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use sofka::benchsupport as bs;
use sofka::columns;
use sofka::logfilter::LogMatcher;

/// 2.1 — one watch event followed by the redraw's row-count query. That pair
/// is the real steady-state unit: after 2.1 an ordinary unsorted/unfiltered
/// update keeps the existing key order, while paths that can change ordering
/// or membership are covered by the filter benchmarks below.
fn rows_cache(c: &mut Criterion) {
    let mut g = c.benchmark_group("rows_cache");
    for n in [500usize, 2_000] {
        let (mut app, _rx) = bs::pods_app(n);
        // Warm the caches so the first measured iteration isn't a cold build.
        black_box(app.row_count());
        g.bench_with_input(BenchmarkId::new("event_then_rebuild", n), &n, |b, &n| {
            let mut i = 0usize;
            b.iter(|| {
                bs::touch_one(&mut app, i % n);
                i += 1;
                black_box(app.row_count())
            });
        });
    }
    g.finish();
}

/// 3.1 — the same rebuild with a filter active. `no_match` is the expensive
/// case: every object misses on name, so `fuzzy_match_row` falls through to a
/// full uncached row extraction plus a fuzzy match per cell.
fn filter(c: &mut Criterion) {
    let mut g = c.benchmark_group("filter");
    let n = 2_000usize;
    for (label, pat) in [
        ("name_hit", "workload-00042"),
        ("no_match", "zzzznotpresent"),
        ("broad", "svc"),
    ] {
        let (mut app, _rx) = bs::pods_app(n);
        app.filter = pat.to_string();
        black_box(app.row_count());
        g.bench_with_input(BenchmarkId::new(label, n), &n, |b, &n| {
            let mut i = 0usize;
            b.iter(|| {
                bs::touch_one(&mut app, i % n);
                i += 1;
                black_box(app.row_count())
            });
        });
    }
    g.finish();
}

/// 2.2 — cell extraction per row. `pods` pays `pod_summary` three times;
/// `helm` pays base64 x2 + gunzip + a full JSON parse five times.
fn cells(c: &mut Criterion) {
    let mut g = c.benchmark_group("cells");

    let pods: Vec<_> = (0..256).map(bs::pod).collect();
    let pod_spec = columns::build_spec("pods", None, None, false);
    g.bench_function("pods_256", |b| {
        b.iter(|| {
            for o in &pods {
                black_box(pod_spec.cells(o));
            }
        });
    });

    // Helm is two orders of magnitude slower per row, so it gets far fewer.
    let helm: Vec<_> = (0..16).map(bs::helm_secret).collect();
    let helm_spec = columns::build_spec("helm", None, None, false);
    g.bench_function("helm_16", |b| {
        b.iter(|| {
            for o in &helm {
                black_box(helm_spec.cells(o));
            }
        });
    });

    g.finish();
}

/// 3.3 — a common user-column path through labels. Keep the former
/// serialize-all implementation alongside the production fast path as an
/// explicit baseline; both return the same owned `Value`.
fn metadata(c: &mut Criterion) {
    let mut g = c.benchmark_group("metadata");
    let pods: Vec<_> = (0..2_000).map(bs::pod).collect();
    let pointer = "/metadata/labels/app.kubernetes.io~1name";
    let rest = pointer.strip_prefix("/metadata").unwrap();

    g.bench_function("typed_labels_2000", |b| {
        b.iter(|| {
            for pod in &pods {
                black_box(sofka::views::extract(pod, pointer));
            }
        });
    });
    g.bench_function("serialized_baseline_2000", |b| {
        b.iter(|| {
            for pod in &pods {
                let meta = serde_json::to_value(&pod.metadata).unwrap();
                black_box(meta.pointer(rest).cloned());
            }
        });
    });
    g.finish();
}

/// 4.1 — the substring/regex scan, over a buffer the size the log view keeps.
fn log_filter(c: &mut Criterion) {
    let mut g = c.benchmark_group("log_filter");
    let lines = bs::log_lines(10_000);
    for (label, pat) in [
        ("empty", ""),
        ("substr_hit", "reconcile"),
        ("substr_miss", "zzzznotpresent"),
        ("substr_late", "duration"),
        ("regex", "/failed to sync [0-9]+/"),
        ("inverse", "!healthz"),
    ] {
        let m = LogMatcher::new(pat);
        g.bench_function(label, |b| {
            b.iter(|| {
                let mut hits = 0usize;
                for l in &lines {
                    if m.matches(l) {
                        hits += 1;
                    }
                }
                black_box(hits)
            });
        });
    }
    g.finish();
}

/// 2.3 / 4.2 — the per-frame height re-measure. `ascii` takes the fast path;
/// `wide` forces the per-char `unicode_width` walk on every tenth line.
fn log_wrap(c: &mut Criterion) {
    let mut g = c.benchmark_group("log_wrap");
    let ascii = bs::log_lines(10_000);
    let wide = bs::log_lines_wide(10_000);
    for (label, lines) in [("ascii", &ascii), ("wide", &wide)] {
        g.bench_function(label, |b| {
            b.iter(|| {
                let mut total = 0usize;
                for l in lines.iter() {
                    total += bs::wrapped_height(l, 120);
                }
                black_box(total)
            });
        });
    }
    g.finish();
}

/// 2.3 — what one *frame* of the log view costs, which is the number that
/// actually matters. `steady` is a redraw with no new lines (scrolling, a
/// cursor move, the 1 Hz tick); `streaming` is a redraw after a batch of new
/// lines arrives. Before the index these were both O(buffer).
fn log_viewport(c: &mut Criterion) {
    use sofka::app::LogsView;

    let mut g = c.benchmark_group("log_viewport");
    let lines = bs::log_lines(10_000);

    for (label, wrap_width) in [("nowrap", 0usize), ("wrap", 120)] {
        // Steady state: the buffer is unchanged between frames.
        let mut logs = LogsView::default();
        logs.view.lines.extend(lines.iter().cloned());
        logs.set_filter("reconcile".into());
        logs.refresh_index(wrap_width); // warm
        g.bench_function(BenchmarkId::new("steady", label), |b| {
            b.iter(|| black_box(logs.refresh_index(wrap_width).total_rows()));
        });

        // Streaming: 50 new lines per frame, the shape of a busy pod.
        let mut logs = LogsView::default();
        logs.view.lines.extend(lines.iter().cloned());
        logs.set_filter("reconcile".into());
        logs.refresh_index(wrap_width);
        let batch: Vec<String> = bs::log_lines(50);
        g.bench_function(BenchmarkId::new("streaming", label), |b| {
            b.iter(|| {
                logs.view.lines.extend(batch.iter().cloned());
                black_box(logs.refresh_index(wrap_width).total_rows())
            });
        });
    }
    g.finish();
}

criterion_group!(
    benches,
    rows_cache,
    filter,
    cells,
    metadata,
    log_filter,
    log_wrap,
    log_viewport
);
criterion_main!(benches);
