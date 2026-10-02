# Spec 03 — API consistency: documented state rules, clearer names

**Status:** Implemented
**Effort:** Medium
**Module:** `src/presence.rs`, `src/lib.rs` (docs), `Cargo.toml` (lint), `CHANGELOG.md`

## Context

`Presence<T>` has three states. Every combinator therefore has to decide what
happens when the states mix, and that decision is the crate's core design.
Today it exists only in the code. There are several rules, nothing says which
method follows which, and one doc is wrong:

- **The state of `self` decides** in `map`, `and`, `and_then`, `filter`,
  `flatten`, `copied`, `cloned`, `transpose` and `unzip`. A `Null` or
  `Absent` `self` passes through unchanged, so `Null.and(Absent) == Null`.
- **First `Some` wins** in `or` and `or_else`. When `self` isn't `Some`, the
  argument is returned whatever its state, so `Null.or(Absent) == Absent`.
- **`Absent` takes precedence over `Null`, which takes precedence over
  `Some`,** in `zip`, `zip_with`, `collect` (`FromIterator`), `sum` and
  `product`. So `Null.zip(Absent) == Absent`. The `zip` doc says the result
  is `Null` only "if both are `Null`", which is wrong:
  `Some(1).zip(Null) == Null`.
- **`xor`** returns the value when exactly one side is `Some`, `Null` when
  both are `Null`, and `Absent` otherwise, including `Some` xor `Some`.
- **`filter`** turns a `Some` that fails the predicate into `Absent`, and
  **`take_if`** returns `Absent` when it takes nothing. Neither says why.

Names that mislead or duplicate:

- `to_nested_option` is identical to `to_nullable`.
- `reduce` is identical to `zip_with`, and its name suggests
  `Iterator::reduce`.
- `is_null_or(f)` is also `true` for `Absent`. The doc says so, but the name
  doesn't. The crate already uses `is_nullish()` for "`Null` or `Absent`".
- The owning iterator is called `Item`; std calls it `IntoIter`.

Behaviour that users rely on without any guarantee:

- The derived ordering `Absent < Null < Some(_)` is undocumented.
- The blanket `From<T> for Presence<T>` makes
  `let p: Presence<Option<i32>> = None.into()` equal to `Some(None)`. std's
  `Option` has the same trap, and nothing warns about it.

Safety documentation: the `unsafe` blocks in `as_pin_ref` and `as_pin_mut`
have no `// SAFETY:` comments, and nothing enforces one.

Who hits this: anyone who combines `Presence` values and has to guess the
rule, or reads a doc that contradicts the code. Desired outcome: every
combinator's rule is written down once and pinned by property tests, the
names say what the methods do, and existing code keeps compiling, with
warnings at most.

Decisions (settled in the spec interview):

1. **No behaviour changes.** The existing rules are documented and pinned
   by property tests, not unified.
2. `filter`, `xor(Some, Some)` and `take_if` keep returning `Absent`. The
   docs explain why: `Absent` means "leave unchanged" in PATCH terms, which
   is the safe default, whereas `Null` would mean "clear the field".
3. **Renames ship as deprecations.** The new names `is_nullish_or` and
   `IntoIter` are added. The old names `to_nested_option`, `reduce`,
   `is_null_or` and `Item` stay as `#[deprecated(since = "0.3.0")]`
   aliases pointing to their replacements (`to_nullable`, `zip_with`,
   `is_nullish_or`, `IntoIter`), to be removed in a later release.
4. `Iter`, `IterMut` and `IntoIter` stay in `presence_rs::presence` and are
   not re-exported at the crate root, like `std::option::IntoIter`.
5. The blanket `From<T>` stays. Its doc points `Option` inputs to
   `from_optional` and `from_nullable`.
6. The ordering `Absent < Null < Some(_)`, with `Some` values compared by
   value, becomes a documented guarantee pinned by a property test.
7. `// SAFETY:` comments are added, and `clippy::undocumented_unsafe_blocks`
   is enabled.
8. CHANGELOG entries go under `[Unreleased]` (Added, Deprecated,
   Documentation). There's no version bump.

---

## Acceptance Tests

"Any presence" means `Some` of any value, `Null` or `Absent`. Scenarios
marked *(property)* hold for all generated inputs.

### Scenario: A Null or Absent receiver passes through the self-decides combinators unchanged

```
Given any presence p that is Null or Absent (property)
And   any presence q and any function f
When  p.map(f), p.and(q), p.and_then(f) and p.filter(f) are evaluated
Then  each result is in the same state as p
And   an outer Null or Absent of type Presence<Presence<T>> flattens to the same state
```

### Scenario: and on Some returns the argument unchanged

