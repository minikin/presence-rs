# Spec 08: JSON Schema through `schemars`

**Status:** Approved
**Effort:** Small
**Module:** `src/schemars.rs`, `src/lib.rs`, `Cargo.toml`, `README.md`, `tests/schemars_tests.rs`, `.github/workflows/ci.yml`, `CHANGELOG.md`

## Context

`Presence` exists mostly for PATCH request bodies, and teams that document
such APIs generate JSON Schema or OpenAPI from their Rust types. Today a
`Presence<T>` field cannot appear in a type that derives
`schemars::JsonSchema`, because `Presence<T>` does not implement it. The
team either drops the type or writes the schema by hand. No other tri-state
crate implements `JsonSchema` correctly, so this also sets `presence-rs`
apart.

A `Presence` field that follows the crate's documented serde rule,
`#[serde(default, skip_serializing_if = "Presence::is_absent")]`, means "may
be left out, may be `null`, otherwise a `T`". That is exactly the schema
schemars 1.x generates for an `Option<T>` field with the same attributes.
The schemars derive reads those two serde attributes to decide that a field
is not required, so users annotate nothing new.

Decisions (settled in the spec interview):

1. **schemars 1.x only**, through an optional `schemars` feature. schemars
   0.8 is still used (dropshot, okapi, tauri), but the newest adopters are
   on 1.x. A `schemars_0_8` feature can be added later the way `serde_with`
   does it, with a renamed dependency.
