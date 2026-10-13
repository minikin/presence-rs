# Spec 06: Applying and composing patches

**Status:** Implemented
**Effort:** Small
**Module:** `src/presence.rs`, `README.md`, `CHANGELOG.md`

## Context

`Presence` exists mostly for PATCH-style updates, as in JSON Merge Patch
(RFC 7396). Each field of a request is either left out (`Absent`: leave the
stored value alone), explicitly `null` (`Null`: clear it), or a value
(`Some`: set it). Every user writes the same three-arm `match` to apply such
a field to their model, where the field is usually an `Option<T>`. The
README's own "Practical Example" is that hand-written `match`.

The second need is folding a series of patches into one, for example
coalescing queued updates before a write. Here a later patch overrides an
earlier one unless the later one leaves the field out.

Desired outcome: two methods. `apply_to` applies a patch to an `Option<T>`
field and hands back what it replaced. `merge` composes two patches.

Decisions (settled in the spec interview):

1. **`patch.apply_to(&mut Option<T>) -> Option<T>`**:
   - `Absent` leaves the target unchanged and returns `None`;
   - `Null` sets it to `None`;
   - `Some(v)` sets it to `Some(v)`.

   Both of the last two return the target's previous value, like
   `Option::take` and `Option::replace`. A `None` return therefore means
   either "untouched" or "it was already `None`".
2. **`earlier.merge(later) -> Presence<T>`**: `later`, unless it is
   `Absent`, in which case `earlier`. This is a new state rule, "last
   non-`Absent` wins", added to the "Combining states" table.
3. Only `Option<T>` targets. Targets of type `T` or `Presence<T>` are
   non-goals.
4. **Name** *(amended after review, with the user's approval)*: the
   composing method was first named `then`. It is `merge` because `then`
   suggests a closure-taking combinator like `and_then`, and
   `Absent.and_then(..)` stays `Absent` while `Absent.merge(x)` is `x`.
   "Merge, later wins unless missing" is the usual meaning in
   config-layering crates.

Defaults the spec takes (approval confirms them):

- A new `# Patching` section in the `presence` module docs, with doctests.
- The README's "Practical Example" is rewritten around `apply_to`.
- CHANGELOG `### Added` under `[Unreleased]`. No version bump.

---

## Acceptance Tests

### Scenario: An Absent patch leaves the target unchanged

```
Given any target of type Option<i32> (property)
When  Presence::Absent is applied to it
Then  the target is unchanged
And   apply_to returns None
```

### Scenario: A Null patch clears the target and returns what it held

```
Given any target of type Option<i32> (property)
When  Presence::Null is applied to it
Then  the target is None
And   apply_to returns the target's previous value
```

### Scenario: A Some patch sets the target and returns what it held

```
Given any target of type Option<i32> and any value v (property)
When  Presence::Some(v) is applied to it
Then  the target is Some(v)
And   apply_to returns the target's previous value
```

### Scenario: merge keeps the later patch unless it is Absent

```
Given any presences earlier and later (property)
When  earlier.merge(later) is evaluated
Then  the result is earlier if later is Absent
And   otherwise the result is later
```

### Scenario: Applying a composed patch equals applying the patches in order

```
Given any target of type Option<i32> and any presences a and b (property)
When  a.merge(b) is applied to one copy of the target
And   a and then b are applied, one after the other, to another copy
Then  both copies end up equal
```

### Scenario: Composing patches is associative and Absent changes nothing

```
Given any presences a, b and c (property)
When  they are composed with merge
Then  a.merge(b).merge(c) equals a.merge(b.merge(c))
And   Absent.merge(a) and a.merge(Absent) both equal a
```

---

## Tasks

Each task lists its scenarios, the test types that pin it (unit /
property / acceptance), and, when it depends on earlier tasks, a
`Needs:` naming them. Tasks with no `Needs:` are roots; tasks whose needs
are all done are ready; the graph is what `scripts/keeler-graph.sh` reads.

- [x] **T1 - Apply a patch to an `Option` field.** Scenarios: _An Absent patch leaves the target unchanged; A Null patch clears the target and returns what it held; A Some patch sets the target and returns what it held_. Tests: acceptance + property (each case for any target), plus a doctest. Adds `apply_to`, the `# Patching` module docs section, and the CHANGELOG `Added` entry.
- [x] **T2 - Compose patches with `merge`.** Needs: T1. Scenarios: _merge keeps the later patch unless it is Absent; Applying a composed patch equals applying the patches in order; Composing patches is associative and Absent changes nothing_. Tests: acceptance + property (identity, associativity, agreement with sequential `apply_to`), plus a doctest. Adds `merge`, its row in the "Combining states" table, and extends the CHANGELOG entry.
- [x] **T3 - Rewrite the README's Practical Example around `apply_to`.** Needs: T1. Scenarios: _none (documentation; it runs as a doctest through `ReadmeDoctests`)_. Tests: the README doctest.

---

## Implementation Notes

- **`apply_to`**:
  `match self { Absent => None, Null => target.take(), Some(v) => target.replace(v) }`.
  No `#[must_use]`: applying the patch is the point of the call, and the
  returned old value is optional.
- **`merge`**: `match later { Absent => self, other => other }`. Add
  `#[must_use]`, matching `or`.
- **Docs:**
  - a `merge` row in the "Combining states" table ("last non-`Absent`
    wins");
  - a `# Patching` section showing a request struct applied to a model;
  - the README's "Practical Example" rewritten to use `apply_to`. It runs
    as a doctest via `ReadmeDoctests`.
- **Tests:** in `tests/acceptance.rs` under `// Spec 06: patching`, using
  `any_presence()` and an `any::<Option<i32>>()` target.

### Invariants worth a property test

- The three `apply_to` cases, for any target.
- `merge` is associative, and `Absent` is its identity on both sides (a
  monoid).
- Applying a composed patch equals applying the patches in order, which
  ties `merge` to `apply_to`.

### Non-goals

- `apply_to` for targets of type `T` or `Presence<T>`.
- Recursive or object-level merge (the rest of RFC 7396). That needs a
  JSON value type.
- serde helpers for patch structs.
- A version bump.
