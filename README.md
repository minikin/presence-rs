# Presence

[![CI](https://github.com/minikin/presence-rs/workflows/CI/badge.svg)](https://github.com/minikin/presence-rs/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/presence-rs.svg)](https://crates.io/crates/presence-rs)
[![docs.rs](https://img.shields.io/docsrs/presence-rs)](https://docs.rs/presence-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> A Rust library providing a tri-state type for representing value presence
> in schemas and data structures.

> [!TIP]
> If you what to read more about the motivation behind this crate, check out
> [Stop Losing Intent: Absent, Null, and Value in Rust](https://minikin.me/blog/presence-rs)

- [Presence](#presence)
  - [Overview](#overview)
  - [Cardinality](#cardinality)
  - [Why Not `Option<Option<T>>`?](#why-not-optionoptiont)
  - [Usage](#usage)
  - [Examples](#examples)
    - [Basic Usage](#basic-usage)
    - [Practical Example: API Update Request](#practical-example-api-update-request)
  - [Serde](#serde)
  - [JSON Schema and OpenAPI](#json-schema-and-openapi)
  - [Use Cases](#use-cases)
  - [Contributing](#contributing)
  - [License](#license)

## Overview

`Presence<T>` extends the traditional `Option<T>` two-state model (Some/None)
with an additional distinction between "absent" and "null". The crate is
`#![no_std]` and, with default features, needs no allocator.
This is particularly useful when working with serialization formats like JSON
where the following states are semantically different:

- **Absent**: Field not present in the data structure: `{}`
- **Null**: Field present but explicitly set to null: `{"field": null}`
- **Some**: Field present with a concrete value: `{"field": value}`

## Cardinality

The `Presence` type increases the cardinality (number of possible states) of any
wrapped type by adding two states: `Absent` and `Null`.

| Type             | Valid States                                  | Cardinality |
| ---------------- | --------------------------------------------- | ----------- |
| `bool`           | `true`, `false`                               | 2           |
| `Option<bool>`   | `None`, `Some(true)`, `Some(false)`           | 3           |
| `Presence<bool>` | `Absent`, `Null`, `Some(true)`, `Some(false)` | 4           |

This distinction is particularly important in schema design and APIs where the semantic
difference between "field not present" and "field explicitly set to null" has meaning.

## Why Not `Option<Option<T>>`?

While `Option<Option<T>>` can technically represent three states, `Presence<T>`
offers several advantages:

- **Clarity**: `Presence::Absent`, `Presence::Null`, and `Presence::Some(value)`
are self-documenting. Compare this to `None`, `Some(None)`, and `Some(Some(value))`
where the meaning of nested `None` values is ambiguous.
- **Ergonomics**: Method names like `is_absent()`, `is_null()`, and `is_present()`
clearly express intent, versus checking `option.is_none()` or `option == Some(None)`.
- **Type Safety**: The compiler understands the three distinct states,
making pattern matching more explicit and reducing cognitive load.
- **Semantics**: `Presence` models the domain concept directly rather than
forcing a tri-state model into a two-level optional structure.

```rust
use presence_rs::Presence;

// With Presence - clear and explicit
let value: Presence<i32> = Presence::Null;
match value {
    Presence::Absent => println!("Field not in payload"),
    Presence::Null => println!("Field explicitly null"),
    Presence::Some(v) => println!("Value: {}", v),
}

// With Option<Option<T>> - confusing
let value: Option<Option<i32>> = Some(None);
match value {
    None => println!("Field not in payload"),
    Some(None) => println!("Field explicitly null"), // Wait, which None?
    Some(Some(v)) => println!("Value: {}", v),
}
```

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
presence-rs = "0.3.0"
```

## Examples

### Basic Usage

```rust
use presence_rs::Presence;

// Create Presence values
let absent: Presence<i32> = Presence::Absent;
let null: Presence<i32> = Presence::Null;
let some: Presence<i32> = Presence::Some(42);

// Query the state
assert!(absent.is_absent());
assert!(null.is_null());
assert!(some.is_present());
```

### Practical Example: API Update Request

A PATCH request leaves out fields it does not touch, sends `null` for fields to
clear, and sends values for fields to set. `Presence::apply_to` applies a field to
the stored model (and returns the value it replaced, which this example ignores);
`Presence::merge` folds several requests into one.

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

Enable the `serde` feature. The example below also uses `serde` and `serde_json` directly:

```toml
[dependencies]
presence-rs = { version = "0.3.0", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

`Some(value)` is written as the value; `Null` and `Absent` are both written as `null`.
A struct field keeps all three states apart only when it has **both** attributes:

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

## Use Cases

This type is particularly useful in:

- **API clients/servers** where you need to distinguish between a field not
being sent vs. being explicitly set to null
- **Partial updates** where absence means "don't change" vs. null means "clear the value"
- **Schema validation** where field presence has semantic meaning
- **GraphQL implementations** where null and undefined are distinct concepts
- **Database operations** where you need to differentiate between "not provided"
and "set to NULL"

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