2. **utoipa gets documentation, not an impl.** The only way to implement
   utoipa's schema traits for a generic type is the `#[doc(hidden)]`
   `utoipa::__dev::ComposeSchema`, utoipa 6 needs Rust 1.88 (above this
   crate's 1.85), and the impl would render each field as a `$ref` to a
   component. The README shows `#[schema(value_type = Option<T>)]`
   instead, which gives an inline, nullable, non-required property.
3. **The no-allocator claim becomes conditional.** schemars needs `alloc`.
   With default features the crate stays `no_std` and allocation-free. The
   `schemars` feature pulls in `alloc`, and the README says so.

Facts established by experiment (schemars 1.2.2, utoipa 5.5.0 and 6.0.0):

- The schemars derive marks a field optional when it has both
  `#[serde(default)]` and `skip_serializing_if`, regardless of the field's
  type and of whether the schema is generated for deserialization or
  serialization.
- With `#[serde(default)]` but no `skip_serializing_if`, schemars adds
  `"default": null` to the property, which reads as "omitted means null",
  the opposite of `Absent`. The documented attribute pair avoids it.
- `schemars = { version = "1", default-features = false }` builds at Rust
  1.85 inside a `#![no_std]` crate with `alloc`.

Defaults the spec takes (approval confirms them):

- The impl delegates `inline_schema`, `schema_name`, `schema_id` and
  `json_schema` to `Option<T>`, so the schema of a `Presence<T>` is the
  schema of an `Option<T>`. It does not override schemars'
  `#[doc(hidden)]` `_schemars_private_*` hooks, whose own source marks
  them as not public API. A consequence: a `Presence` field without
  `#[serde(default)]` is listed as required, although serde reads a missing
  field as `Null`. The crate already tells users to add `#[serde(default)]`,
  and this keeps the impl on public API.
- `extern crate alloc` lives in `src/schemars.rs`, not in `src/lib.rs`, so
  every file of the default build stays free of `alloc`, and the
  allocation scan in `tests/allocation.rs` keeps covering all of them.
- The README gains a "JSON Schema and OpenAPI" section with a schemars
  example and the utoipa `value_type` example, both run as doctests. That
  adds `schemars` (with `derive`) and `utoipa` 5 as dev-dependencies.
  utoipa 5 is picked over 6 because 6 needs Rust 1.88.
- CHANGELOG entries go under `[Unreleased]`.

---

## Acceptance Tests

### Scenario: A documented Presence field gets the schema of an Option field

```
Given a struct deriving JsonSchema with a field
      #[serde(default, skip_serializing_if = "Presence::is_absent")] name: Presence<String>
And   the same struct with the field typed Option<String> and the same attributes
When  the JSON Schema of each is generated with schemars' default settings
Then  the two schemas are equal
And   "name" has the schema {"type": ["string", "null"]}
And   "name" is not listed in "required"
```

### Scenario: The field stays optional when the schema describes serialization

```
Given the struct with the documented Presence field
When  its JSON Schema is generated for serialization
Then  "name" is not listed in "required"
```

### Scenario: The OpenAPI 3.0 settings mark the field nullable

```
Given the struct with the documented Presence field
When  its schema is generated with SchemaSettings::openapi3()
Then  "name" has the schema {"type": "string", "nullable": true}
And   "name" is not listed in "required"
```

### Scenario: A Presence of a referenced type adds no definition of its own

```
Given a struct deriving JsonSchema with a documented field inner: Presence<Inner>,
      where Inner is a struct deriving JsonSchema
When  its JSON Schema is generated
Then  "inner" is {"anyOf": [{"$ref": "#/$defs/Inner"}, {"type": "null"}]}
And   "$defs" holds Inner and no definition named after Presence
```

### Scenario: The library builds without std with the schemars feature

```
Given the thumbv7em-none-eabihf target, which has no std
When  the library is built for it with the schemars feature
Then  the build succeeds
```

### Scenario: The default build still uses no allocator

```
Given the library source files compiled without the schemars feature
When  they are scanned for the alloc crate
Then  none of them uses it
```

---

## Tasks

- [x] **T1: Implement `JsonSchema` for `Presence<T>` behind a `schemars` feature.** Scenarios: _A documented Presence field gets the schema of an Option field; The field stays optional when the schema describes serialization; The OpenAPI 3.0 settings mark the field nullable; A Presence of a referenced type adds no definition of its own_. Tests: acceptance in a new `tests/schemars_tests.rs`, comparing whole schemas as `serde_json::Value`. Adds the optional dependency and feature, the self dev-dependency feature, `src/schemars.rs`, the `mod` line in `src/lib.rs`, and a CHANGELOG `Added` entry under `[Unreleased]`.
- [x] **T2: Keep the no_std and no-alloc guarantees.** Needs: T1. Scenarios: _The library builds without std with the schemars feature; The default build still uses no allocator_. Tests: the `no-std` CI job gains a `--features schemars` build, run locally first; the allocation scan in `tests/allocation.rs` stays on the default-build files.
- [x] **T3: Document JSON Schema and OpenAPI use.** Needs: T1. Scenarios: _none (documentation; the README runs as doctests)_. Tests: README doctests for the schemars derive and for utoipa's `#[schema(value_type = Option<T>)]`, which add `schemars` (with `derive`) and `utoipa` 5 as dev-dependencies. Adds the README section, the conditional "with default features" in the Overview, and the CHANGELOG `Documentation` entry.

---

## Implementation Notes

- `Cargo.toml`: `schemars = { version = "1", default-features = false,
  optional = true }` and `schemars = ["dep:schemars"]` under `[features]`.
  The crate's own dev-dependency on itself enables `schemars` beside
  `serde`, so the gates build and measure `src/schemars.rs`.
- `src/lib.rs`: `#[cfg(feature = "schemars")] mod schemars;`, next to the
  `serde` module.
- `src/schemars.rs`:

  ```rust
  extern crate alloc;

  impl<T: JsonSchema> JsonSchema for Presence<T> {
      fn inline_schema() -> bool { <Option<T>>::inline_schema() }
      fn schema_name() -> Cow<'static, str> { <Option<T>>::schema_name() }
      fn schema_id() -> Cow<'static, str> { <Option<T>>::schema_id() }
      fn json_schema(g: &mut SchemaGenerator) -> Schema { <Option<T>>::json_schema(g) }
  }
  ```

- Tests go in a new `tests/schemars_tests.rs`, one per scenario, comparing
  `serde_json::Value`s. The equality with the `Option` struct is the main
  invariant, so the first scenario's test compares whole schemas.
- `.github/workflows/ci.yml`: the `no-std` job builds with `--features
  schemars` as well. The MSRV job already runs `--all-features`, so it
  covers schemars at 1.85.
- `tests/allocation.rs`: the scan keeps its list of default-build files
  and does not list `src/schemars.rs`.
- README: a "JSON Schema and OpenAPI" section after "Serde". It explains
  that the two serde attributes also drive the schema, shows the schemars
  derive, and shows utoipa's `#[schema(value_type = Option<T>)]`. The
  Overview's "needs no allocator" sentence gains "with default features".

### Invariants worth a property test

- None beyond the schema equality in the first scenario: the schema is a
  function of the type, not of a value, so there is no input space to
  sample.

### Non-goals

- schemars 0.8 or 0.9 support.
- A utoipa impl (`ToSchema` or `ComposeSchema`).
- Overriding schemars' `_schemars_private_*` hooks, or making a field
  without `#[serde(default)]` optional in the schema.
- Schemas for anything other than `Presence<T>`.
