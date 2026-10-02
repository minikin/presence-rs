# Spec 04 — Allocation-free collect, sum and product; no_std

**Status:** Implemented
**Effort:** Medium
**Module:** `src/presence.rs`, `src/lib.rs`, `Cargo.toml`, `.github/workflows/ci.yml`, `CHANGELOG.md`

## Context

`FromIterator`, `Sum` and `Product` for `Presence` first copy every `Some`
value into a temporary `Vec` and only then build the result. That costs an
O(n) heap allocation every time a sequence of presences is collected,
summed or multiplied, even when the result is a single number. Those three
`Vec`s are also the only thing in the library that needs an allocator.
Everything else it uses (`fmt`, `iter`, `pin`, `slice`, `ops`, `mem`)
exists in `core`.

Who hits it:
- anyone summing or collecting presences in a hot path pays for an
  allocation they didn't ask for;
- anyone on `no_std` (embedded, WASM without an allocator, kernels) can't
  use the crate at all, even though a tri-state schema type fits those
  domains.

Desired outcome: the three impls stream their values with no intermediate
buffer, and the crate is `#![no_std]` with no new feature flags.

What streaming keeps:
- The state rule from spec 03: `Absent` takes precedence over `Null`,
  which takes precedence over `Some`.
- How much of the input is consumed: iteration stops at the first `Absent`
  and otherwise runs to the end. After a `Null`, the rest is still scanned,
  because an `Absent` later on wins.

What streaming changes:
- Values before the first `Null` or `Absent` now go into the sum, product
  or collection as they arrive, then get discarded if the result is `Null`
  or `Absent`. This is what std's `Option` does. For example,
  `[Some(i32::MAX), Some(1), Null].sum()` overflows, so it panics in debug
  builds, where it used to return `Null`.
- Collecting builds the target value from that prefix too, before
  discarding it.

Decisions (settled in the spec interview):

1. **Stream like std.** Values before the first `Null` or `Absent` are
   combined as they arrive. The overflow edge case is documented on
   `Sum`/`Product` and in the CHANGELOG under `Changed`.
2. **Always `#![no_std]`.** Nothing needs `std` or `alloc`, so there is no
   `std` or `alloc` feature.
3. **serde:** the dependency becomes `default-features = false` and keeps
   `features = ["derive"]`, so the `serde` feature works on `no_std` and
   crates relying on our `derive` unification keep working. Dropping
   serde's default `std` feature can still break a crate that relied on us
   enabling it (for example, deriving for `String` with its own `serde`
   pulled in without `std`). The CHANGELOG says so under `Changed`, with
   the fix: enable serde's `std` or `alloc` feature yourself. *(Corrected
   after review, with the user's permission.)*
4. **Proof:**
   - a CI job builds the library for a `no_std` target
     (`thumbv7em-none-eabihf`), both without features and with `serde`;
   - a test with a counting global allocator shows that `sum`, `product`
     and collecting into a type that doesn't allocate perform zero
     allocations.
5. **CHANGELOG** under `[Unreleased]`: `Added` for `no_std` support,
   `Changed` for the streaming behaviour and its overflow edge case. No
   version bump.

---

## Acceptance Tests

### Scenario: Summing and multiplying presences allocates nothing

```
Given a sequence of 1,000 presences mixing Some, Null and Absent in any order
When  it is summed and, separately, multiplied
Then  no heap allocation happens during either call
```

### Scenario: Collecting into a non-allocating type allocates nothing

```
Given a sequence of 1,000 presences of i64 values mixing Some, Null and Absent
When  it is collected into Presence<Count>, a test type that counts items without allocating
Then  no heap allocation happens during the call
```

### Scenario: Streaming keeps the Absent over Null over Some result

```
Given any list of presences (property)
When  it is collected into Presence<Vec<_>>, summed and multiplied
Then  each result is Absent if any element is Absent
And   otherwise Null if any element is Null
And   otherwise Some of the collected values, sum and product
```

### Scenario: Iteration stops at the first Absent and otherwise reads every element

```
Given any list of presences (property)
When  it is collected, summed or multiplied through an iterator that counts the elements it yields
Then  the count is the position of the first Absent plus one, if there is an Absent
And   otherwise the count is the length of the list, even when a Null came first
```

### Scenario: A Null followed by an Absent still gives Absent

```
Given the list [Some(1), Null, Some(2), Absent]
When  it is collected, summed and multiplied
Then  every result is Absent
```

