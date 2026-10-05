//! `JsonSchema` for [`Presence<T>`].
//!
//! The schema of a `Presence<T>` is the schema of an `Option<T>`: a `T` or `null`. Whether
//! a struct field may be left out is decided by the derive from the field's serde
//! attributes, as documented in the crate-level [Serde section](crate#serde).

extern crate alloc;

use crate::presence::Presence;
use alloc::borrow::Cow;
use schemars::{JsonSchema, Schema, SchemaGenerator};

impl<T: JsonSchema> JsonSchema for Presence<T> {
    // `Option<T>` is always inlined. `every_schema_method_matches_option` fails if a
    // schemars release changes that.
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> Cow<'static, str> {
        <Option<T>>::schema_name()
    }

    fn schema_id() -> Cow<'static, str> {
        <Option<T>>::schema_id()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        <Option<T>>::json_schema(generator)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::Presence;
    use schemars::{JsonSchema, schema_for};

    #[test]
    fn every_schema_method_matches_option() {
        assert_eq!(
            <Presence<u8>>::inline_schema(),
            <Option<u8>>::inline_schema()
        );
        assert_eq!(<Presence<u8>>::schema_name(), <Option<u8>>::schema_name());
        assert_eq!(<Presence<u8>>::schema_id(), <Option<u8>>::schema_id());
    }

    #[test]
    fn a_root_schema_matches_option() {
        assert_eq!(schema_for!(Presence<u8>), schema_for!(Option<u8>));
    }
}
