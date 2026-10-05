# Spec 02: Serde round-trip: visible guidance and pinned behavior

**Status:** Implemented
**Effort:** Small
**Module:** `src/lib.rs`, `src/serde.rs`, `src/presence.rs` (docs), `README.md`, `Cargo.toml`, `CHANGELOG.md`

## Context

`Presence<T>` exists to keep three states apart: a field that is missing,
a field that is `null`, and a field with a value. With the `serde` feature
that only holds when a struct field carries **both** attributes:

- `#[serde(default)]`: a missing field deserializes as `Absent`;
- `#[serde(skip_serializing_if = "Presence::is_absent")]`: `Absent` is
  omitted from the output instead of written as `null`.

Without `#[serde(default)]`, serde treats a missing field exactly like an
explicit `null`, so the field silently deserializes as `Null` (verified with
`serde_json`). Without `skip_serializing_if`, `Absent` is written as `null`
and comes back as `Null`. In both cases the one distinction this crate
exists for is lost, and nothing fails.

Who hits it: anyone using `Presence<T>` in serde structs, which is the
crate's main use (JSON PATCH-style APIs). They follow the docs.rs page or the
README, and neither tells them the rule:

- The guidance lives only in the module docs of `src/serde.rs`, which is
  private (`mod serde;` in `src/lib.rs`), so rustdoc never renders it.
- `Cargo.toml` has no `[package.metadata.docs.rs]`. docs.rs builds without
  the `serde` feature, so even the `Serialize`/`Deserialize` impls are
  missing from the published docs.
- The README never mentions serde.
- The one example (`User` in `src/serde.rs`) leaves out
  `#[serde(default)]` and only checks serialization. That is exactly the
  broken configuration.

Desired outcome: a user reading docs.rs or the README sees the two-attribute
rule and a working example. Tests pin the correct round-trip for all three
states, and also pin both lossy configurations so that any change to them
is deliberate.

Decisions (settled in the spec interview):

1. **No behavior change.** A missing field without `#[serde(default)]` stays
   `Null`; this is documented as a pitfall and pinned by a test. Making it
   fail loudly was rejected: it means driving the deserializer through
   `deserialize_any` and rebuilding `T` from raw visitor callbacks. That is
   a large, risky change, and it breaks formats that don't describe
   themselves (bincode, postcard).
2. **Where the rule appears:** a "Serde" section in the crate-root docs,
   a short pointer in the `Presence` type docs, a "Serde" section in the
   README, and `[package.metadata.docs.rs] all-features = true` so the
   impls and the docs render on docs.rs. The private module's docs shrink
   to a pointer to the crate-root section, so the guidance lives in one
   place.
3. **Tests (JSON via `serde_json`):** round-trip of all three states with
   both attributes, plus the two lossy configurations pinned.
4. **CHANGELOG:** one entry under `## [Unreleased]` → `### Documentation`;
   no version bump.

---

## Acceptance Tests

All scenarios use JSON via `serde_json` and a struct
`{ name: String, age: Presence<u32> }`. "Both attributes" means
`age` carries `#[serde(default)]` and
`#[serde(skip_serializing_if = "Presence::is_absent")]`.

### Scenario: A Some field round-trips with both attributes

```
Given a struct whose `age` field has both attributes
And   age is Presence::Some(30)
When  it is serialized to JSON and deserialized back
Then  the JSON is {"name":"Alice","age":30}
And   the deserialized age equals Presence::Some(30)
```

### Scenario: A Null field round-trips with both attributes

```
Given a struct whose `age` field has both attributes
And   age is Presence::Null
When  it is serialized to JSON and deserialized back
Then  the JSON is {"name":"Bob","age":null}
And   the deserialized age equals Presence::Null
```

### Scenario: An Absent field round-trips with both attributes

```
Given a struct whose `age` field has both attributes
And   age is Presence::Absent
When  it is serialized to JSON and deserialized back
Then  the JSON is {"name":"Charlie"} with no age key
And   the deserialized age equals Presence::Absent
```

### Scenario: Every presence value survives a round-trip with both attributes

```
Given a struct whose `age` field has both attributes
And   age is any Presence<u32> (Some of any value, Null, or Absent)
When  it is serialized to JSON and deserialized back
Then  the deserialized struct equals the original
```

### Scenario: A missing field without serde(default) deserializes as Null

```
Given a struct whose `age` field has skip_serializing_if but no #[serde(default)]
When  the JSON {"name":"Charlie"} is deserialized
Then  deserialization succeeds
And   age equals Presence::Null, not Presence::Absent
```

