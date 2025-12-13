# Presence

[![CI](https://github.com/mnkn/presence-rs/workflows/CI/badge.svg)](https://github.com/mnkn/presence-rs/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/presence-rs.svg)](https://crates.io/crates/presence-rs)
[![Documentation](https://docs.rs/presence-rs/badge.svg)](https://docs.rs/presence-rs)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

> A Rust library providing a tri-state type for representing value presence
> in schemas and data structures.

- [Presence](#presence)
  - [Overview](#overview)
  - [Cardinality](#cardinality)
  - [Why Not `Option<Option<T>>`?](#why-not-optionoptiont)
  - [Usage](#usage)
  - [Examples](#examples)
    - [Basic Usage](#basic-usage)
    - [Practical Example: API Update Request](#practical-example-api-update-request)
  - [Use Cases](#use-cases)
  - [Contributing](#contributing)
  - [License](#license)

## Overview

`Presence<T>` extends the traditional `Option<T>` two-state model (Some/None)
with an additional distinction between "absent" and "null".
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
// With Presence - clear and explicit
match value {
    Presence::Absent => println!("Field not in payload"),
    Presence::Null => println!("Field explicitly null"),
    Presence::Some(v) => println!("Value: {}", v),
}

// With Option<Option<T>> - confusing
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
presence-rs = "0.1.0"
```

## Examples

### Basic Usage

```rust
use presence_rs::presence::Presence;

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

```rust
use presence_rs::presence::Presence;

#[derive(Debug)]
struct UserUpdate {
    name: Presence<String>,
    email: Presence<String>,
    age: Presence<u32>,
}

fn apply_update(current_name: &str, update: UserUpdate) -> String {
    match update.name {
        Presence::Absent => {
            // Field not provided - keep current value
            println!("Name unchanged: {}", current_name);
            current_name
        }
        Presence::Null => {
            // Field explicitly set to null - clear it
            println!("Name cleared");
            String::new()
        }
        Presence::Some(new_name) => {
            // Field has a new value - update it
            println!("Name updated to: {}", new_name);
            new_name
        }
    }
}

// Example: Partial update where only email is provided
let update = UserUpdate {
    name: Presence::Absent, // Not in request payload
    email: Presence::Some("new@example.com".to_string()),
    age: Presence::Null, // Explicitly set to null
};

apply_update("Alice".to_string(), update);
// Output: "Name unchanged: Alice"
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

This project is licensed under the MIT License - see the LICENSE file for details.