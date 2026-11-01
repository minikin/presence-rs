# Spec 09: Deriving a patch struct

**Status:** Implemented
**Effort:** Large
**Module:** `derive/` (new `presence-rs-derive` crate), `src/patch.rs`, `src/lib.rs`, `Cargo.toml`, `.github/workflows/`, `README.md`, `CHANGELOG.md`

## Context

The main use of `Presence` is a PATCH request body. Today a user with a
model such as

```rust
struct User {
    id: u64,
    name: String,
    nickname: Option<String>,
}
```

writes a second struct by hand, `UserPatch`, with one `Presence` field per
model field. They put `#[serde(default, skip_serializing_if =
"Presence::is_absent")]` on every field, then write an `apply_to` call for
each field. Each new field on the model means three edits in two places,
and a missed edit compiles fine and silently drops updates.

The desired outcome is `#[derive(Patch)]` on the model, which generates the
patch struct, its serde attributes, and the code that applies it, and keeps
all three in sync with the model.

Decisions settled in the spec interview:

1. **Field mapping.** A model field of type `Option<T>` becomes
   `Presence<T>` in the patch, with `Presence::apply_to` semantics: left
   out leaves it alone, `null` clears it, a value sets it. Any other field
   type `T` becomes `Option<T>`: left out leaves it alone, a value
   replaces it. A `null` for such a field is a deserialization error,
   because the model has no way to hold it.
2. **Applying.** `patch.apply_to(&mut model)` returns `()` and cannot
   fail. An invalid `null` is stopped at deserialization and never reaches
   it.
3. **Trait.** A `Patch` trait in `presence-rs` declares `apply_to`,
   `merge` and `is_empty`, and the derive implements it, so generic code
   can accept any derived patch.
4. **Naming and visibility.** The generated struct is `{Model}Patch` with
   the model's visibility and each field's visibility.
   `#[patch(name = Foo)]` overrides the name.
5. **Derives.** The patch always derives `Default`, which is the empty
   patch. `#[patch(derive(...))]` adds more. The serde field attributes are
   emitted only when `Serialize` or `Deserialize` is in that list.
6. **Skipping.** `#[patch(skip)]` leaves a field out of the patch.
7. **Accepted input.** Structs with named fields, including generics,
   lifetimes and `where` clauses, which carry over to the patch. Every
   other item is a compile error.
8. **Methods.** `merge(self, later)` composes two patches field by field
   with the rule of `Presence::merge`: the later field wins unless it is
   unset. `is_empty(&self)` is true when every field is unset.
9. **Layout.** A `derive/` workspace member, `presence-rs-derive`,
   released in lockstep with `presence-rs`. It is re-exported as
   `presence_rs::Patch` behind a new `derive` feature, off by default.
   MSRV 1.85. The generated code builds in `no_std` without an allocator.
10. **Serde attributes from the model.** The container attributes
    `rename_all` and `deny_unknown_fields` and the field attributes
    `rename` and `alias` are copied, so the patch uses the model's wire
    names. A field with `#[serde(skip)]` or `#[serde(skip_deserializing)]`
    is left out of the patch. `default`, `skip_serializing` and
    `skip_serializing_if` are dropped, since the patch sets its own.
    `with`, `serialize_with`, `deserialize_with` and `flatten` are a
    compile error on that field whose message suggests `#[patch(skip)]`.
    *(Clarified after review, with the user's approval: all of this applies
    to a patch that derives serde. A patch without serde derives never
    touches the wire, so it ignores the model's serde attributes: a codec
    is no error and a `#[serde(skip)]` field stays in the patch.)*
11. **Type aliases.** Only `Option<T>`, `std::option::Option<T>` and
    `core::option::Option<T>` count as nullable. A type alias of `Option`
    is treated as any other type, and this limitation is documented.
12. **Crate path.** `#[patch(crate = path)]` overrides the default
    `::presence_rs`, for users who rename the dependency.
13. **JSON Schema.** With `JsonSchema` in the derive list, a field from a
    non-`Option` model field has the schema of `T`, not nullable, matching
    the rejected `null`. A field from an `Option<T>` model field has the
    schema of `T` or `null`. Neither is listed in `required`.
