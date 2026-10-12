# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]


## [0.3.0] - 2026-10-05

PATCH helpers, `no_std` without an allocator, and JSON Schema through a new
`schemars` feature. Four methods and one type are renamed, and the old names
stay as deprecated aliases. A few changes can break a build: the `serde`
feature no longer enables serde's `std` and `derive`, the serde floor rises
to 1.0.194, and `Presence::from(&p)` needs its target type named (see
Changed).

### Added

- **PATCH helpers.** `Presence::apply_to` applies a presence to an `Option`
  field as a PATCH would: `Absent` leaves it, `Null` clears it, `Some` sets
  it, and the value it replaced comes back. `Presence::merge` composes two
  patches. The later one wins unless it is `Absent`, so a series of PATCH
  requests folds into one.
- **`unwrap_or_absent_or_null(absent, null)`** returns the value of `Some`,
  `absent` for `Absent` and `null` for `Null`. The name gives the order of
  the defaults. `unwrap_or_else_absent_or_null` is its lazy form and calls
  only the closure for the state it meets.
- **JSON Schema.** With the `schemars` feature, `Presence<T>` implements
  `schemars::JsonSchema` (schemars 1.x) with the schema of an `Option<T>`:
  a `T` or `null`. A field with the documented
  `#[serde(default, skip_serializing_if = "Presence::is_absent")]` is not
  listed in `required`. The feature needs `alloc`.
- **`no_std`.** The crate is `#![no_std]` and needs no allocator, with or
  without the `serde` feature.
- **Borrowing.** `IntoIterator` for `&Presence<T>` and `&mut Presence<T>`,
  so `for x in &presence` works as it does for `Option`. `From<&Presence<T>>
  for Presence<&T>` and `From<&mut Presence<T>> for Presence<&mut T>`, as
  `Option` has.
- `presence::IntoIter`, the owning iterator `Presence::into_iter` returns,
  named as in `std::option`.
- `Presence::is_nullish_or`: `true` for `Null` and `Absent`, otherwise the
  predicate's answer.

### Changed

- **BREAKING:** the `serde` feature depends on serde with
  `default-features = false` and without `derive`, since the impls are
  written by hand. It no longer turns on serde's `std` or `derive` for you.
  A crate that derived `Serialize` or `Deserialize`, or serialized types
  like `String`, through that must enable serde's `derive` and `std` (or
  `alloc`) itself.
- **BREAKING:** the `serde` feature requires serde 1.0.194 or later
  (January 2024), the oldest serde that resolves together with the
  `schemars` feature's dependencies.
- **BREAKING:** with the new `From` impls for references,
  `Presence::from(&p)` and `(&p).into()` no longer infer their target type.
  Code that got a `Presence<&Presence<T>>` that way must name the type, as
  with `Option`.
- `collect`, `sum` and `product` over presences stream their values instead
  of copying them into a temporary `Vec`, so `sum`, `product` and collecting
  into a type that does not allocate perform no allocation. The values
  before the first `Null` or `Absent` are combined as they arrive, as with
  `Option`. An overflow among them panics in debug builds even when the
  result is `Null` or `Absent`.
- The published crate holds only the library source, the README, LICENSE
  and this file. Repository tooling, specs and tests are no longer
  packaged.

### Deprecated

- `Presence::unwrap_or_null_default`, renamed to `unwrap_or_absent_or_null`.
  Its name mentions only `null` while its first argument is the `Absent`
  default, so the two were easy to swap. The arguments keep their order.
- `Presence::is_null_or`, renamed to `is_nullish_or`, because it is also
  `true` for `Absent`.
- `Presence::to_nested_option`, the same as `to_nullable`.
- `Presence::reduce`, the same as `zip_with`.
- `presence::Item`, renamed to `presence::IntoIter`.

### Documentation

- The serde round-trip rule is in the crate docs, the `Presence` docs and
  the README: a struct field needs both `#[serde(default)]` and
  `#[serde(skip_serializing_if = "Presence::is_absent")]`. Without
  `default`, a missing field deserializes as `Null`.
