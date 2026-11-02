# Cargo features, MSRV and no_std

Every feature is off by default. The bare crate is `Presence<T>` and its
inherent API with no dependencies.

| Feature | Pulls in | Adds |
| --- | --- | --- |
| `serde` | `serde` 1.0.194, `default-features = false` | `Serialize` and `Deserialize` for `Presence<T>` |
| `schemars` | `schemars` 1, `default-features = false` | `JsonSchema` for `Presence<T>`, needs `alloc` |
| `derive` | `presence-rs-derive`, pinned to the same version | `#[derive(Patch)]` |

`serde` depends on serde without `derive` and without `std`, because the
impls are written by hand. A crate that derives `Serialize` or
`Deserialize`, or serializes a type like `String`, enables serde's
`derive` and `std` or `alloc` itself.

`derive` does not imply `serde`. A patch that derives `Serialize` or
`Deserialize` needs both features, and one that derives `JsonSchema`
needs `schemars` too.

`presence-rs-derive` is released in lockstep with `presence-rs` and
depended on with `=`, so the two always match. Depend on `presence-rs`
with the `derive` feature rather than on the derive crate: its output
names `presence-rs` items.

docs.rs builds with all features and labels the feature-gated items.

## MSRV

Rust 1.85, edition 2024, for both crates. The serde floor of 1.0.194
(January 2024) is the oldest serde that resolves together with the
`schemars` feature's dependencies.

## no_std

The crate is `#![no_std]`. With default features and with `serde` it
needs no allocator: `collect`, `sum` and `product` over presences stream
their values rather than buffering them into a `Vec`, which a test with a
counting global allocator checks. The `schemars` feature is the one that
needs `alloc`, for the `Cow<'static, str>` in a schema name.

Code that `#[derive(Patch)]` generates builds without `std` or an
allocator as well, serde derives included. The repository keeps a fixture
crate that derives, serializes and applies a patch under `#![no_std]` and
builds it for `thumbv7em-none-eabihf`.
