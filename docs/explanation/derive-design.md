# Design of the derive

A patch struct written by hand repeats the model. Each field appears
again with a `Presence` around it, with two serde attributes, and once
more inside `apply_to`. Three edits in two places per field, and a missed
one compiles and silently drops updates. `#[derive(Patch)]` exists to
make the patch a function of the model.

## Two field mappings, not one

`Presence<T>` is the right patch field only for a model field that can
hold nothing. `Null` has to write something, and for an `Option<T>`
field that is `None`. A model field of plain `T` has no such value, so
its patch field is an `Option<T>`: `Some` assigns, `None` leaves the
field alone.

That makes `{"name": null}` a request the model cannot satisfy. The
derive rejects it at parse time, with a `deserialize_with` that accepts
any value and errors on `null`, rather than letting it through as a
no-op. A client that sends it gets a deserialization error naming the
field instead of a success and an unchanged row.

The alternative, mapping every field to `Presence<T>` and failing inside
`apply_to`, would move the error past deserialization into the handler
and make `apply_to` fallible for every field of every patch. Keeping
`apply_to` infallible is worth the second mapping.

## Tokens, not types

A derive macro sees the tokens of the model, not resolved types, so the
derive matches the spelling of a field's type: `Option<T>`,
`std::option::Option<T>` or `core::option::Option<T>` with one type
argument. A type alias of `Option` is not recognized, and a field of
that type gets the non-`Option` mapping, which can set the field but not
clear it.

Serde derives in `#[patch(derive(...))]` are recognized the same way, by
the last path segment. `serde::Deserialize` and a bare `Deserialize` both
count, `rkyv::Deserialize` counts wrongly, and
`use serde::Deserialize as De` does not count at all. Token matching
cannot tell those apart, so each case is listed under [the derive's
limits](../reference/derive-patch.md#limits).

The same constraint explains why the serde attributes the derive emits
carry over only `rename_all`, `deny_unknown_fields`, `rename` and
`alias`. Those four shape the wire names, which the patch has to share
with the model. A custom codec cannot carry over: it is written against
the model's field type, and the patch's is a different type. The derive
reports that as an error on the field and suggests `#[patch(skip)]`,
rather than emitting code that fails to compile deeper in.

## A trait, not inherent methods

`apply_to`, `merge` and `is_empty` are methods of the `Patch` trait in
`presence-rs`, which the derive implements, so a handler, a middleware or
a test helper can be generic over any derived patch and reach the whole
interface through one bound.

`merge` and `Default` make a patch a monoid: `merge` is associative, the
default patch is its identity, and applying `a.merge(b)` equals applying
`a` and then `b`. That is what lets a service batch a queue of PATCH
bodies into one write, and it is the property the derive's tests check
field by field.

## Its own crate

A proc-macro crate can export only macros, so the derive lives in
`presence-rs-derive` and `presence-rs`
re-exports it behind the `derive` feature. The two are released together
and depended on with `=`, so a version mismatch between the macro and
the items its output names cannot happen. The generated code refers to
`::presence_rs` by that path, and `#[patch(crate = path)]` overrides it
for a crate that renames the dependency.