- A "Combining states" section says which state each combinator returns.
  `map`, `and`, `and_then`, `filter`, `flatten`, `unzip`, `transpose`,
  `copied` and `cloned` keep the receiver's state. `or` and `or_else`
  return the first `Some`, otherwise the alternative. `zip`, `zip_with`,
  `collect`, `sum` and `product` give `Absent` precedence over `Null` and
  `Null` over `Some`. The `zip` and `zip_with` docs claimed `Null` only
  when both sides are `Null`, which was wrong.
- The ordering `Absent < Null < Some(_)` is a documented guarantee, with
  `Some` values compared by contents.
- The README examples compile and run as doctests. A new "JSON Schema and
  OpenAPI" section covers the `schemars` feature and shows how to describe
  a `Presence<T>` field to utoipa with `#[schema(value_type = Option<T>)]`.
  The README also compares `Presence` with `Option<Option<T>>` and
  `serde_with::rust::double_option`, and states the minimum supported Rust
  version, 1.85.
- docs.rs builds with all features and labels feature-gated items such as
  the serde impls.
- The `presence!` docs say that `presence!(null)` is always `Null`, even
  with a variable named `null` in scope, and that `presence!((null))`
  passes the variable.


## [0.2.0] - 2026-01-02

### Added

- `Presence` is re-exported at the crate root.
- `#[must_use]` on methods whose result is the point of the call.
- `transpose()` turns a `Presence<Result<T, E>>` into a
  `Result<Presence<T>, E>`.

### Removed

- **BREAKING:** `map_defined()`, which did what `map()` does.


## [0.1.1] - 2025-12-15

### Documentation

- SECURITY.md, a Contributor Covenant Code of Conduct and updated issue
  templates.
- The README's CI badge links to the right workflow.


## [0.1.0] - 2025-12-14

The first release: `Presence<T>`, a tri-state type with `Absent`, `Null`
and `Some(T)`, and an API modelled on `Option<T>`.

### Added

- Queries: `is_absent`, `is_null`, `is_present`, `is_defined`, `is_nullish`,
  `is_some_and`, `is_absent_or` and `is_null_or`.
- References and slices: `as_ref`, `as_mut`, `as_pin_ref`, `as_pin_mut`,
  `as_deref`, `as_deref_mut`, `as_slice` and `as_mut_slice`.
- Extraction: `unwrap`, `expect`, `unwrap_or`, `unwrap_or_else`,
  `unwrap_or_default`, and `unwrap_or_null_default` with a separate default
  for each empty state.
- Transformations: `map`, `map_or`, `map_or_else`, `map_or_default`,
  `map_defined`, `inspect`, `flatten`, `copied` and `cloned`.
- Combinators: `and`, `and_then`, `or`, `or_else`, `xor`, `filter`, `zip`,
  `zip_with`, `unzip` and `reduce`.
- In-place updates: `insert`, `get_or_insert`, `get_or_insert_with`,
  `get_or_insert_default`, `replace`, `take` and `take_if`.
- Conversions: `ok_or`, `ok_or_else`, `from_optional`, `to_optional`,
  `from_nullable`, `to_nullable`, `to_nested_option`, `From<T>`, and `From`
  in both directions between `Presence<T>` and `Option<Option<T>>`.
- Iteration: `iter`, `iter_mut`, `len`, `is_empty`, `IntoIterator`,
  `FromIterator`, `Sum` and `Product`.
- `Default` (`Absent`), `Display`, and the derived `Clone`, `Copy`, `Debug`,
  `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash`.
- The `presence!` macro.
- `Serialize` and `Deserialize` behind the `serde` feature.

[Unreleased]: https://github.com/minikin/presence-rs/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/minikin/presence-rs/releases/tag/v0.3.0
[0.2.0]: https://github.com/minikin/presence-rs/releases/tag/v0.2.0
[0.1.1]: https://github.com/minikin/presence-rs/releases/tag/v0.1.1
[0.1.0]: https://github.com/minikin/presence-rs/releases/tag/v0.1.0
