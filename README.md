# Presence

[![CI](https://github.com/minikin/presence-rs/workflows/CI/badge.svg)](https://github.com/minikin/presence-rs/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/presence-rs.svg)](https://crates.io/crates/presence-rs)
[![docs.rs](https://img.shields.io/docsrs/presence-rs)](https://docs.rs/presence-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

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
PATCH requests. Serde and JSON Schema support sit behind features. The crate
is `#![no_std]` and, with default features, needs no allocator.

The blog post [Stop Losing Intent: Absent, Null, and Value in
Rust](https://minikin.me/blog/presence-rs) explains the motivation.

## Install

```toml
[dependencies]
presence-rs = "0.3.0"
```

The minimum supported Rust version is 1.85.

## Why not `Option<Option<T>>`?

`Option<Option<T>>` has three states too, but `None`, `Some(None)` and
`Some(Some(v))` do not say which one means "missing" and which "null", and
every reader has to remember the convention. `Presence` names the states,
and its methods (`is_absent`, `is_null`, `is_present`) name the checks:

```rust
use presence_rs::Presence;

// With Presence
let value: Presence<i32> = Presence::Null;
match value {
    Presence::Absent => println!("Field not in payload"),
    Presence::Null => println!("Field explicitly null"),
    Presence::Some(v) => println!("Value: {}", v),
}

// With Option<Option<T>>
let value: Option<Option<i32>> = Some(None);
match value {
    None => println!("Field not in payload"),
    Some(None) => println!("Field explicitly null"), // Wait, which None?
    Some(Some(v)) => println!("Value: {}", v),
}
```

With serde, `Option<Option<T>>` also needs
[`serde_with::rust::double_option`](https://docs.rs/serde_with/latest/serde_with/rust/double_option/index.html)
to tell `null` from a missing field: every such field carries
`#[serde(default, skip_serializing_if = "Option::is_none", with = "::serde_with::rust::double_option")]`.
`Presence` implements `Serialize` and `Deserialize` itself, so a field needs only
`default` and `skip_serializing_if` (see [Serde](#serde)). Code that already uses
`Option<Option<T>>` converts with `From` in both directions: `None` is `Absent`,
`Some(None)` is `Null` and `Some(Some(v))` is `Some(v)`.

## Quick start

```rust
use presence_rs::Presence;

let absent: Presence<i32> = Presence::Absent;
let null: Presence<i32> = Presence::Null;
let some: Presence<i32> = Presence::Some(42);

assert!(absent.is_absent());
assert!(null.is_null());
assert!(some.is_present());
```

## Applying a PATCH request

`apply_to` applies one field of a request to the stored model and returns the
value it replaced, which this example ignores. `merge` folds several requests
into one: the later field wins unless it is `Absent`.

```rust
use presence_rs::Presence;

#[derive(Debug, PartialEq)]
struct User {
    name: Option<String>,
    email: Option<String>,
    age: Option<u32>,
}

struct UserUpdate {
    name: Presence<String>,
    email: Presence<String>,
    age: Presence<u32>,
}

impl UserUpdate {
    fn apply_to(self, user: &mut User) {
        self.name.apply_to(&mut user.name);
        self.email.apply_to(&mut user.email);
        self.age.apply_to(&mut user.age);
    }

    fn merge(self, later: UserUpdate) -> UserUpdate {
        UserUpdate {
            name: self.name.merge(later.name),
            email: self.email.merge(later.email),
            age: self.age.merge(later.age),
        }
    }
}

let mut user = User {
    name: Some("Alice".to_string()),
    email: Some("alice@example.com".to_string()),
    age: Some(30),
};

// Only the email is set and the age is cleared; the name is not in the payload.
let update = UserUpdate {
    name: Presence::Absent,
    email: Presence::Some("new@example.com".to_string()),
    age: Presence::Null,
};

// A later request sets the age again; merged, it wins over the earlier `null`.
let later = UserUpdate {
    name: Presence::Absent,
    email: Presence::Absent,
    age: Presence::Some(31),
};

update.merge(later).apply_to(&mut user);

assert_eq!(
    user,
    User {
        name: Some("Alice".to_string()),
        email: Some("new@example.com".to_string()),
        age: Some(31),
    }
);
```

## Serde

Enable the `serde` feature. The example also uses `serde` and `serde_json`
directly:

```toml
[dependencies]
presence-rs = { version = "0.3.0", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

`Some(value)` is written as the value, and `Null` and `Absent` are both
written as `null`. A struct field keeps all three states apart only when it
has **both** attributes:

- `#[serde(default)]` makes a missing field `Absent`. Without it, a missing `Presence`
  field is read exactly like `null` (the same rule serde applies to `Option`), so it
  silently becomes `Null`.
- `#[serde(skip_serializing_if = "Presence::is_absent")]` leaves `Absent` out of the
  output. Without it, `Absent` is written as `null` and reads back as `Null`.

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

## JSON Schema and OpenAPI

With the `schemars` feature, `Presence<T>` implements `schemars::JsonSchema`
(schemars 1.x) with the schema of an `Option<T>`: a `T` or `null`. The two serde
attributes from [Serde](#serde) also drive the schema: with both, the field is not
listed in `required`. The feature needs `alloc`.

```toml
[dependencies]
presence-rs = { version = "0.3.0", features = ["serde", "schemars"] }
schemars = "1"
```

```rust
use presence_rs::Presence;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, JsonSchema)]
struct Patch {
    #[serde(default, skip_serializing_if = "Presence::is_absent")]
    nickname: Presence<String>,
}

let schema = serde_json::to_value(schema_for!(Patch)).unwrap();
assert_eq!(schema["properties"]["nickname"], json!({"type": ["string", "null"]}));
assert!(schema.get("required").is_none());
```

Without `#[serde(default)]`, schemars lists the field as required. Without
`skip_serializing_if`, it adds `"default": null` to the property, which reads as
"a missing field means `null`", the opposite of `Absent`.

utoipa has no `Presence` support. Describe a `Presence<T>` field to it as an
`Option<T>` with `#[schema(value_type = Option<T>)]`. Together with the two serde
attributes, the field is nullable and not required:

```rust
use presence_rs::Presence;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{PartialSchema, ToSchema};

#[derive(Serialize, Deserialize, ToSchema)]
struct Patch {
    #[serde(default, skip_serializing_if = "Presence::is_absent")]
    #[schema(value_type = Option<String>)]
    nickname: Presence<String>,
}

let schema = serde_json::to_value(Patch::schema()).unwrap();
assert_eq!(schema["properties"]["nickname"], json!({"type": ["string", "null"]}));
assert!(schema.get("required").is_none());
```

## Deriving a patch

The `derive` feature writes the request struct of [Applying a PATCH
request](#applying-a-patch-request) for you. `#[derive(Patch)]` on `User`
generates `UserPatch`, where a model field of type `Option<T>` becomes a
`Presence<T>` and any other field `T` becomes an `Option<T>`, which can set the
field but not clear it. `UserPatch` implements the `Patch` trait: `apply_to`,
`merge`, `is_empty`, and a `Default` that sets nothing.

```toml
[dependencies]
presence-rs = { version = "0.3.0", features = ["derive", "serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

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

The options:

- `#[patch(name = AccountUpdate)]` names the struct. The default is the model's
  name followed by `Patch`.
- `#[patch(derive(...))]` adds derives to the patch. With `Serialize` or
  `Deserialize`, bare or under `serde::`, every field gets the two serde
  attributes from [Serde](#serde), a `null` for a non-`Option` field fails to
  deserialize, and the model's `rename_all`, `deny_unknown_fields`, `rename`
  and `alias` carry over. With `JsonSchema`, a non-`Option` field has the
  schema of its model type, not nullable, and no field is listed in
  `required`.
- `#[patch(skip)]` leaves a field out of the patch, and applying the patch
  never touches it. When the patch derives serde, `#[serde(skip)]` and
  `#[serde(skip_deserializing)]` on the model field do the same.
- `#[patch(crate = path)]` names the path to presence-rs, for a crate that
  renames the dependency.

Limits of the derive:

- It sees tokens, not types, so it does not recognize a type alias of `Option`.
  A field of such a type counts as a non-`Option` field and cannot be cleared.
- A model field with `#[serde(with)]`, `serialize_with`, `deserialize_with` or
  `flatten` needs `#[patch(skip)]` when the patch derives serde, because the
  codec handles the model's type and not the patch's.
- A generic parameter used only by skipped fields is unused in the patch, and
  rustc rejects the struct (E0392).
- Serde derives on the patch need the `serde` feature next to `derive`, and
  `JsonSchema` needs the `schemars` feature.
- A serde derive is recognized by its last path segment, `Serialize` or
  `Deserialize`. Under another name, such as `use serde::Deserialize as De`,
  the patch gets no serde attributes and a missing key reads as `null`. A
  same-named derive from another crate, such as `rkyv::Deserialize`, gets
  serde attributes and fails to compile.
- When the model has `#[serde(deny_unknown_fields)]`, the patch rejects the
  keys of its `#[patch(skip)]` fields as unknown.
- A model field that is already a `Presence<T>` counts as a non-`Option`
  field. The patch can replace it with a value but cannot set it to `Null`.

## Contributing

Pull requests are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) covers the
workflow and the checks a change has to pass.

## License

MIT. See [LICENSE](LICENSE).
