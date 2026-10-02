# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `IntoIterator` for `&Presence<T>` and `&mut Presence<T>`, so `for x in &presence` and `for x in &mut presence` work as they do for `Option`
- `presence::IntoIter`, the owning iterator returned by `Presence::into_iter`, named as in `std::option`
- `Presence::is_nullish_or`, `true` for `Null` and `Absent` or when the value matches the predicate
- `no_std` support: the crate is `#![no_std]` and needs no allocator, with or without the `serde` feature

### Changed
- The published crate contains only the library source, README, LICENSE and CHANGELOG; repository tooling, specs and tests are no longer packaged
- The `serde` feature now depends on `serde` with `default-features = false`, so it no longer turns on `serde`'s `std` feature for you. If your crate derives `Serialize`/`Deserialize` for types like `String` and relied on that, enable `serde`'s `std` (or `alloc`) feature in your own `Cargo.toml`
- `collect`, `sum` and `product` over presences stream their values instead of copying them into a temporary `Vec`, so `sum`, `product` and collecting into a type that does not allocate perform no allocation. The target is always built from the values before the first `Null` or `Absent` (possibly none) and then discarded if the result is `Null` or `Absent`. Values before the first `Null` or `Absent` are now combined as they arrive, as with `Option`: an overflow among them panics in debug builds even though the result is `Null` or `Absent`

### Deprecated
- `presence::Item`, renamed to `presence::IntoIter`
- `Presence::is_null_or`, renamed to `is_nullish_or` because it is also `true` for `Absent`
- `Presence::to_nested_option`, the same as `to_nullable`
- `Presence::reduce`, the same as `zip_with`

### Documentation
- Document the serde round-trip rule where users see it: the crate docs, the `Presence` type docs and the README now explain that a struct field needs both `#[serde(default)]` and `#[serde(skip_serializing_if = "Presence::is_absent")]`; without `default`, a missing field deserializes as `Null`
- Build the docs.rs page with all features, so the `Serialize`/`Deserialize` impls are shown
- Fix the README examples that did not compile; README code blocks now run as doctests
- Document how combinators treat `Null` and `Absent` in a new "Combining states" section: `map`, `and`, `and_then`, `filter`, `flatten`, `unzip`, `transpose`, `copied` and `cloned` keep the receiver's state; `or` and `or_else` return the first `Some`, otherwise the alternative
- Document that `zip`, `zip_with`, `collect`, `sum` and `product` give `Absent` precedence over `Null` and `Null` over `Some`, and fix the `zip` and `zip_with` docs, which claimed `Null` only when both sides are `Null`
- Document the `xor` rule, and why a `filter` that rejects a value, an `xor` of two `Some`s and a `take_if` that takes nothing return `Absent`
- Guarantee the ordering `Absent < Null < Some(_)` (with `Some` values compared by contents), and note that converting `None` into a `Presence<Option<_>>` gives `Some(None)`

## [0.2.0] - 2026-01-02

### Added
- Re-export `Presence` type at crate root for convenience
- Add `#[must_use]` attributes for better compile-time warnings
- Add `transpose()` method for converting `Presence<Result<T, E>>` to `Result<Presence<T>, E>`
- Remove redundant `map_defined()` method

### Documentation
- Update documentation
- Update CONTRIBUTING.md

## [0.1.1] - 2025-12-15

### Documentation
- Add SECURITY.md for security policy documentation
- Add Contributor Covenant Code of Conduct
- Update issue templates
- Fix CI badge link in README.md

## [0.1.0] - 2025-12-14

Initial release of the `Presence<T>` crate.

### Added

#### Core Type
- Initial implementation of `Presence<T>` tri-state type with `Absent`, `Null`, and `Some(T)` variants
- Core query methods: `is_absent()`, `is_null()`, `is_present()`, `is_some()`, `is_none()`
- Predicate-based query methods: `is_some_and()`, `is_none_or()`, `contains()`, `exists()`

#### Reference and Deref Methods
- Reference methods: `as_ref()`, `as_mut()`, `as_pin_ref()`, `as_pin_mut()`
- Deref methods: `as_deref()`, `as_deref_mut()`
- Slice representation: `as_slice()`, `as_mut_slice()`

#### Value Extraction
- Safe extraction with defaults: `unwrap_or()`, `unwrap_or_default()`, `unwrap_or_else()`
- Panic-based extraction: `unwrap()`, `expect()`

#### Transformations
- Map operations: `map()`, `map_or()`, `map_or_else()`
- Flatmap and flatten operations
- Cloning and copying utilities: `cloned()`, `copied()`

#### Combinators
- Basic combinators: `and()`, `and_then()`, `or()`, `or_else()`, `xor()`, `filter()`
- Insert and get operations: `insert()`, `get_or_insert()`, `get_or_insert_with()`, `get_or_insert_default()`
- Replace and take operations: `replace()`, `take()`, `take_if()`
- Zip operations: `zip()`, `zip_with()`, `unzip()`

#### Conversions
- Result conversions: `ok_or()`, `ok_or_else()`, `transpose()`
- From/Into implementations for `Option<T>`, `Option<Option<T>>`, and value types
- Cardinality helpers: `collapse()`, `expand()` for converting between `Presence<T>` and `Option<T>`

#### Iterator Support
- `IntoIterator` trait implementation
- Iterator methods for `Presence::Item`
- `FromIterator` trait implementation for collecting iterators into `Presence`
- `Product` and `Sum` trait implementations

#### Traits
- `Default` trait implementation (defaults to `Presence::Absent`)
- `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash` implementations

#### Macros
- `presence!` macro for ergonomic construction of `Presence` values

#### Serialization
- Optional serde support via the `serde` feature flag
- Serialization/deserialization that preserves the tri-state distinction

#### Testing
- Comprehensive test suite covering:
  - General functionality
  - Query methods
  - Transformations
  - Conversions
  - Iterator operations
  - Macro usage
  - Serde serialization/deserialization

#### Documentation
- Complete API documentation with examples
- Comprehensive README with usage guide and practical examples
- Inline documentation and doc tests

#### Development Infrastructure
- CI/CD pipeline with GitHub Actions
- Rust edition 2024 support
- MIT OR Apache-2.0 dual license
- Cargo.toml configuration with optional features

[0.2.0]: https://github.com/minikin/presence-rs/releases/tag/v0.2.0
[0.1.1]: https://github.com/minikin/presence-rs/releases/tag/v0.1.1
[0.1.0]: https://github.com/minikin/presence-rs/releases/tag/v0.1.0
