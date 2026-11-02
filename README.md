# Presence

<p align="center">
  <a href="https://crates.io/crates/presence-rs"><img src="https://github.com/minikin/presence-rs/raw/badges/crates.svg" alt="crates.io version"></a>
  <a href="https://github.com/minikin/presence-rs/actions/workflows/ci.yml"><img src="https://github.com/minikin/presence-rs/raw/badges/ci.svg" alt="CI"></a>
  <a href="https://github.com/minikin/presence-rs/actions/workflows/ci.yml"><img src="https://github.com/minikin/presence-rs/raw/badges/coverage.svg" alt="coverage"></a>
  <a href="CONTRIBUTING.md"><img src="https://github.com/minikin/presence-rs/raw/badges/crap.svg" alt="CRAP score"></a>
  <a href="LICENSE"><img src="https://github.com/minikin/presence-rs/raw/badges/license.svg" alt="license"></a>
</p>

`Presence<T>` is `Option<T>` with one more state. A JSON field can be left
out, sent as `null` or sent with a value, and a PATCH request means something
different by each: leave the stored value alone, clear it, or set it.
`Option<T>` has two states for those three, so the difference is lost the
moment the request is parsed. `Presence<T>` keeps it:

- `Absent`: the field is not there, `{}`
- `Null`: the field is there and `null`, `{"field": null}`
- `Some(value)`: the field has a value, `{"field": 42}`

The API follows `Option`'s, so `map`, `and_then`, `unwrap_or` and the rest
work as you expect. On top of it, `apply_to` and `merge` apply and combine
PATCH requests, and `#[derive(Patch)]` writes the request struct for you.
Serde and JSON Schema support sit behind features. The crate is `#![no_std]`
and, with default features, needs no allocator.

The blog post [Stop Losing Intent: Absent, Null, and Value in
Rust](https://minikin.me/blog/presence-rs) explains the motivation.

## Install

```toml
[dependencies]
presence-rs = "0.3.0"
```

The minimum supported Rust version is 1.85. Every feature is off by default:
`serde`, `schemars` and `derive`.

## Quick start

`apply_to` applies one field of a request to the model and returns the value
it replaced. `Absent` leaves the field alone, `Null` clears it, `Some` sets
it.

```rust
use presence_rs::Presence;

struct User {
    nickname: Option<String>,
}

let mut user = User {
    nickname: Some("al".to_string()),
};

Presence::<String>::Absent.apply_to(&mut user.nickname);
assert_eq!(user.nickname.as_deref(), Some("al"));

Presence::Some("neo".to_string()).apply_to(&mut user.nickname);
assert_eq!(user.nickname.as_deref(), Some("neo"));

Presence::Null.apply_to(&mut user.nickname);
assert_eq!(user.nickname, None);
```

`merge` folds several requests into one: the later field wins unless it is
`Absent`.

## Serde

Enable the `serde` feature. A struct field keeps all three states apart only
when it has **both** attributes:

- `#[serde(default)]` makes a missing field `Absent`. Without it, a missing
  `Presence` field is read exactly like `null` (the same rule serde applies to
  `Option`), so it silently becomes `Null`.
- `#[serde(skip_serializing_if = "Presence::is_absent")]` leaves `Absent` out
  of the output. Without it, `Absent` is written as `null` and reads back as
  `Null`.

```rust
use presence_rs::Presence;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Patch {
    #[serde(default, skip_serializing_if = "Presence::is_absent")]
    nickname: Presence<String>,
}

let set: Patch = serde_json::from_str(r#"{"nickname":"neo"}"#).unwrap();
let cleared: Patch = serde_json::from_str(r#"{"nickname":null}"#).unwrap();
let untouched: Patch = serde_json::from_str("{}").unwrap();
assert_eq!(set.nickname, Presence::Some("neo".to_string()));
assert_eq!(cleared.nickname, Presence::Null);
assert_eq!(untouched.nickname, Presence::Absent);

assert_eq!(serde_json::to_string(&untouched).unwrap(), "{}");
```

With the `schemars` feature, `Presence<T>` implements `schemars::JsonSchema`
(schemars 1.x) with the schema of an `Option<T>`: a `T` or `null`. utoipa has
no `Presence` support, so a field is described to it as an `Option<T>` with
`#[schema(value_type = Option<T>)]`. [JSON Schema and
OpenAPI](docs/guides/json-schema.md) has both.

## Deriving a patch

The `derive` feature writes the request struct for you. `#[derive(Patch)]` on
`User` generates `UserPatch`, where a model field of type `Option<T>` becomes
a `Presence<T>` and any other field `T` becomes an `Option<T>`, which can set
the field but not clear it. `UserPatch` implements the `Patch` trait:
`apply_to`, `merge`, `is_empty`, and a `Default` that sets nothing.

```rust
use presence_rs::Patch;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Patch)]
#[patch(derive(Debug, Serialize, Deserialize))]
struct User {
    #[patch(skip)]
    id: u64,
    name: String,
    nickname: Option<String>,
}

let mut user = User {
    id: 7,
    name: "Alice".to_string(),
    nickname: Some("al".to_string()),
};

let patch: UserPatch = serde_json::from_str(r#"{"name":"Alicia","nickname":null}"#).unwrap();
patch.apply_to(&mut user);
assert_eq!(
    user,
    User {
        id: 7,
        name: "Alicia".to_string(),
        nickname: None,
    }
);

// `name` is not an `Option` on the model, so `null` cannot clear it.
assert!(serde_json::from_str::<UserPatch>(r#"{"name":null}"#).is_err());
assert_eq!(serde_json::to_string(&UserPatch::default()).unwrap(), "{}");
```

With `Serialize` or `Deserialize` in `#[patch(derive(...))]`, every field gets
the two serde attributes above, and the model's `rename_all`,
`deny_unknown_fields`, `rename` and `alias` carry over. `#[patch(name)]`,
`#[patch(skip)]` and `#[patch(crate)]` cover naming, leaving fields out and a
renamed dependency. [The Patch derive](docs/reference/derive-patch.md) lists
every attribute and the cases the derive cannot handle.

## Documentation

- [Getting started](docs/getting-started.md), then
  [applying a PATCH request](docs/guides/patch-request.md),
  [the serde round trip](docs/guides/serde.md),
  [JSON Schema and OpenAPI](docs/guides/json-schema.md) and
  [deriving a patch](docs/guides/derive-patch.md).
- Reference: [the three states and the API](docs/reference/presence.md),
  [the Patch derive](docs/reference/derive-patch.md),
  [cargo features, MSRV and no_std](docs/reference/features.md).
- How it works: [why three states](docs/explanation/three-states.md),
  [Presence against nested Options](docs/explanation/option-option.md),
  [design of the derive](docs/explanation/derive-design.md).
- The API docs are on [docs.rs](https://docs.rs/presence-rs).

## Contributing

Pull requests are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) covers the
workflow and the checks a change has to pass.

## License

MIT. See [LICENSE](LICENSE).