14. **Release.** `release.yml` publishes `presence-rs-derive` before
    `presence-rs`. Configuring trusted publishing for the new crate on
    crates.io is a manual step for the maintainer.

Defaults the spec takes (approval confirms them):

- `merge` and `is_empty` are trait methods, not inherent ones, so generic
  code has the whole interface.
- A merged field from a non-`Option` model field is `later.or(earlier)`.
- `#[patch(...)]` keys other than `name`, `derive`, `crate` (container)
  and `skip` (field) are a compile error.
- The `derive` feature does not enable `serde`. A patch that derives
  `Serialize` or `Deserialize` needs the `serde` feature too, as the docs
  say.
- A README section "Deriving a patch" and a CHANGELOG `Added` entry under
  `[Unreleased]`. No version bump.

---

## Acceptance Tests

Each scenario uses the `User` model from the Context unless it says
otherwise. "Nullable field" means a patch field derived from an
`Option<T>` model field, and "required field" means one derived from any
other type.

### Scenario: An unset nullable field leaves the model's field unchanged

```
Given any User (property)
And   a UserPatch whose nickname is Presence::Absent
When  the patch is applied to the user
Then  the user's nickname is unchanged
```

### Scenario: A null nullable field clears the model's field

```
Given any User (property)
And   a UserPatch whose nickname is Presence::Null
When  the patch is applied to the user
Then  the user's nickname is None
```

### Scenario: A set nullable field sets the model's field

```
Given any User and any string s (property)
And   a UserPatch whose nickname is Presence::Some(s)
When  the patch is applied to the user
Then  the user's nickname is Some(s)
```

### Scenario: A required field is replaced only when set

```
Given any User and any Option<String> name (property)
And   a UserPatch whose name is that Option
When  the patch is applied to the user
Then  the user's name is the set value if name is Some
And   otherwise the user's name is unchanged
```

### Scenario: The default patch is empty and changes nothing

```
Given any User (property)
When  UserPatch::default() is applied to it
Then  the user is unchanged
And   UserPatch::default().is_empty() is true
```

### Scenario: is_empty is true exactly when every field is unset

```
Given any UserPatch (property)
When  is_empty is called
Then  it is true if name is None and nickname is Absent
And   it is false otherwise
```

### Scenario: Applying a merged patch equals applying the patches in order

```
Given any User and any UserPatches a and b (property)
When  a.merge(b) is applied to one copy of the user
And   a and then b are applied to another copy
Then  both copies are equal
```

### Scenario: Merging patches is associative and the empty patch changes nothing

```
Given any UserPatches a, b and c (property)
When  they are composed with merge
Then  a.merge(b).merge(c) equals a.merge(b.merge(c))
And   UserPatch::default().merge(a) and a.merge(UserPatch::default()) both equal a
```

### Scenario: A skipped field is not part of the patch

```
Given a model whose id field has #[patch(skip)]
When  code builds the patch with an id field
Then  it fails to compile
And   applying any patch to any model leaves id unchanged (property)
```

### Scenario: The patch is named after the model and keeps its visibility

```
Given a pub(crate) model Account with a pub field and a private field
When  Patch is derived
Then  a pub(crate) struct AccountPatch exists
And   its fields have the visibility of the matching model fields
```

### Scenario: The patch name can be overridden

```
Given a model with #[patch(name = AccountUpdate)]
When  Patch is derived
Then  the patch struct is named AccountUpdate
```

### Scenario: Generics, lifetimes and where clauses carry over

```
Given a model Labelled<'a, T> where T: Clone, with fields label: &'a str and value: Option<T>
When  Patch is derived
Then  LabelledPatch<'a, T> exists with the same bounds
And   applying it follows the field mapping rules
```

### Scenario: Requested derives are added to the patch

```
Given a model with #[patch(derive(Debug, Clone, PartialEq))]
When  Patch is derived
Then  the patch implements Debug, Clone, PartialEq and Default
```

### Scenario: A patch without serde derives carries no serde attributes

```
Given a model without Serialize or Deserialize in #[patch(derive(...))]
And   a crate that does not depend on serde
When  Patch is derived
Then  the crate compiles
```

### Scenario: Unset fields are left out of the serialized patch

```
Given a UserPatch that derives Serialize, with name None and nickname Absent
When  it is serialized to JSON
Then  the JSON is {}
```

