# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2025-12-22

### Changed
- Remove redundant `map_defined()` method

### Documentation
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

[0.1.2]: https://github.com/minikin/presence-rs/releases/tag/v0.1.2
[0.1.1]: https://github.com/minikin/presence-rs/releases/tag/v0.1.1
[0.1.0]: https://github.com/minikin/presence-rs/releases/tag/v0.1.0
