# JSON Schema and OpenAPI

With the `schemars` feature, `Presence<T>` implements
`schemars::JsonSchema` (schemars 1.x) with the schema of an `Option<T>`:
a `T` or `null`. Every method of the impl delegates to `Option<T>`, so
the two schemas are equal, names and ids included. The feature needs
`alloc`.

```toml
[dependencies]
presence-rs = { version = "0.3.0", features = ["serde", "schemars"] }
schemars = "1"
```

The two serde attributes from [Serde round trip](serde.md) also drive the
schema, since schemars reads them: with both, the field is not listed in
`required`.

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

Without `#[serde(default)]`, schemars lists the field as required.
Without `skip_serializing_if`, it adds `"default": null` to the property,
which reads as "a missing field means `null`", the opposite of `Absent`.
Neither matches what the code then does with the payload, so the
attributes are worth checking against the generated schema in a test.

## utoipa

utoipa has no `Presence` support. Describe a `Presence<T>` field to it as
an `Option<T>` with `#[schema(value_type = Option<T>)]`. Together with
the two serde attributes, the field is nullable and not required:

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

## A schema cannot express absence

`{}`, `{"nickname": null}` and `{"nickname": "neo"}` all validate against
the schema above, which is what the three states need. The schema says
nothing about the difference between the first two, because JSON Schema
has no vocabulary for "this field being absent means something other
than null". That distinction lives in the Rust type and in the handler
that applies the patch.

With `#[derive(Patch)]` and `JsonSchema` in `#[patch(derive(...))]`, a
field derived from a non-`Option` model field gets the schema of the
model's type and is not nullable, which matches a patch that rejects
`null` there. [The Patch derive](../reference/derive-patch.md) has the
generated attributes.