### Scenario: Missing keys deserialize to unset fields

```
Given a UserPatch that derives Deserialize
When  the JSON {} is deserialized
Then  the patch is empty
```

### Scenario: A null nullable field round-trips as null

```
Given a UserPatch that derives Serialize and Deserialize
When  the JSON {"nickname": null} is deserialized
Then  nickname is Presence::Null
And   serializing the patch gives {"nickname": null}
```

### Scenario: A null required field is a deserialization error

```
Given a UserPatch that derives Deserialize
When  the JSON {"name": null} is deserialized
Then  deserialization fails
```

### Scenario: A serialized patch deserializes to the same patch

```
Given any UserPatch that derives Serialize and Deserialize (property)
When  it is serialized to JSON and deserialized back
Then  the result equals the original
```

### Scenario: The model's wire names are kept

```
Given a model with #[serde(rename_all = "camelCase")], a field display_name and a field email with #[serde(rename = "mail", alias = "e_mail")]
When  Patch is derived with Serialize and Deserialize
Then  the patch reads and writes displayName and mail
And   it also accepts e_mail when deserializing
```

### Scenario: Unknown keys are rejected when the model rejects them

```
Given a model with #[serde(deny_unknown_fields)]
When  JSON with a key the model does not have is deserialized into its patch
Then  deserialization fails
```

### Scenario: Fields skipped by serde are left out of the patch

```
Given a model with a field marked #[serde(skip)] and another marked #[serde(skip_deserializing)]
When  Patch is derived
Then  the patch has neither field
```

### Scenario: A field with a custom serde codec is a compile error

```
Given a model with a field marked #[serde(with = "...")], #[serde(serialize_with = "...")], #[serde(deserialize_with = "...")] or #[serde(flatten)]
When  Patch is derived
Then  compilation fails with an error on that field
And   the error message suggests #[patch(skip)]
```

### Scenario: A type alias of Option is treated as a required field

```
Given a model field of type MaybeName, where type MaybeName = Option<String>
When  Patch is derived with Deserialize
Then  the patch field has type Option<MaybeName>
And   {"field": null} fails to deserialize
```

### Scenario: The schema of a patch matches what it accepts

```
Given a UserPatch that derives JsonSchema, Serialize and Deserialize
When  its JSON Schema is generated
Then  name has the schema of a string, not nullable
And   nickname has the schema of a string or null
And   neither is listed in required
```

### Scenario: Only structs with named fields can derive Patch

```
Given a tuple struct, a unit struct, an enum or a union
When  Patch is derived on it
Then  compilation fails with an error pointing at the item
```

### Scenario: An unknown patch attribute is a compile error

```
Given a model with #[patch(rename = X)] or a field with #[patch(nullable)]
When  Patch is derived
Then  compilation fails with an error pointing at the unknown key
```

### Scenario: The crate path can be overridden

```
Given a crate that depends on presence-rs under the name presence
And   a model with #[patch(crate = presence)]
When  Patch is derived
Then  the crate compiles and the patch applies as usual
```

### Scenario: Generic code accepts any derived patch

```
Given a function generic over P: Patch that applies P to a P::Target
When  it is called with a UserPatch and a User
Then  the user is patched as if UserPatch::apply_to had been called
```

### Scenario: The derive builds without std or an allocator

```
Given a no_std crate without alloc that enables the derive and serde features
When  it derives Patch with Serialize and Deserialize and is built for thumbv7em-none-eabihf
Then  the build succeeds
```

---

## Tasks

Each task lists its scenarios, the test types that pin it (unit /
property / acceptance), and, when it depends on earlier tasks, a
`Needs:` naming them. Tasks with no `Needs:` are roots; tasks whose needs
are all done are ready; the graph is what `scripts/keeler-graph.sh` reads.

