# Spec 07: `unwrap_or_absent_or_null` and its lazy form

**Status:** Implemented
**Effort:** Small
**Module:** `src/presence.rs`, `tests/acceptance.rs`, `CHANGELOG.md`

## Context

`Presence::unwrap_or_null_default(absent_default, null_default)` returns the
value of `Some`, or one default for `Absent` and another for `Null`. The name
mentions only `null`, yet the first argument is the `Absent` default. Both
arguments are `T`, so a caller who reads the name as "null default first"
writes `p.unwrap_or_null_default(null_value, absent_value)`, which compiles
and silently swaps the two. This is finding M6 of the 2026-09-14 review.

The method has been public since 0.1.0, so it cannot be removed. It becomes a
deprecated alias of a new method whose name states the argument order, as
spec 03 did for `to_nested_option`, `reduce`, `is_null_or` and `Item`.

A lazy form is added at the same time, for defaults that are costly to build.
It calls only the closure for the state it meets.

Decisions (settled in the spec interview):

1. **`unwrap_or_absent_or_null(self, absent: T, null: T) -> T`**. The
   arguments keep the old order, `Absent` first, which the name now spells
   out and which matches the documented ordering `Absent < Null < Some(_)`.
   Callers moving off the alias rename the call and keep the arguments.
2. **`unwrap_or_else_absent_or_null(self, absent: FA, null: FN) -> T`**, with
   `FA: FnOnce() -> T` and `FN: FnOnce() -> T`. It pairs with
   `unwrap_or_absent_or_null` as `unwrap_or_else` pairs with `unwrap_or`.
3. **`unwrap_or_null_default` stays**, with the same behavior and argument
   order, delegating to `unwrap_or_absent_or_null`. It carries
   `#[deprecated(since = "0.3.0")]` with a note that names the replacement
   and says the first argument is the `Absent` default. When it is removed
   is not decided here.
4. **Ships in 0.3.0.** 0.3.0 is not tagged or published yet, and adding a
   method and deprecating another breaks no caller. The entries go under the
   existing `[0.3.0]` heading in `CHANGELOG.md`.

Defaults the spec takes (approval confirms them):

- Both new methods are `#[inline]` and `#[must_use]` with the same message
  as `unwrap_or` and `unwrap_or_else`. The deprecated alias keeps its
  current attributes plus `#[deprecated]`.
- The crate docs example, the method doctests and
  `tests/transformation_tests.rs` move to the new name. Only the
  deprecated-names acceptance scenario keeps the old one, under
  `#[expect(deprecated, reason = ...)]`.
- The `[0.3.0]` date in `CHANGELOG.md` stays as is here. It is set to the
  real release day when 0.3.0 is tagged, which is outside this spec.

---

## Acceptance Tests

### Scenario: unwrap_or_absent_or_null returns the value of Some

```
Given Presence::Some(v) for any i32 v, and any i32 defaults a and n (property)
When  unwrap_or_absent_or_null(a, n) is called
Then  the result is v
```

### Scenario: unwrap_or_absent_or_null returns the first argument for Absent

```
Given Presence::<i32>::Absent and any defaults a and n (property)
When  unwrap_or_absent_or_null(a, n) is called
Then  the result is a
```

### Scenario: unwrap_or_absent_or_null returns the second argument for Null

```
Given Presence::<i32>::Null and any defaults a and n (property)
When  unwrap_or_absent_or_null(a, n) is called
Then  the result is n
```

### Scenario: The lazy form returns what the eager form returns

```
Given any Presence<i32> p and any defaults a and n (property)
When  unwrap_or_else_absent_or_null(|| a, || n) is called on p
Then  the result equals p.unwrap_or_absent_or_null(a, n)
```

### Scenario: The lazy form calls at most the closure for the state it meets

```
Given a Presence<i32> in each of the states Some, Null and Absent
When  unwrap_or_else_absent_or_null is called with two closures that count their calls
Then  for Some neither closure is called
And   for Absent only the first closure is called, exactly once
And   for Null only the second closure is called, exactly once
```

