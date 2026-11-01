//! Support code for `#[derive(Patch)]`. Not public API: anything here can change in
//! any release.

pub use serde::Deserialize;
use serde::de::{Deserializer, Error};

/// Deserializes a value that must not be `null`, for a patch field whose model field
/// cannot hold one.
///
/// It goes through `Option<T>` rather than `T`, so `null` is rejected even when `T`
/// itself accepts it, as a type alias of `Option` does, and so formats that are not
/// self-describing read the same tag that `Option<T>`'s `Serialize` wrote.
///
/// # Errors
///
/// When the input is `null`, or is not a valid `T`.
pub fn some<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    match Option::<T>::deserialize(deserializer)? {
        Some(value) => Ok(Some(value)),
        None => Err(D::Error::custom(
            "invalid type: null, expected a value: this field cannot be cleared",
        )),
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::string::ToString;

    fn from_json<T: serde::de::DeserializeOwned>(json: &str) -> serde_json::Result<Option<T>> {
        some(&mut serde_json::Deserializer::from_str(json))
    }

    #[test]
    fn some_wraps_a_value_in_some() {
        assert_eq!(from_json::<u8>("5").unwrap(), Some(5));
    }

    #[test]
    fn some_rejects_null() {
        let error = from_json::<u8>("null").unwrap_err();
        assert!(
            error.to_string().starts_with("invalid type: null"),
            "{error}"
        );
    }

    #[test]
    fn some_rejects_null_even_for_a_type_that_accepts_it() {
        assert!(from_json::<Option<u8>>("null").is_err());
        assert_eq!(from_json::<Option<u8>>("1").unwrap(), Some(Some(1)));
    }
}