- [x] **T1 - Workspace, the `Patch` trait and a derive that applies nullable and required fields.** Scenarios: _An unset nullable field leaves the model's field unchanged; A null nullable field clears the model's field; A set nullable field sets the model's field; A required field is replaced only when set; The patch is named after the model and keeps its visibility; Generic code accepts any derived patch_. Tests: acceptance + property (each `apply_to` case for any `User`) + unit tests on `expand`. Adds the `[workspace]`, `derive/` with its thin entry point and `expand`, `src/patch.rs` with `apply_to` only, and the `derive` feature. CI test and lint jobs build the workspace.
- [x] **T2 - `merge`, `is_empty` and the empty `Default` patch.** Needs: T1. Scenarios: _The default patch is empty and changes nothing; is_empty is true exactly when every field is unset; Applying a merged patch equals applying the patches in order; Merging patches is associative and the empty patch changes nothing_. Tests: acceptance + property (monoid laws, agreement of `merge` with sequential `apply_to`, `is_empty` iff every field unset) + unit tests on the generated methods. Adds the two methods to the trait and to the generated impl.
- [x] **T3 - Container and field options: `name`, `derive(...)`, `skip`, generics.** Needs: T2. Scenarios: _A skipped field is not part of the patch; The patch name can be overridden; Generics, lifetimes and where clauses carry over; Requested derives are added to the patch_. Tests: acceptance + property (a skipped field is unchanged for any model and patch) + trybuild compile-fail for building a patch with a skipped field + unit tests per attribute parser. Adds `#[patch(...)]` parsing, one function per key.
- [x] **T4 - Compile errors for unsupported input and unknown keys.** Needs: T3. Scenarios: _Only structs with named fields can derive Patch; An unknown patch attribute is a compile error_. Tests: trybuild compile-fail with checked-in `.stderr` + unit tests on `expand` returning errors. Errors are combined so one build reports all of them.
- [x] **T5 - Serde support for derived patches.** Needs: T3. Scenarios: _A patch without serde derives carries no serde attributes; Unset fields are left out of the serialized patch; Missing keys deserialize to unset fields; A null nullable field round-trips as null; A null required field is a deserialization error; A serialized patch deserializes to the same patch_. Tests: acceptance + property (JSON round trip) + trybuild pass for the serde-free crate + unit tests on `__private::some`. Adds the field attributes, derive-list detection, and the hidden `__private` module behind `serde`.
- [x] **T6 - Carry the model's serde attributes over to the patch.** Needs: T5. Scenarios: _The model's wire names are kept; Unknown keys are rejected when the model rejects them; Fields skipped by serde are left out of the patch; A field with a custom serde codec is a compile error_. Tests: acceptance + trybuild compile-fail for each of `with`, `serialize_with`, `deserialize_with`, `flatten` + unit tests on the serde attribute filter.
- [x] **T7 - Treat a type alias of `Option` as a required field.** Needs: T5. Scenarios: _A type alias of Option is treated as a required field_. Tests: acceptance + unit tests on `Option` path detection (`Option`, `std::option::Option`, `core::option::Option`, an alias).
- [x] **T8 - JSON Schema for derived patches.** Needs: T6. Scenarios: _The schema of a patch matches what it accepts_. Tests: acceptance (schema of `name`, `nickname`, and an empty `required`) + unit test on the emitted `#[schemars(with = ...)]`.
- [x] **T9 - Override the crate path.** Needs: T4. Scenarios: _The crate path can be overridden_. Tests: trybuild pass fixture that depends on `presence-rs` renamed to `presence` + unit test that generated paths use the override.
- [x] **T10 - `no_std` build, CI and the two-crate release.** Needs: T5. Scenarios: _The derive builds without std or an allocator_. Tests: the `derive/tests/no_std/` fixture crate built for `thumbv7em-none-eabihf` in `ci.yml` and `release.yml`. Adds the MSRV check for both crates and makes `release.yml` publish `presence-rs-derive` before `presence-rs`. Each workflow change goes in its own named step next to the existing `no_std` and publish steps.
- [x] **T11 - Document the derive in the README, the crate docs and the CHANGELOG.** Needs: T7, T8, T9, T10. Scenarios: _none (documentation; the README section runs as a doctest through `ReadmeDoctests`)_. Tests: README and crate-level doctests. Adds "Deriving a patch" to the README, the `Option` alias limitation, the `with` limitation, and a CHANGELOG `Added` entry under `[Unreleased]`.

---

## Implementation Notes

