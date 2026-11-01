# Applying a PATCH request

`Presence::apply_to` takes an `&mut Option<T>` field of the model and
applies the three states to it: `Absent` leaves it alone, `Null` sets it
to `None`, `Some(value)` sets it to `Some(value)`. It returns the value it
replaced, so a caller that has to report the old value or undo the write
gets it without reading the field first.

```rust
use presence_rs::Presence;

let mut email = Some("old@example.com".to_string());

assert_eq!(Presence::Absent.apply_to(&mut email), None);
assert_eq!(email.as_deref(), Some("old@example.com"));

let replaced = Presence::Some("new@example.com".to_string()).apply_to(&mut email);
assert_eq!(replaced.as_deref(), Some("old@example.com"));

let cleared = Presence::<String>::Null.apply_to(&mut email);
assert_eq!(cleared.as_deref(), Some("new@example.com"));
assert_eq!(email, None);
```

`Absent` returns `None` whatever the field held, since it reads nothing
and writes nothing.

## A request struct

The request struct holds one `Presence` per model field a client may
change, and its `apply_to` calls `Presence::apply_to` once per field.
`Presence::merge` folds two requests into one: the later field wins unless
it is `Absent`.

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

`#[derive(Patch)]` generates this struct, including its `apply_to` and
`merge`. See [Deriving a patch](derive-patch.md).

## Why merging is safe to batch

`merge` keeps `self` when `later` is `Absent` and takes `later`
otherwise, so applying `a.merge(b)` to a model equals applying `a` and
then `b`. It is associative, and the all-`Absent` patch is its identity.
A queue of PATCH bodies collapses into one write without changing the
result, which is what a service needs to batch them.

```rust
use presence_rs::Presence;

let empty: Presence<u32> = Presence::Absent;
assert_eq!(empty.merge(Presence::Some(1)), Presence::Some(1));
assert_eq!(Presence::Some(1).merge(empty), Presence::Some(1));
assert_eq!(Presence::Some(1).merge(Presence::Null), Presence::Null);
```

## Fields the model cannot clear

`apply_to` needs an `Option<T>` on the model, because `Null` has to put
something there. A model field of plain `T` has no empty value, so a
request for it is an `Option<T>`: `Some` replaces it, `None` leaves it
alone. That is the mapping `#[derive(Patch)]` uses, and
[its reference page](../reference/derive-patch.md) has the generated code.
