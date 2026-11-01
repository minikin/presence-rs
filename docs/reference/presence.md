# The three states and the API

```rust
pub enum Presence<T> {
    Absent,
    Null,
    Some(T),
}
```

| State | JSON | PATCH meaning |
| --- | --- | --- |
| `Absent` | `{}` | leave the stored value alone |
| `Null` | `{"field": null}` | clear the stored value |
| `Some(value)` | `{"field": 42}` | set the stored value |

`Presence<T>` derives `Clone`, `Copy`, `Debug`, `Hash`, `PartialEq`,
`Eq`, `PartialOrd` and `Ord`, and it is `#[must_use]`.

`Default::default()` is `Absent`. The variants are ordered
`Absent < Null < Some(_)`, with two `Some` values compared by their
contents. That order is part of the public API and will not change
without a breaking release. `Display`, for a `T` that implements it,
writes `(absent)`, `null` or the value.

For a `T` with `N` values, `Presence<T>` has `N + 2`: `Presence<bool>`
has `Some(true)`, `Some(false)`, `Null` and `Absent`.

The `presence!` macro builds the three states: `presence!()` is `Absent`,
`presence!(null)` is `Null`, and `presence!(expr)` is `Some(expr)`.
`presence!(null)` matches the token `null`, so it is `Null` even when a
variable named `null` is in scope. Wrap the variable in parentheses,
`presence!((null))`, to pass its value.

## Combining states

Every combinator follows one of five rules for `Null` and `Absent`.

| Rule | Methods | Result |
| --- | --- | --- |
| Self decides | `map`, `and`, `and_then`, `filter`, `flatten`, `unzip`, `transpose`, `copied`, `cloned` | A `Null` or `Absent` receiver keeps its state. Only `Some` looks at the closure or the argument. |
| First `Some` wins | `or`, `or_else` | `self` when it is `Some`, otherwise the alternative, whatever its state. |
| `Absent` over `Null` over `Some` | `zip`, `zip_with`, `collect`, `sum`, `product` | `Absent` if any input is `Absent`, otherwise `Null` if any is `Null`, otherwise `Some`. The result does not depend on the order of the inputs. |
| Last non-`Absent` wins | `merge` | `later` unless it is `Absent`, in which case `self`. |
| Exactly one `Some` | `xor` | The `Some` side when exactly one side is `Some`, `Null` when both are `Null`, `Absent` otherwise, two `Some` values included. |

So `Null.and(Absent)` is `Null` and `Null.or(Absent)` is `Absent`, while
`Null.zip(Absent)` is `Absent` whichever side each is on.

A method that has to invent a result out of nothing returns `Absent`:
`filter` rejecting a value, `xor` of two `Some` values, `take_if` taking
nothing. In PATCH terms `Absent` means "leave the field unchanged", which
is the safe answer where `Null` would mean "clear it".

## Querying

| Method | `Some(v)` | `Null` | `Absent` |
| --- | --- | --- | --- |
| `is_present` | `true` | `false` | `false` |
| `is_null` | `false` | `true` | `false` |
| `is_absent` | `false` | `false` | `true` |
| `is_defined` | `true` | `true` | `false` |
| `is_nullish` | `false` | `true` | `true` |
| `is_some_and(f)` | `f(v)` | `false` | `false` |
| `is_absent_or(f)` | `f(v)` | `false` | `true` |
| `is_nullish_or(f)` | `f(v)` | `true` | `true` |
| `len` | `1` | `0` | `0` |
| `is_empty` | `false` | `true` | `true` |

`is_absent_or` is the one that answers `false` for `Null`: a cleared
field is a change, an absent one is not.

## Extracting a value

`unwrap`, `expect(msg)`, `unwrap_or(default)`, `unwrap_or_else(f)` and
`unwrap_or_default` treat `Null` and `Absent` alike. `unwrap` and
`expect` panic, and their messages name the state they met, `Null` or
`Absent`.

`unwrap_or_absent_or_null(absent, null)` is the one that separates them,
with the `Absent` default first, as the name's order says.
`unwrap_or_else_absent_or_null(absent, null)` is its lazy form and calls
only the closure for the state it meets.

`map_or(default, f)`, `map_or_else(default, f)` and `map_or_default(f)`
collapse `Null` and `Absent` into the default as well.

