# Deriving a patch

The `derive` feature generates the request struct of [Applying a PATCH
request](patch-request.md). `#[derive(Patch)]` on a model struct writes a
second struct beside it, with one field per model field, and implements
the `Patch` trait for it: `apply_to`, `merge`, `is_empty`, plus a
`Default` that sets nothing.

```toml
[dependencies]
presence-rs = { version = "0.3.0", features = ["derive", "serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

A model field of type `Option<T>` becomes a `Presence<T>`, which can
leave the field alone, clear it or set it. Any other field type `T`
becomes an `Option<T>`, which can leave it alone or set it, because the
model has nowhere to put a cleared value.

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

Serde comes in through `#[patch(derive(...))]`. With `Serialize` or
`Deserialize` in that list, every generated field gets the two attributes
from [Serde round trip](serde.md), and the model's `rename_all`,
`deny_unknown_fields`, `rename` and `alias` carry over, so the patch
speaks the model's wire names. Without a serde derive the patch never
touches the wire and gets no serde attributes.

`#[patch(skip)]` keeps a field out of the patch, which is how `id` above
stays read-only. The full list of attributes, the generated code and the
cases the derive cannot handle are in [The Patch
derive](../reference/derive-patch.md).

## Merging and emptiness

The trait's `merge` composes two patches field by field, and `is_empty`
answers whether applying one would change anything. A handler can reject
an empty body before it opens a transaction.

```rust
use presence_rs::{Patch, Presence};

#[derive(Debug, PartialEq, Patch)]
struct User {
    name: String,
    nickname: Option<String>,
}

let rename = UserPatch {
    name: Some("Alicia".to_string()),
    nickname: Presence::Absent,
};
let clear_nickname = UserPatch {
    name: None,
    nickname: Presence::Null,
};
assert!(UserPatch::default().is_empty());

let mut user = User {
    name: "Alice".to_string(),
    nickname: Some("al".to_string()),
};
rename.merge(clear_nickname).apply_to(&mut user);
assert_eq!(
    user,
    User {
        name: "Alicia".to_string(),
        nickname: None,
    }
);
```

`Patch` is a trait rather than a set of inherent methods, so a handler
can be generic over it:

```rust
use presence_rs::Patch;

fn apply_if_any<P: Patch>(patch: P, target: &mut P::Target) -> bool {
    if patch.is_empty() {
        return false;
    }
    patch.apply_to(target);
    true
}
```
