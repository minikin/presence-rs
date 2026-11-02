# Why three states

A PATCH request body carries one instruction per field, and JSON gives
three ways to write a field: leave it out, send `null`, send a value.
RFC 7396, JSON Merge Patch, assigns them the meanings this crate uses:
an omitted field is not part of the request, `null` removes the member,
and a value replaces it.

A handler that parses such a body into a struct of `Option<T>` fields
has already lost the first distinction. `None` covers both "the client
said nothing about this field" and "the client asked to clear it", and
the two mean opposite things: one must not touch the stored row, the
other must write a `NULL` into it. A handler that writes `NULL` for
`None` clears fields no client asked it to clear, and one that skips
`None` ignores every request to clear a field.

The usual workaround is a convention outside the type system: a
second set of booleans, a `HashMap<String, Value>` kept beside the
struct, or a `serde_json::Value` inspected by hand. Each works and each
has to be maintained field by field, and a field added to the model
without its entry in the convention compiles and silently drops updates.

`Presence<T>` puts the third state in the type, so the parser records
the client's intent and `apply_to` is the only code that has to know
what each state means. Adding a field to the request means adding one
`Presence` field, and `#[derive(Patch)]` makes that automatic.

## Absent is the conservative state

Three states mean a combinator sometimes has to produce one out of
nothing: `filter` rejecting a value, `xor` of two `Some` values,
`take_if` taking nothing. Every one of those returns `Absent`, not
`Null`, because in PATCH terms `Absent` leaves the stored value alone
while `Null` destroys it. A bug in a filter predicate then costs an
update rather than a column.

`Default` is `Absent` for the same reason, which also makes the default
patch the one that changes nothing, and so the identity of `merge`.

## Beyond PATCH

The distinction is not specific to HTTP. IPLD schemas treat an absent
field and a null field as different, which is where the `is_defined`
name comes from: a `Null` field exists in the structure, an `Absent` one
does not. A database row has the same split between a column that is
`NULL` and a column that was not in the `SELECT` list. A form has it
between a control the user never touched and one the user cleared.

For the comparison with the ways to encode three states in `Option`, see
[Presence against nested Options](option-option.md).