- **Workspace.** The root `Cargo.toml` gains `[workspace] members =
  [".", "derive"]`. `derive/Cargo.toml` is a `proc-macro = true` crate
  depending on `syn` 2, `quote` and `proc-macro2`. Both crates share
  `version` and `rust-version` through `[workspace.package]`. The root
  crate takes `presence-rs-derive = { path = "derive", version = "=X.Y.Z",
  optional = true }` and `derive = ["dep:presence-rs-derive"]`.
- **Trait.** `src/patch.rs`:

  ```rust
  pub trait Patch: Default {
      type Target;
      fn apply_to(self, target: &mut Self::Target);
      fn merge(self, later: Self) -> Self;
      fn is_empty(&self) -> bool;
  }
  ```

  The trait is always available. The derive macro shares its name, as
  serde's do, and is re-exported next to it under `derive`.
- **Generated code.** For each field, `apply_to` calls
  `Presence::apply_to` (nullable) or `if let Some(v) = self.f { target.f =
  v }` (required). `merge` uses `Presence::merge` or `later.f.or(self.f)`.
  `is_empty` is a conjunction of `is_absent()` and `is_none()`. All paths
  go through the crate path (`::presence_rs` by default) and `::core`, so
  the output works in `no_std`.
- **Required fields and `null`.** A required field gets
  `#[serde(default, skip_serializing_if = "Option::is_none",
  deserialize_with = "<crate>::__private::some")]`. `some` deserializes a
  `T` and wraps it in `Some`, so `null` reaches `T`'s own deserializer and
  fails there. It lives in a `#[doc(hidden)] pub mod __private` behind the
  `serde` feature and is not public API.
- **Schema.** A required field also gets `#[schemars(with = "T")]` when
  `JsonSchema` is in the derive list, so its schema is `T`, not `T` or
  `null`.
- **Detecting derives.** The `#[patch(derive(...))]` list is matched on
  the last path segment (`Serialize`, `Deserialize`, `JsonSchema`), so
  `serde::Serialize` and `Serialize` both count.
- **Errors.** Every compile error uses `syn::Error::new_spanned` on the
  offending item, field or attribute key, and errors are combined so one
  build reports all of them.
- **Coverage and CRAP.** Code that runs inside rustc as a proc macro is
  invisible to `cargo llvm-cov`, so a thin `#[proc_macro_derive]` entry
  point hands a `proc_macro2::TokenStream` to an `expand` function that
  unit tests call directly. Without that split the derive crate reports
  near-zero coverage and fails both gates. CI holds every function to a
  CRAP score of 10 (`KEELER_CRAP_MAX`), so attribute parsing is split into
  one small function per key rather than one large `match`.
- **Tests.**
  - Acceptance tests in `derive/tests/acceptance.rs`, one per scenario,
    with proptest strategies for `User` and `UserPatch`.
  - Compile-fail scenarios through `trybuild`, with fixtures in
    `derive/tests/ui/` and checked-in `.stderr` files.
  - The `no_std` scenario is a fixture crate under `derive/tests/no_std/`
    that CI builds for `thumbv7em-none-eabihf`, next to the existing
    `no_std` builds.
  - The renamed-crate scenario is a trybuild pass fixture with its own
    dependency rename.
- **CI and release.**
  - `ci.yml` and `release.yml` build and test the workspace.
  - The MSRV check covers both crates.
  - `release.yml` publishes `presence-rs-derive`, waits for it to appear
    in the index, then publishes `presence-rs`.

### Invariants worth a property test

- The three nullable cases and the two required cases of `apply_to`, for
  any model.
- `merge` is associative with `Default` as its identity on both sides.
- Applying `a.merge(b)` equals applying `a` then `b`, which ties `merge`
  to `apply_to`.
- `is_empty` agrees with "applying changes nothing for every model".
- Serialize then deserialize is the identity on patches.

### Non-goals

- Nested patches (`#[patch(nested)]`). A field whose type has its own
  patch is replaced whole.
- Enums, tuple structs and unit structs.
- Diffing two models into a patch.
- JSON Patch (RFC 6902) and recursive JSON Merge Patch.
- Validation hooks.
- Returning an undo value from `apply_to`.
- `From<Model>` for a full patch.
- utoipa support.
- Support for serde codecs on patched fields (`with` and friends).
- An override for type aliases of `Option`.
- A version bump or a release.
