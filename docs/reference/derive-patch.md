# The Patch derive

`#[derive(Patch)]` comes from the `derive` feature, which re-exports it
from the `presence-rs-derive` crate. Depend on `presence-rs` with that
feature rather than on `presence-rs-derive` directly: the generated code
names `presence-rs` items.

It accepts a struct with named fields, including generics, lifetimes and
`where` clauses, which carry over to the patch. Any other input is a
compile error.

## The generated code

For each kept field, the patch holds:

| Model field | Patch field | `apply_to` |
| --- | --- | --- |
| `Option<T>` | `Presence<T>` | `Presence::apply_to`: `Absent` leaves, `Null` clears, `Some` sets |
| any other `T` | `Option<T>` | `Some` assigns, `None` leaves |

Beside the struct the derive emits an `impl Default`, where every field
is `Absent` or `None`, and an `impl Patch` with:

- `type Target`, the model.
- `apply_to(self, &mut Target)`, the per-field writes above.
- `merge(self, later)`, field by field: `Presence::merge(self, later)`
  for a `Presence` field, `Option::or(later, self)` for an `Option` one.
  Both keep the earlier value when the later field is unset.
- `is_empty(&self)`, true when every field is `Absent` or `None`.

The patch struct keeps the model's visibility and each field's
visibility, copies each field's doc comments, and carries a doc comment
of its own, `A partial update of a {Model}.`, which becomes the
description when the patch derives `JsonSchema`.

## Container attributes

| Attribute | Effect |
| --- | --- |
| `#[patch(name = AccountUpdate)]` | Names the patch struct. The default is the model's name with `Patch` appended. |
| `#[patch(derive(Debug, Clone, Serialize, Deserialize, JsonSchema))]` | Adds derives to the patch. `Default` in this list is ignored, since the patch always derives it. |
| `#[patch(crate = path)]` | The path to `presence-rs` in the generated code, for a crate that renames the dependency. The default is `::presence_rs`. |

`#[patch(skip)]` on a field leaves it out of the patch, so applying a
patch never touches it.

Any other `#[patch(...)]` key is a compile error naming the key.

## Serde attributes

A serde derive is recognized by the last segment of its path in
`#[patch(derive(...))]`, `Serialize` or `Deserialize`, bare or qualified.
With one of them present, each generated field gets the attributes a
patch needs:

```rust,ignore
// From an `Option<T>` model field.
#[serde(default, skip_serializing_if = "::presence_rs::Presence::is_absent")]
nickname: ::presence_rs::Presence<String>,

// From any other model field.
#[serde(
    default,
    skip_serializing_if = "::core::option::Option::is_none",
    deserialize_with = "::presence_rs::__private::some",
    bound(deserialize = "String: ::presence_rs::__private::Deserialize<'de>")
)]
name: ::core::option::Option<String>,
```

`__private::some` is what makes `{"name": null}` a deserialization
error: a non-`Option` model field has nowhere to hold a cleared value, so
the request is rejected at parse time rather than silently ignored. The
`bound` is there because `deserialize_with` stops serde from inferring
`T: Deserialize` for a generic field.

From the model, the derive carries over the container's `rename_all` and
`deny_unknown_fields` and each field's `rename` and `alias`, so the patch
uses the model's wire names. It drops `default`, `skip_serializing` and
`skip_serializing_if`, since it sets its own. A field with
`#[serde(skip)]` or `#[serde(skip_deserializing)]` is left out of the
patch, as `#[patch(skip)]` would do.

All of that applies only to a patch that derives serde. A patch without a
serde derive never touches the wire, so it ignores the model's serde
attributes: a custom codec is no error there, and a `#[serde(skip)]`
field stays in the patch.

## JsonSchema

With `JsonSchema` in `#[patch(derive(...))]`, a field from a non-`Option`
model field gets `#[schemars(with = "T")]`, the schema of the model's
type and not nullable, which matches the rejected `null`. A field from an
`Option<T>` model field gets the schema of `Presence<T>`, a `T` or
`null`. Neither is listed in `required`: with a serde derive schemars
reads the serde attributes above, and without one the derive emits
`#[schemars(default, skip_serializing_if = ...)]` instead.

`JsonSchema` needs the `schemars` feature next to `derive`, and a serde
derive needs the `serde` feature.

## Limits

- The derive sees tokens, not types. It recognizes `Option<T>`,
  `std::option::Option<T>` and `core::option::Option<T>` with exactly one
  type argument. A type alias of `Option` counts as a non-`Option` field
  and cannot be cleared.
- A model field whose type is already a `Presence<T>` counts as a
  non-`Option` field. The patch can replace it with a value but cannot
  set it to `Null`.
- A model field with `#[serde(with)]`, `serialize_with`,
  `deserialize_with` or `flatten` is a compile error when the patch
  derives serde, because the codec handles the model's type and not the
  patch's. The message suggests `#[patch(skip)]`.
- A generic parameter used only by skipped fields is unused in the patch,
  and rustc rejects the struct with E0392.
- A serde derive under another name, such as
  `use serde::Deserialize as De`, is not recognized, so the patch gets no
  serde attributes and a missing key reads as `null`. A same-named derive
  from another crate, such as `rkyv::Deserialize`, is recognized as serde
  and fails to compile.
- With `#[serde(deny_unknown_fields)]` on the model, the patch rejects
  the keys of its `#[patch(skip)]` fields as unknown.