```
Given p = Some(x) for any x and any presence q (property)
When  p.and(q) is evaluated
Then  the result equals q
```

### Scenario: or returns self when Some, otherwise the argument unchanged

```
Given any presences p and q (property)
When  p.or(q) is evaluated
Then  the result is p if p is Some, otherwise exactly q
And   Null.or(Absent) is Absent and Absent.or(Null) is Null
```

### Scenario: zip gives Absent precedence over Null and Null over Some

```
Given any presences p and q (property)
When  p.zip(q) is evaluated
Then  the result is Some((a, b)) if both are Some
And   otherwise Absent if either is Absent
And   otherwise Null
And   swapping p and q swaps the tuple but not the state
```

### Scenario: collect, sum and product use the same precedence as zip

```
Given any list of presences (property)
When  it is collected into Presence<Vec<_>>, summed and multiplied
Then  each result is Absent if any element is Absent
And   otherwise Null if any element is Null
And   otherwise Some of the collected values, sum and product
```

### Scenario: xor returns the single Some, Null for two Nulls, and Absent otherwise

```
Given any presences p and q (property)
When  p.xor(q) is evaluated
Then  the result is the Some side if exactly one side is Some
And   Null if both are Null
And   Absent in every other case, including Some xor Some
```

### Scenario: filter turns a failing Some into Absent

```
Given Some(3) and a predicate that rejects it
When  filter is applied
Then  the result is Absent, not Null
```

### Scenario: take_if that takes nothing returns Absent and leaves the presence unchanged

```
Given Some(10), Null and Absent, and a predicate that rejects 10
When  take_if is called on each
Then  each call returns Absent
And   each presence keeps its original state
```

### Scenario: is_nullish_or is true for Null and Absent and tests the value of Some

```
Given any presence p and a predicate f (property)
When  p.is_nullish_or(f) is evaluated
Then  it is true for Null and Absent
And   it equals f(x) for Some(x)
```

### Scenario: Deprecated names still work and match their replacements

```
Given any presence p (property)
When  to_nested_option, reduce, is_null_or and the Item type are used with deprecation allowed
Then  to_nested_option equals to_nullable
And   reduce equals zip_with
And   is_null_or equals is_nullish_or
And   a value of type Item<T> is the iterator into_iter returns
```

### Scenario: into_iter returns presence::IntoIter

```
Given a Presence::Some(1)
When  it is converted with into_iter
Then  the iterator has type presence_rs::presence::IntoIter<i32>
And   it yields 1 once
```

### Scenario: Presences are ordered Absent, then Null, then Some by value

```
Given any presences p and q (property)
When  they are compared
Then  Absent < Null < Some(x) for every x
And   Some(a).cmp(&Some(b)) equals a.cmp(&b)
```

### Scenario: Converting an Option into a Presence of Option wraps it

```
Given None of type Option<i32>
When  it is converted with into() into Presence<Option<i32>>
Then  the result is Some(None), not Null or Absent
```

---

## Tasks

Each task lists its scenarios, the test types that pin it (unit /
property / acceptance), and — when it depends on earlier tasks — a
`Needs:` naming them. Tasks with no `Needs:` are roots; tasks whose needs
are all done are ready; the graph is what `scripts/keeler-graph.sh` reads.
Acceptance tests go in `tests/acceptance.rs` under a `// Spec 03 — <task name>`
heading per task; each task adds its own CHANGELOG line under the matching
`[Unreleased]` subsection.

