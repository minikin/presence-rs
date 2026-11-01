# Serde round trip

The `serde` feature implements `Serialize` and `Deserialize` for
`Presence<T>` by hand. `Some(value)` is written as the value, and both
`Null` and `Absent` are written as `null`. Reading a `null` back gives
`Null`, and a value gives `Some`.

Nothing in those impls can produce `Absent`: the state comes from a
struct field that is missing from the input, which is serde's job and not
the type's. So a `Presence` field keeps all three states apart only with
both attributes:

- `#[serde(default)]` makes a missing field `Absent`. Without it, a
  missing `Presence` field is read exactly like `null`, the same rule
  serde applies to `Option`, so it silently becomes `Null`.
- `#[serde(skip_serializing_if = "Presence::is_absent")]` leaves `Absent`
  out of the output. Without it, `Absent` is written as `null` and reads
  back as `Null`.

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

assert_eq!(serde_json::to_string(&set).unwrap(), r#"{"nickname":"neo"}"#);
assert_eq!(serde_json::to_string(&cleared).unwrap(), r#"{"nickname":null}"#);
assert_eq!(serde_json::to_string(&untouched).unwrap(), "{}");
```

With both attributes the JSON that comes in is the JSON that goes out, so
a service can read a PATCH body, log it and forward it with its meaning
intact. Drop one attribute and the round trip collapses one state into
another:

| Attributes | `{}` reads as | `Absent` writes as |
| --- | --- | --- |
| both | `Absent` | omitted |
| `default` only | `Absent` | `null` |
| `skip_serializing_if` only | `Null` | omitted |
| no attribute | `Null` | `null` |

A standalone `Presence<T>`, outside a struct field, reaches two of the
three states: `serde_json::from_str::<Presence<i32>>("null")` is `Null`,
and no input gives `Absent`.

`#[derive(Patch)]` puts both attributes on every generated field, so a
derived patch round-trips without any attribute written by hand. See
[Deriving a patch](derive-patch.md).

## No allocator needed

The impls are written against serde's core traits and allocate nothing of
their own, so the `serde` feature works in `no_std` without an allocator.
Serializing a `String` still needs one, from serde's own features rather
than from this crate. [Cargo features, MSRV and
no_std](../reference/features.md) has the detail.
