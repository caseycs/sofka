//! Heap cost of the object store and the view cache.
//!
//! Criterion measures time; the Tier 1 items in `plan.txt` are justified by
//! memory, so they need their own probe. dhat's ad-hoc stats give live heap
//! bytes at a point in the program, which is exactly the question here: what
//! does holding N pods cost, and what does the view cache multiply it by?
//!
//! Run with:
//!   cargo run --release --example memprobe --features "bench,dhat-heap"

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use sofka::benchsupport as bs;

const N: usize = 2_000;
const VIEWS: usize = 8; // VIEW_CACHE_MAX

fn mib(bytes: f64) -> f64 {
    bytes / (1024.0 * 1024.0)
}

fn live() -> f64 {
    dhat::HeapStats::get().curr_bytes as f64
}

fn main() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let base = live();

    // ---- one view's worth of objects -------------------------------------
    let one = bs::items(N);
    let after_one = live();
    let per_view = after_one - base;
    println!(
        "store, {N} pods:                {:8.2} MiB   ({:6.2} KiB/pod)",
        mib(per_view),
        per_view / N as f64 / 1024.0
    );

    // ---- seeding a cached view: Arc clone vs the deep clone it replaced ---
    let before_arc = live();
    let arc_copy = bs::arc_clone_items(&one);
    let arc_cost = live() - before_arc;
    println!(
        "  + Arc-clone of that snapshot: {:8.2} MiB   (what seeding costs now)",
        mib(arc_cost)
    );
    drop(arc_copy);

    let before_deep = live();
    let deep_copy = bs::deep_clone_items(&one);
    let deep_cost = live() - before_deep;
    println!(
        "  + deep clone of that snapshot:{:8.2} MiB   (what it cost before 1.1/1.2)",
        mib(deep_cost)
    );
    println!(
        "  -> seeding a cached view is {:.0}x cheaper in memory",
        deep_cost / arc_cost.max(1.0)
    );
    drop(deep_copy);

    // ---- the view cache: VIEWS independent object sets --------------------
    // This is the shape behind the 497 MiB in docs/benchmark-k9s.md. Distinct
    // objects per view, because two different views list different resources.
    let mut cache: Vec<sofka::store::Items> = Vec::new();
    for _ in 0..VIEWS {
        cache.push(bs::items(N));
    }
    let after_cache = live();
    println!();
    println!(
        "view cache, {VIEWS} x {N} pods:     {:8.2} MiB",
        mib(after_cache - after_one)
    );
    println!(
        "  -> bounding the cache by objects instead of views would cap this at\n     one view's {:.0} MiB, not {:.0} MiB",
        mib(per_view),
        mib(after_cache - after_one)
    );

    let stats = dhat::HeapStats::get();
    println!();
    println!(
        "peak live heap:                {:8.2} MiB",
        mib(stats.max_bytes as f64)
    );
    println!(
        "total allocated over run:      {:8.2} MiB",
        mib(stats.total_bytes as f64)
    );
    println!("total allocation count:        {:8}", stats.total_blocks);

    drop(cache);
    drop(one);
}
