//! Acceptance tests for spec 04: streaming `sum`, `product` and `collect` allocate nothing.
//!
//! A counting global allocator wraps the system one. The counter is per thread, so the
//! test harness's own threads cannot disturb a measurement.

use presence_rs::Presence;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct CountingAllocator;

thread_local! {
    // Const-initialized and without `Drop`, so with native thread-locals it is a plain
    // static: reading it never allocates, which `alloc` below relies on.
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

// SAFETY: `alloc` and `dealloc` forward to `System`; `realloc` and `alloc_zeroed` keep
// the trait defaults, which go through these two, so every allocation is counted once.
// The counter is a thread-local `Cell` that neither allocates nor touches the memory
// handed out, and `wrapping_add` keeps `alloc` from panicking.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.with(|count| count.set(count.get().wrapping_add(1)));
        // SAFETY: the caller upholds `GlobalAlloc::alloc`'s contract, which `System` shares.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` was allocated by `System` through `alloc` above with this `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// Runs `f` and returns how many allocations it made on this thread, not counting the
/// drop of its result.
fn allocations_during<R>(f: impl FnOnce() -> R) -> usize {
    let before = ALLOCATIONS.with(Cell::get);
    let result = f();
    let after = ALLOCATIONS.with(Cell::get);
    drop(result);
    after.wrapping_sub(before)
}

/// 1,000 presences: `Some(±1)` (so products cannot overflow) for the first 950, so a
/// long run of values reaches the target, then `Null`s and, when `with_absent`, one
/// `Absent` near the end.
fn mixed_presences<T: Copy>(one: T, minus_one: T, with_absent: bool) -> Vec<Presence<T>> {
    (0..1000)
        .map(|i| match i {
            990 if with_absent => Presence::Absent,
            950 | 970 => Presence::Null,
            _ if i % 2 == 0 => Presence::Some(one),
            _ => Presence::Some(minus_one),
        })
        .collect()
}

#[test]
fn summing_and_multiplying_presences_allocates_nothing() {
    // Given a sequence of 1,000 presences mixing Some, Null and Absent in any order
    let inputs = [
        mixed_presences(1_i64, -1, true),
        mixed_presences(1_i64, -1, false),
        mixed_presences(1_i64, -1, false)
            .into_iter()
            .filter(Presence::is_present)
            .collect(),
    ];

    for items in &inputs {
        // When it is summed and, separately, multiplied
        let sum_allocations = allocations_during(|| items.iter().copied().sum::<Presence<i64>>());
        let product_allocations =
            allocations_during(|| items.iter().copied().product::<Presence<i64>>());

        // Then no heap allocation happens during either call
        assert_eq!((sum_allocations, product_allocations), (0, 0));
    }
}

/// A collection target that only counts its items, so it never allocates.
struct Count(usize);

impl FromIterator<i64> for Count {
    fn from_iter<I: IntoIterator<Item = i64>>(iter: I) -> Self {
        Count(iter.into_iter().count())
    }
}

#[test]
fn collecting_into_a_non_allocating_type_allocates_nothing() {
    // Given a sequence of 1,000 presences of i64 values mixing Some, Null and Absent
    let inputs = [
        mixed_presences(1_i64, -1, true),
        mixed_presences(1_i64, -1, false),
        mixed_presences(1_i64, -1, false)
            .into_iter()
            .filter(Presence::is_present)
            .collect(),
    ];

    for items in &inputs {
        // When it is collected into Presence<Count>, a test type that counts items without
        // allocating
        let allocations = allocations_during(|| items.iter().copied().collect::<Presence<Count>>());

        // Then no heap allocation happens during the call
        assert_eq!(allocations, 0);
    }
    let all_some = &inputs[2];
    let collected = all_some.iter().copied().collect::<Presence<Count>>();
    assert!(collected.is_some_and(|Count(n)| n == all_some.len()));
}

#[test]
fn library_source_never_uses_the_alloc_crate() {
    // The no-alloc promise (crates.io category `no-std::no-alloc`) is not checked by the
    // no_std CI build, because that target ships `alloc`; scan the library source instead.
    for (file, source) in [
        ("src/lib.rs", include_str!("../src/lib.rs")),
        (
            "src/presence/mod.rs",
            include_str!("../src/presence/mod.rs"),
        ),
        (
            "src/presence/convert.rs",
            include_str!("../src/presence/convert.rs"),
        ),
        (
            "src/presence/iter.rs",
            include_str!("../src/presence/iter.rs"),
        ),
        (
            "src/presence/query.rs",
            include_str!("../src/presence/query.rs"),
        ),
        (
            "src/presence/refs.rs",
            include_str!("../src/presence/refs.rs"),
        ),
        (
            "src/presence/transform.rs",
            include_str!("../src/presence/transform.rs"),
        ),
        ("src/serde.rs", include_str!("../src/serde.rs")),
    ] {
        let library_code = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert!(
            !library_code.contains("extern crate alloc") && !library_code.contains("alloc::"),
            "{file} uses the alloc crate outside its tests"
        );
    }
}