### Scenario: Values before a Null are combined as they arrive

```
Given the list [Some(i32::MAX), Some(1), Null]
When  it is summed in a debug build
Then  the sum panics with an overflow, as Option's Sum does
```

### Scenario: The library builds without std

```
Given the no_std target thumbv7em-none-eabihf
When  the library is built for it without features and with the serde feature
Then  both builds succeed
```

---

## Tasks

Each task lists its scenarios, the test types that pin it (unit /
property / acceptance), and — when it depends on earlier tasks — a
`Needs:` naming them. Tasks with no `Needs:` are roots; tasks whose needs
are all done are ready; the graph is what `scripts/keeler-graph.sh` reads.

- [x] **T1 — Stream collect, sum and product through one adapter.** Scenarios: _Streaming keeps the Absent over Null over Some result; Iteration stops at the first Absent and otherwise reads every element; A Null followed by an Absent still gives Absent; Values before a Null are combined as they arrive_. Tests: acceptance (in `tests/acceptance.rs` under `// Spec 04 — streaming`) + property (consumed count is first `Absent` + 1, else the full length; the result matches the precedence rule — the spec 03 property test must keep passing unchanged). Replaces the three `Vec` buffers in `src/presence.rs` with one private adapter; documents the overflow edge case on `Sum`/`Product`; CHANGELOG `Changed` entry.
- [x] **T2 — Prove that streaming allocates nothing.** Needs: T1. Scenarios: _Summing and multiplying presences allocates nothing; Collecting into a non-allocating type allocates nothing_. Tests: acceptance in a new `tests/allocation.rs` binary with a counting `#[global_allocator]`.
- [x] **T3 — Build without std.** Needs: T1. Scenarios: _The library builds without std_. Tests: a `no_std` CI job in `.github/workflows/ci.yml` building for `thumbv7em-none-eabihf` without features and with `serde`, run locally before pushing. Adds `#![no_std]` to `src/lib.rs`, moves `std::` paths to `core::`, sets the serde dependency to `default-features = false`, and adds the CHANGELOG `Added` entry.

---

## Implementation Notes

### Approach

- **One shared adapter,** private to the module, used by all three impls.
  It wraps the source iterator by `&mut` and:
  - yields `T` for each `Some`;
  - stops at the first `Null` (recording it) or `Absent` (recording it and
    stopping for good).

  The impl hands the adapter to `V::from_iter`, `T::sum` or `T::product`.
  Afterwards, if a `Null` was seen and no `Absent`, it drains the rest of
  the source looking for an `Absent`, and stops at the first one. Result:
  `Absent` if one was seen, else `Null` if one was seen, else `Some(value)`.
  This keeps cyclomatic complexity low by putting the state machine in one
  place rather than three.
- **`#![no_std]`:** add it in `src/lib.rs` and change `std::` paths to
  `core::` throughout `src/`. Unit tests in `#[cfg(test)]` and integration
  tests may still use `std`: put `extern crate std;` inside the test module
  where it's needed.
- **Allocation test:** a separate integration test binary,
  `tests/allocation.rs`, with a `#[global_allocator]` wrapper around
  `std::alloc::System` that counts allocations. Each scenario reads the
  count right before and right after the call under test.
  - Put the counter in a thread-local, or read it around a call that runs
    on one thread, so that test-harness threads don't interfere.
  - Build the input first (that `Vec` allocates), then count only around
    `sum()`, `product()` and `collect()`.
- **No-std CI job:** add `msrv`-style job steps (or a new job, `no_std`)
  that install the `thumbv7em-none-eabihf` target and run
  `cargo build --lib --target thumbv7em-none-eabihf`, without features and
  with `--features serde`.
- **Overflow scenario:** `#[should_panic]` on the debug-build test; tests
  run in the dev profile, so overflow checks are on.

### Invariants worth a property test

- The result state follows `Absent` over `Null` over `Some`, and `Some`
  values match the buffered computation. The spec 03 test
  `collect_sum_and_product_use_the_same_precedence_as_zip` already pins
  this, and the "Streaming keeps…" scenario owns it here; it must keep
  passing unchanged.
- The number of elements consumed is first `Absent` + 1, else the full
  length.

### Non-goals

- `std` or `alloc` feature flags.
- Changing the `Absent` over `Null` over `Some` rule, or which elements
  are consumed.
- Avoiding the prefix computation, which would need a buffer.
- Benchmarks.
- Other combinators or APIs; the Merge Patch helper is its own spec.
