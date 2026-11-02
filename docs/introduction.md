# Presence

`Presence<T>` is `Option<T>` with one more state. A JSON field can be left
out, sent as `null` or sent with a value, and a PATCH request means
something different by each: leave the stored value alone, clear it, or set
it. `Option<T>` has two states for those three, so the difference is lost
the moment the request is parsed. `Presence<T>` keeps it:

- `Absent`: the field is not there, `{}`
- `Null`: the field is there and `null`, `{"field": null}`
- `Some(value)`: the field has a value, `{"field": 42}`

The API follows `Option`'s, so `map`, `and_then`, `unwrap_or` and the rest
work as you expect. On top of it, `apply_to` and `merge` apply and combine
PATCH requests, and `#[derive(Patch)]` writes the request struct for a
model struct.

- [Getting started](getting-started.md) installs the crate and runs the
  three states through serde.
- [Applying a PATCH request](guides/patch-request.md) covers `apply_to`
  and `merge`, the two methods the rest of the crate is built around.
- [Serde round trip](guides/serde.md) has the two field attributes that a
  `Presence` field needs, and what each one breaks when it is missing.
- [JSON Schema and OpenAPI](guides/json-schema.md) covers the `schemars`
  feature and how to describe a `Presence` field to utoipa.
- [Deriving a patch](guides/derive-patch.md) generates the patch struct
  instead of writing it by hand. The
  [reference page](reference/derive-patch.md) lists its attributes and
  limits.
- [The three states and the API](reference/presence.md) says which state
  every combinator returns.
- [Why three states](explanation/three-states.md) and
  [Presence against nested Options](explanation/option-option.md) cover
  the motivation and the alternatives.

The blog post [Stop Losing Intent: Absent, Null, and Value in
Rust](https://minikin.me/blog/presence-rs) works through the problem at
length.
