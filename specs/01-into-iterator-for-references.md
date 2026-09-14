# Spec 01 — `IntoIterator` for `&Presence<T>` and `&mut Presence<T>`

**Status:** Approved
**Effort:** Small
**Module:** `src/presence.rs`

## Context

`Presence<T>` mirrors `std::option::Option`, but unlike `Option` it cannot be
iterated by reference in a `for` loop: `for x in &presence` and
`for x in &mut presence` do not compile, so users must call `presence.iter()`
or `presence.iter_mut()` explicitly. Code ported from `Option` breaks, and
generic code bounded on `IntoIterator` (e.g. `fn f<I: IntoIterator>(i: I)`)
cannot accept a borrowed `Presence`.

The gap is also visible to tooling: pedantic clippy flags `iter()` and
`iter_mut()` with `iter_without_into_iter`, currently silenced by two
`#[expect(...)]` attributes added during the Keeler cleanup.

Everything needed already exists: `Presence::iter()` returns `Iter<'_, T>` and
`Presence::iter_mut()` returns `IterMut<'_, T>` (both exact-size,
double-ended and fused). The change is purely additive — `for x in &presence`
did not compile before, so no existing code changes meaning.

Decisions (proposed in the spec interview, accepted by the user):

1. `&Presence<T>` yields `&T` through the existing `Iter<'a, T>`;
   `&mut Presence<T>` yields `&mut T` through the existing `IterMut<'a, T>`.
   No new public types. `Some` yields exactly one item; `Null` and `Absent`
   yield none — identical to `iter()` / `iter_mut()`.
2. Both impls get doc examples (doctests) covering `Some`, `Null` and `Absent`.
3. `CHANGELOG.md` gets an `## [Unreleased]` section with an `### Added` entry;
   no version bump.
4. Non-goals are listed below.

---

## Acceptance Tests

### Scenario: Iterating a borrowed Some yields a reference to its value

```
Given a Presence::Some(42)
When  it is iterated with `for x in &presence`
Then  the loop body runs exactly once with x == &42
And   the presence is still usable afterwards and equals Presence::Some(42)
```

### Scenario: Iterating a borrowed Null yields nothing

```
Given a Presence::<i32>::Null
When  it is iterated with `for x in &presence`
Then  the loop body never runs
And   the presence still equals Presence::Null
```

### Scenario: Iterating a borrowed Absent yields nothing

```
Given a Presence::<i32>::Absent
When  it is iterated with `for x in &presence`
Then  the loop body never runs
And   the presence still equals Presence::Absent
```

### Scenario: Iterating a mutably borrowed Some allows modifying its value

```
Given a mutable Presence::Some(42)
When  it is iterated with `for x in &mut presence` and the body sets *x = 100
Then  the loop body runs exactly once
And   the presence afterwards equals Presence::Some(100)
```

### Scenario: Iterating a mutably borrowed Null or Absent yields nothing and leaves the state unchanged

```
Given a mutable Presence::<i32>::Null and a mutable Presence::<i32>::Absent
When  each is iterated with `for x in &mut presence`
Then  the loop body never runs for either
And   they still equal Presence::Null and Presence::Absent respectively
```

### Scenario: A borrowed Presence is accepted where IntoIterator is required

```
Given a function generic over `I: IntoIterator` that collects its items into a Vec
When  it is called with `&Presence::Some(7)`, `&Presence::<i32>::Null` and `&Presence::<i32>::Absent`
Then  it returns vec![&7], an empty Vec and an empty Vec respectively
```

### Scenario: Reference iteration matches iter() and iter_mut()

```
Given any Presence<i32> (Some of any value, Null, or Absent)
When  `(&presence).into_iter()` is compared with `presence.iter()`
And   `(&mut presence).into_iter()` is compared with `presence.iter_mut()`
Then  both pairs yield the same items and report the same len() and size_hint()
```

### Scenario: The clippy suppression is no longer needed

```
Given the crate with both impls in place and the two
      `#[expect(clippy::iter_without_into_iter)]` attributes removed
When  `cargo clippy --all-targets -- -D warnings` runs with pedantic lints
Then  it reports no warnings
```

---

## Tasks

---

## Implementation Notes

- Add, next to the existing `impl<T> IntoIterator for Presence<T>`:

  ```rust
  impl<'a, T> IntoIterator for &'a Presence<T> {
      type Item = &'a T;
      type IntoIter = Iter<'a, T>;
      fn into_iter(self) -> Iter<'a, T> { self.iter() }
  }

  impl<'a, T> IntoIterator for &'a mut Presence<T> {
      type Item = &'a mut T;
      type IntoIter = IterMut<'a, T>;
      fn into_iter(self) -> IterMut<'a, T> { self.iter_mut() }
  }
  ```

  Each with a doc comment and a doctest for `Some`, `Null` and `Absent`,
  matching the style of the existing `IntoIterator for Presence<T>`.
- Remove both `#[expect(clippy::iter_without_into_iter, ...)]` attributes on
  `Presence::iter` and `Presence::iter_mut`. With the impls present, an
  unfulfilled `#[expect]` would itself warn, so removal is required, not
  optional.
- Update the `Iter` / `IterMut` struct docs to mention they are also created
  by `IntoIterator` for references.
- Add `## [Unreleased]` → `### Added` to `CHANGELOG.md`.
- **Property to test (proptest):** for every `Presence<i32>` generated from
  `Some(any::<i32>())`, `Null`, `Absent`: `(&p).into_iter().collect::<Vec<_>>()
  == p.iter().collect::<Vec<_>>()`, `len()` agrees, and the item count is
  `1` iff `p.is_present()`, else `0`; likewise for `&mut p` vs `iter_mut()`.
- Acceptance tests go in `tests/acceptance.rs`, one per scenario, named after it.

### Non-goals

- No version bump or release.
- No `IntoIterator` for `Pin<&Presence<T>>` or other smart-pointer wrappers.
- No change to the behavior of `iter()`, `iter_mut()`, `Iter`, `IterMut` or
  the consuming `IntoIterator for Presence<T>`.
- None of the other review findings (e.g. `collect`/`sum`/`product`
  short-circuiting on `Null`, `From<Option<T>>`).