- [x] **T1 — Document and pin "self decides" and "first Some wins".** Scenarios: _A Null or Absent receiver passes through the self-decides combinators unchanged; and on Some returns the argument unchanged; or returns self when Some, otherwise the argument unchanged_. Tests: acceptance + property (a non-`Some` receiver's state survives `map`/`and`/`and_then`/`filter`/`flatten`; `Some(x).and(q) == q`; `p.or(q)` is `p` if `Some` else `q`). Creates the shared "any presence" proptest strategy in `tests/acceptance.rs`, and the `# Combining states` section in the `presence` module docs with the rule table; adds a one-line rule note to the doc of each method covered here.
- [x] **T2 — Document and pin the Absent > Null > Some precedence.** Needs: T1. Scenarios: _zip gives Absent precedence over Null and Null over Some; collect, sum and product use the same precedence as zip_. Tests: acceptance + property (`zip` state is symmetric and equals the state of `collect` over `[p, q]`; `collect`/`sum`/`product` agree on state for any list). Fixes the wrong `zip` doc; adds rule notes to `zip`, `zip_with`, `FromIterator`, `Sum`, `Product` and their row in the T1 table.
- [x] **T3 — Document and pin the Absent results of xor, filter and take_if.** Needs: T2. Scenarios: _xor returns the single Some, Null for two Nulls, and Absent otherwise; filter turns a failing Some into Absent; take_if that takes nothing returns Absent and leaves the presence unchanged_. Tests: acceptance + property (the `xor` table for any pair). Adds the "Absent means leave unchanged" rationale to the docs of the three methods and the `xor` row of the table.
- [x] **T4 — Rename the owning iterator to `IntoIter`.** Scenarios: _into_iter returns presence::IntoIter_. Tests: acceptance. Renames `Item` to `IntoIter` in the iterator section of `src/presence.rs` and adds the deprecated `pub type Item<A> = IntoIter<A>;` alias.
- [x] **T5 — Add `is_nullish_or` and deprecate the duplicate names.** Needs: T3, T4. Scenarios: _is_nullish_or is true for Null and Absent and tests the value of Some; Deprecated names still work and match their replacements_. Tests: acceptance + property (`is_nullish_or` truth table; each of `to_nested_option`, `reduce`, `is_null_or` equals its replacement; `Item<T>` is the type `into_iter` returns), plus a doctest on `is_nullish_or`. Turns `is_null_or`, `to_nested_option` and `reduce` into `#[deprecated(since = "0.3.0")]` wrappers and moves existing tests and doctests off the old names.
- [x] **T6 — Guarantee the ordering and document the From<T> trap.** Needs: T1. Scenarios: _Presences are ordered Absent, then Null, then Some by value; Converting an Option into a Presence of Option wraps it_. Tests: acceptance + property (state order, and `T`'s order inside `Some`). Adds the `Ord` guarantee to the `Presence` type docs and the note to the `From<T>` impl doc.
- [x] **T7 — Document the unsafe blocks and enforce it.** Scenarios: _none (enforced by clippy)_. Tests: none beyond the lint gate. Adds `// SAFETY:` comments in `as_pin_ref` and `as_pin_mut` and `undocumented_unsafe_blocks = "warn"` under `[lints.clippy]` in `Cargo.toml`.

---

## Implementation Notes

### Approach

- **Docs:** a `# Combining states` section in the `presence` module docs
  with one table: which methods follow which rule (self decides, first
  `Some` wins, `Absent` over `Null` over `Some`, `xor`). Each combinator's
  own doc names its rule and links to the section. Fix the wrong `zip` doc
  in the same pass.
- **Renames:**
  - Rename `Item` to `IntoIter` and add
    `#[deprecated(since = "0.3.0", note = "renamed to IntoIter")] pub type Item<A> = IntoIter<A>;`.
  - Add `is_nullish_or`, and make `is_null_or` a deprecated wrapper that
    calls it.
  - `to_nested_option` and `reduce` become deprecated wrappers of
    `to_nullable` and `zip_with`.
  - Existing tests and doctests that call the old names move to the new
    names. Only the "deprecated names" scenario keeps the old ones, under
    `#[expect(deprecated, reason = ...)]`.
- **`From<T>` doc:** a note that for `Option` inputs, `from_optional` and
  `from_nullable` express intent, and that `None.into()` into
  `Presence<Option<_>>` is `Some(None)`.
- **`Ord` doc:** the guarantee goes on the `Presence` type docs.
- **Unsafe:** add `// SAFETY:` comments to both `unsafe` blocks, and add
  `undocumented_unsafe_blocks = "warn"` to `[lints.clippy]`. CI runs clippy
  with `-D warnings`, so a missing comment fails the build.
- **CHANGELOG:** under `[Unreleased]`:
  - `### Added`: `is_nullish_or`, `IntoIter`.
  - `### Deprecated`: the four old names.
  - `### Documentation`: the state rules, the `Ord` guarantee and the
    `From` note.

### Tests

- Acceptance tests go in `tests/acceptance.rs` under `// Spec 03 — …`
  headings, one per scenario. The property scenarios use `proptest` with
  the existing persistence configuration, and a shared strategy for "any
  presence" that also covers `Some(0)` and negative values.
- Where a scenario lists several methods, one test may check all of them.

### Invariants worth a property test

- Self decides: if `p` is not `Some`, `op(p, …)` has `p`'s state.
- `zip`'s state equals the state of `collect` over `[p, q]`. This ties the
  two definitions of the precedence rule together.
- `zip` is symmetric in state.
- `Ord` is consistent with the state order, and with `T`'s order inside
  `Some`.
- Each deprecated alias equals its replacement.

### Non-goals

- Any change to a combinator's result.
- Removing the deprecated names, which belongs to a later release.
- Re-exporting iterator types at the crate root.
- Removing or restricting `From<T> for Presence<T>`.
- A version bump.
- New combinators (for example a JSON Merge Patch `apply`), allocation-free
  `FromIterator`/`Sum`/`Product`, or `no_std`. Each is its own spec.