### Scenario: An Absent field without skip_serializing_if comes back as Null

```
Given a struct whose `age` field has #[serde(default)] but no skip_serializing_if
And   age is Presence::Absent
When  it is serialized to JSON and deserialized back
Then  the JSON is {"name":"Charlie","age":null}
And   the deserialized age equals Presence::Null
```

---

## Tasks

Each task lists its scenarios, the test types that pin it (unit /
property / acceptance), and, when it depends on earlier tasks, a
`Needs:` naming them. Tasks with no `Needs:` are roots; tasks whose needs
are all done are ready; the graph is what `scripts/keeler-graph.sh` reads.

- [x] **T1 - Pin the three-state round-trip with both attributes.** Scenarios: _A Some field round-trips with both attributes; A Null field round-trips with both attributes; An Absent field round-trips with both attributes; Every presence value survives a round-trip with both attributes_. Tests: acceptance (in `tests/acceptance.rs` under a `// Spec 02: round-trip with both attributes` heading, with a fixture struct whose `age` field has both attributes) + property (`proptest`: for any `Presence<u32>`, `from_str(to_string(x)) == x` through the fixture struct).
- [x] **T2 - Pin the two lossy configurations.** Needs: T1. Scenarios: _A missing field without serde(default) deserializes as Null; An Absent field without skip_serializing_if comes back as Null_. Tests: acceptance (under a `// Spec 02: lossy configurations` heading directly after T1's block, each with its own fixture struct missing one attribute).
- [x] **T3 - Make the round-trip rule visible in the public docs.** Needs: T2. Scenarios: _none (documentation changes from Implementation Notes, checked in review)_. Tests: unit (a doctest in the crate-root `# Serde` section that deserializes and serializes all three states). Covers the crate-root `# Serde` section in `src/lib.rs`, the pointer in the `Presence` type docs, the reduced `src/serde.rs` module docs (removing the broken `User` example), a `## Serde` section in `README.md`, `[package.metadata.docs.rs] all-features = true` in `Cargo.toml`, and the `### Documentation` entry in `CHANGELOG.md`.

---

## Implementation Notes

### Tests

- Acceptance tests go in `tests/acceptance.rs` under a `// Spec 02` header,
  one per scenario, named after it, with Given/When/Then comments. `serde`
  (with `derive`) and `serde_json` are already dev-dependencies, and the
  dev-dependency on the crate itself enables the `serde` feature for every
  test build, so no `cfg` gate is needed.
- The "every presence value" scenario is a `proptest`. Generate
  `Presence<u32>` with `prop_oneof![any::<u32>().prop_map(Presence::Some),
  Just(Presence::Null), Just(Presence::Absent)]`. The invariant is
  `from_str(to_string(x)) == x`.

### Documentation changes (checked in review, not by scenarios)

- **`src/lib.rs`:** a `# Serde` section in the crate-level docs that states
  the two-attribute rule, explains what goes wrong without each attribute,
  and has a doctest example deserializing and serializing all three states.
  Gate the example with `# #[cfg(feature = "serde")] {` as the existing
  module docs do.
- **`src/presence.rs`:** a short paragraph in the `Presence` type docs that
  links to the crate-root Serde section.
- **`src/serde.rs`:** module docs reduced to a pointer to the crate-root
  section. The broken `User` example is removed, not kept in parallel.
  The existing unit tests stay.
- **`README.md`:** a `## Serde` section with the same rule and a correct
  example (including `#[serde(default)]`).
- **`Cargo.toml`:** `[package.metadata.docs.rs]` with `all-features = true`.
- **`CHANGELOG.md`:** under `## [Unreleased]`, a `### Documentation` entry
  describing the now-visible round-trip guidance.

### Invariants

- With both attributes, JSON round-trip is the identity on `Presence<T>`.
  This is a property test.
- Serialization output is unchanged: `Some(v)` → `v`, while `Null` and
  `Absent` → `null` unless skipped.

### Non-goals

- Making a missing field without `#[serde(default)]` an error, or otherwise
  changing `Deserialize`.
- Changing how `Absent` serializes (for example, erroring when it isn't
  skipped).
- A `#[serde(with = ...)]` helper module or a custom derive.
- Formats other than JSON in tests, and formats that don't describe
  themselves.
- Nested types such as `Presence<Option<T>>` or `Presence<Presence<T>>`,
  whose inner `null` is inherently ambiguous.
- `doc(cfg)` feature badges, which need nightly rustdoc.
