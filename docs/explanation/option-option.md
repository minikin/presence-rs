# Presence against nested Options

`Option<Option<T>>` has three inhabitants, so it can carry the three
states, but nothing in it says which inhabitant is which. `None`,
`Some(None)` and `Some(Some(v))` have no names for "missing" and "null",
and every reader has to recall the convention the code picked:

```rust
use presence_rs::Presence;

let value: Presence<i32> = Presence::Null;
match value {
    Presence::Absent => println!("Field not in payload"),
    Presence::Null => println!("Field explicitly null"),
    Presence::Some(v) => println!("Value: {}", v),
}

let value: Option<Option<i32>> = Some(None);
match value {
    None => println!("Field not in payload"),
    Some(None) => println!("Field explicitly null"), // Wait, which None?
    Some(Some(v)) => println!("Value: {}", v),
}
```

The difference shows up in the predicates too. `Presence` has
`is_absent`, `is_null` and `is_present`, where the nested `Option` needs
`matches!(x, Some(None))` written out at each use, with the same chance
of swapping the two arms.

The methods differ in what they can express. `Option<Option<T>>`
inherits `Option`'s API at the outer level, so `map` hands the closure an
`Option<T>` and the inner state has to be unwrapped by hand.
`Presence::map` maps the value and keeps `Null` and `Absent` as they are.
Where a two-state method would have to collapse the two, the crate offers
both: `unwrap_or` treats them alike, `unwrap_or_absent_or_null` takes a
default for each. [The three states and the
API](../reference/presence.md) lists which rule each combinator follows.

`From` converts in both directions, with `None` as `Absent`,
`Some(None)` as `Null` and `Some(Some(v))` as `Some(v)`, so a codebase
that already stores nested `Option`s can convert at its boundary:

```rust
use presence_rs::Presence;

let nested: Option<Option<i32>> = Some(None);
let presence: Presence<i32> = nested.into();
assert_eq!(presence, Presence::Null);

let back: Option<Option<i32>> = presence.into();
assert_eq!(back, Some(None));
```

`to_nullable` and `from_nullable` are the same mapping as named methods,
for the places where inference needs help.

## serde_with::rust::double_option

With serde, a nested `Option` does not tell `null` from a missing field
on its own. The field needs
[`serde_with::rust::double_option`](https://docs.rs/serde_with/latest/serde_with/rust/double_option/index.html),
which means every such field carries:

```rust,ignore
#[serde(default, skip_serializing_if = "Option::is_none", with = "::serde_with::rust::double_option")]
```

`Presence` implements `Serialize` and `Deserialize` itself, so the same
field needs only `default` and `skip_serializing_if`, and no extra
dependency:

```rust,ignore
#[serde(default, skip_serializing_if = "Presence::is_absent")]
```

The attribute count is the minor part. `with = "double_option"` also has
to agree with the type about which nesting level means what, and nothing
checks that it does. The [serde round trip](../guides/serde.md) page
covers what each of the two remaining attributes is for, and
`#[derive(Patch)]` writes both of them.
