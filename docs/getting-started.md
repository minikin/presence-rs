# Getting started

```toml
[dependencies]
presence-rs = "0.3.0"
```

The minimum supported Rust version is 1.85. Every feature is off by
default, and [Cargo features, MSRV and no_std](reference/features.md)
lists them. Most uses want `serde`:

```toml
[dependencies]
presence-rs = { version = "0.3.0", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

The `serde` feature depends on serde with `default-features = false` and
without `derive`, since `Presence` implements `Serialize` and
`Deserialize` by hand. A crate that derives serde traits or serializes a
`String` enables serde's `derive` and `std` itself, as the snippet above
does.

## The three states

```rust
use presence_rs::Presence;

let absent: Presence<i32> = Presence::Absent;
let null: Presence<i32> = Presence::Null;
let some: Presence<i32> = Presence::Some(42);

assert!(absent.is_absent());
assert!(null.is_null());
assert!(some.is_present());

// `Null` and `Absent` are both "nullish", and only `Absent` is undefined.
assert!(null.is_nullish());
assert!(null.is_defined());
assert!(!absent.is_defined());
```

`Presence::default()` is `Absent`, and `Display` writes `(absent)` for
`Absent`, `null` for `Null` and the value for `Some`.

## Reading a PATCH body

A `Presence` field tells the three states apart only with both
`#[serde(default)]` and
`#[serde(skip_serializing_if = "Presence::is_absent")]`:

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

[Serde round trip](guides/serde.md) says what each attribute breaks when
it is left out.

## Applying it

`apply_to` writes one field of a request into an `Option` field of the
model:

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

Presence::Null.apply_to(&mut user.nickname);
assert_eq!(user.nickname, None);
```

Next: [Applying a PATCH request](guides/patch-request.md) for a whole
request struct and for folding several requests into one, then
[Deriving a patch](guides/derive-patch.md) to stop writing that struct by
hand.
