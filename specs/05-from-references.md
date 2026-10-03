# Spec 05 — `From<&Presence<T>>` and `From<&mut Presence<T>>`

**Status:** Implemented
**Effort:** Small
**Module:** `src/presence.rs`, `CHANGELOG.md`

## Context

std has `From<&'a Option<T>> for Option<&'a T>` and
`From<&'a mut Option<T>> for Option<&'a mut T>`. With them, an API can
take `impl Into<Option<&T>>` and be called with either `Some(&x)` or
`&option`. `Presence` has `as_ref()` and `as_mut()` but not these impls,
so code ported from `Option`, and generic code bounded on
`Into<Presence<&T>>`, cannot pass a borrowed `Presence`.

The change only adds API: `(&presence).into()` into `Presence<&T>` did not
compile before. It also doesn't overlap with the blanket
`From<T> for Presence<T>`. An overlap would need `T == Presence<T>`, and
std's `Option` has the same pair of impls next to its own blanket
`From<T>`.

Who benefits: anyone writing functions that accept "a presence of a
reference" generically, and anyone porting `Option` code.

Decisions (no open questions; std's `Option` sets the design):

1. Add both impls. Each is a thin wrapper over `as_ref()` / `as_mut()`, so
   the state is kept: `Some(x)` becomes `Some(&x)`, while `Null` and
   `Absent` stay as they are.
2. Each impl gets a doc example covering all three states.
3. CHANGELOG: `### Added` under `[Unreleased]`. No version bump.

---

## Acceptance Tests

### Scenario: Converting a borrowed presence gives a presence of a reference

```
Given any presence p (property)
When  Presence::from(&p) is evaluated
Then  the result equals p.as_ref()
And   p is still usable afterwards and unchanged
```

### Scenario: Converting a mutably borrowed presence allows changing the value

```
Given a mutable Presence::Some(41)
When  it is converted with Presence::from(&mut presence) and the value is incremented through it
Then  the presence afterwards equals Presence::Some(42)
And   converting a mutable Null or Absent the same way leaves it unchanged
```

### Scenario: A borrowed presence is accepted where Into<Presence<&T>> is required

```
Given a function generic over `impl Into<Presence<&i32>>` that reports the state it receives
When  it is called with `&Presence::Some(7)`, `&Presence::<i32>::Null` and `&Presence::<i32>::Absent`
Then  it receives Some(&7), Null and Absent respectively
```

---

## Tasks

Each task lists its scenarios, the test types that pin it (unit /
property / acceptance), and — when it depends on earlier tasks — a
`Needs:` naming them. Tasks with no `Needs:` are roots; tasks whose needs
are all done are ready; the graph is what `scripts/keeler-graph.sh` reads.

- [x] **T1 — `From<&Presence<T>> for Presence<&T>`.** Scenarios: _Converting a borrowed presence gives a presence of a reference; A borrowed presence is accepted where Into<Presence<&T>> is required_. Tests: acceptance + property (`Presence::from(&p) == p.as_ref()`), plus a doctest on the impl. Adds the impl after `From<T>` in `src/presence.rs` and the CHANGELOG `Added` entry.
- [x] **T2 — `From<&mut Presence<T>> for Presence<&mut T>`.** Needs: T1. Scenarios: _Converting a mutably borrowed presence allows changing the value_. Tests: acceptance, plus a doctest on the impl. Adds the impl directly after T1's (same region, hence the dependency) and extends the CHANGELOG entry.

---

## Implementation Notes

- Put both impls in the `From` section of `src/presence.rs`, next to
  `From<T>`. Bodies: `presence.as_ref()` and `presence.as_mut()`.
- Tests go in `tests/acceptance.rs` under `// Spec 05 — From references`.
  The first scenario is a proptest using the existing `any_presence()`
  strategy.

### Invariants worth a property test

- `Presence::from(&p) == p.as_ref()` for every `p`.

### Non-goals

- Other conversions (`From<Option<T>>`, `TryFrom`, `AsRef` impls).
- Changes to `as_ref` / `as_mut`.
- A version bump.