`ok_or(err)` and `ok_or_else(f)` turn `Some` into `Ok` and both other
states into `Err`.

## Mutating in place

| Method | Effect |
| --- | --- |
| `apply_to(&mut Option<T>)` | `Absent` leaves the target, `Null` takes it, `Some` replaces it. Returns the value that was there, and always `None` for `Absent`. |
| `take` | Leaves `Absent` behind and returns the old presence. |
| `take_if(pred)` | Takes only a `Some` whose value passes, otherwise returns `Absent` and leaves the presence alone. |
| `replace(value)` | Stores `Some(value)` and returns the old presence. |
| `insert(value)` | Stores `Some(value)` and returns `&mut T`. |
| `get_or_insert(value)`, `get_or_insert_with(f)`, `get_or_insert_default` | Fill in a `Null` or an `Absent`, then return `&mut T`. |

## Converting

| Method | `Some(v)` | `Null` | `Absent` |
| --- | --- | --- | --- |
| `to_optional` | `Some(v)` | `None` | `None` |
| `to_nullable` | `Some(Some(v))` | `Some(None)` | `None` |

`from_optional(opt)` reads `None` as `Absent`, and `from_nullable(opt)`
is the inverse of `to_nullable`. `From` and `Into` cover
`Option<Option<T>>` in both directions with the same mapping, so code
that already holds a nested `Option` converts without a helper.

`From<T>` wraps a value in `Some`. `From<&Presence<T>>` and
`From<&mut Presence<T>>` give a `Presence<&T>` and a
`Presence<&mut T>`, as `as_ref` and `as_mut` do. With those impls in
place, `Presence::from(&p)` does not infer its target type, so the type
has to be named.

`transpose` turns a `Presence<Result<T, E>>` into a
`Result<Presence<T>, E>`, with `Null` and `Absent` passing through as
`Ok`.

## References

`as_ref`, `as_mut`, `as_deref`, `as_deref_mut`, `as_pin_ref` and
`as_pin_mut` map over the value and keep `Null` and `Absent`. `copied`
and `cloned` are on `Presence<&T>` and `Presence<&mut T>`. `as_slice`
and `as_mut_slice` give a one-element slice for `Some` and an empty one
otherwise.

## Iterating

`iter`, `iter_mut` and `into_iter` yield one item for `Some` and none for
the other two states, through `Iter`, `IterMut` and `IntoIter`, each a
`DoubleEndedIterator`, `ExactSizeIterator` and `FusedIterator`.
`IntoIterator` is implemented for `Presence<T>`, `&Presence<T>` and
`&mut Presence<T>`, so `for x in &presence` works as it does for
`Option`.

`collect`, `sum` and `product` over an iterator of presences follow the
`Absent` over `Null` over `Some` rule. They stream the values instead of
buffering them, so collecting into a type that does not allocate
allocates nothing, and the values before the first `Null` or `Absent`
are combined before the result is discarded. An overflow among them
panics in a debug build even when the result turns out `Null` or
`Absent`.

Reading stops at the first `Absent`. After a `Null` the rest of the input
is still read, because a later `Absent` would decide the result, so an
endless iterator that never yields `Absent` never finishes.

## Serde

With the `serde` feature, `Some(value)` serializes as the value and both
`Null` and `Absent` as `null`. Deserializing gives `Some` for a value and
`Null` for a `null`, and no input produces `Absent`. A struct field needs
`#[serde(default)]` for that, and
`#[serde(skip_serializing_if = "Presence::is_absent")]` to write nothing
for it. See [Serde round trip](../guides/serde.md).

## Schemars

With the `schemars` feature, `Presence<T>` implements `JsonSchema` by
delegating `inline_schema`, `schema_name`, `schema_id` and `json_schema`
to `Option<T>`, so `schema_for!(Presence<u8>)` equals
`schema_for!(Option<u8>)`. See [JSON Schema and
OpenAPI](../guides/json-schema.md).

## Deprecated since 0.3.0

| Deprecated | Replacement |
| --- | --- |
| `unwrap_or_null_default` | `unwrap_or_absent_or_null`, same argument order |
| `is_null_or` | `is_nullish_or`, which is also `true` for `Absent` |
| `to_nested_option` | `to_nullable` |
| `reduce` | `zip_with` |
| `presence::Item` | `presence::IntoIter` |