### Scenario: The deprecated name behaves exactly like its replacement

```
Given any Presence<i32> p and any defaults a and n (property)
When  p.unwrap_or_null_default(a, n) is called
Then  the result equals p.unwrap_or_absent_or_null(a, n)
And   calling unwrap_or_null_default raises a deprecation warning that names unwrap_or_absent_or_null
```

---

## Tasks

- [x] **T1 - Add `unwrap_or_absent_or_null`.** Scenarios: _unwrap_or_absent_or_null returns the value of Some; unwrap_or_absent_or_null returns the first argument for Absent; unwrap_or_absent_or_null returns the second argument for Null_. Tests: acceptance + property (each state for any value and defaults, and `p.unwrap_or_absent_or_null(d, d) == p.unwrap_or(d)`), plus a doctest with distinct defaults. Adds the method directly below `unwrap_or_null_default` in `src/presence.rs` and its line under `### Added` in the `[0.3.0]` CHANGELOG entry.
- [x] **T2 - Add the lazy form `unwrap_or_else_absent_or_null`.** Needs: T1. Scenarios: _The lazy form returns what the eager form returns; The lazy form calls at most the closure for the state it meets_. Tests: acceptance + property (agreement with `unwrap_or_absent_or_null` for any `p`, `a`, `n`) + unit (closure call counts per state), plus a doctest. Adds the method directly below `unwrap_or_absent_or_null` and a second line under `### Added`.
- [x] **T3 - Deprecate `unwrap_or_null_default` and move callers to the new name.** Needs: T1, T2. Scenarios: _The deprecated name behaves exactly like its replacement_. Tests: acceptance + property (agreement with `unwrap_or_absent_or_null`, under `#[expect(deprecated, reason = ...)]`). Turns the old body into a call to `unwrap_or_absent_or_null`, adds `#[deprecated(since = "0.3.0")]`, moves the crate docs example, the method's doctest and `tests/transformation_tests.rs` to the new name, and adds a line under `### Deprecated`.

---

## Implementation Notes

- `unwrap_or_absent_or_null` takes over the current three-arm `match` of
  `unwrap_or_null_default`, whose body becomes a call to it.
- `unwrap_or_else_absent_or_null` is the same `match` with `absent()` and
  `null()` in the empty arms.
- Doc comments name each argument by state ("`absent` is returned for
  `Absent`"), and the doctests use distinct values for the two defaults so
  a swap would fail them.
- The deprecation note, for example: "renamed to `unwrap_or_absent_or_null`,
  which takes the `Absent` default first".
- The deprecated-names acceptance scenario can join the one spec 03 added in
  `tests/acceptance.rs` or stand beside it. The warning itself is checked by
  the `#[expect(deprecated)]` on that test: without the attribute the build
  fails under `-D warnings`, and with it a missing deprecation fails as an
  unfulfilled expectation.
- CHANGELOG `[0.3.0]`: both methods under `### Added`,
  `unwrap_or_null_default` under `### Deprecated`.

### Invariants worth a property test

- For every `p`, `a`, `n`: `p.unwrap_or_else_absent_or_null(|| a, || n) ==
  p.unwrap_or_absent_or_null(a, n)`.
- For every `p`, `a`, `n`: `p.unwrap_or_null_default(a, n) ==
  p.unwrap_or_absent_or_null(a, n)`.
- For every `p` and `d`: `p.unwrap_or_absent_or_null(d, d) == p.unwrap_or(d)`,
  which ties the new method to the existing single-default one.

### Non-goals

- Removing `unwrap_or_null_default` or announcing a removal version.
- A `Default`-based form (`unwrap_or_default` already covers one default for
  both states).
- Variants that return `Presence` or `Option` instead of `T`, or that take
  `&self`.
- Tagging or publishing 0.3.0, or changing its release date.
