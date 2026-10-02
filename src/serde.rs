//! `Serialize` and `Deserialize` for [`Presence<T>`].
//!
//! How to keep `Absent`, `Null` and `Some` apart in a struct field is documented in the
//! crate-level [Serde section](crate#serde).

use crate::presence::Presence;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl<T: Serialize> Serialize for Presence<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Presence::Some(value) => serializer.serialize_some(value),
            Presence::Null | Presence::Absent => serializer.serialize_none(),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Presence<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(|opt| match opt {
            Some(value) => Presence::Some(value),
            None => Presence::Null,
        })
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::string::{String, ToString};

    #[test]
    fn test_serialize_some() {
        let p = Presence::Some(42);
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, "42");
    }

    #[test]
    fn test_serialize_null() {
        let p: Presence<i32> = Presence::Null;
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, "null");
    }

    #[test]
    fn test_serialize_absent() {
        let p: Presence<i32> = Presence::Absent;
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, "null");
    }

    #[test]
    fn test_deserialize_value() {
        let json = "42";
        let p: Presence<i32> = serde_json::from_str(json).unwrap();
        assert_eq!(p, Presence::Some(42));
    }

    #[test]
    fn test_deserialize_null() {
        let json = "null";
        let p: Presence<i32> = serde_json::from_str(json).unwrap();
        assert_eq!(p, Presence::Null);
    }

    #[test]
    fn test_struct_with_presence() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct Data {
            name: String,
            #[serde(skip_serializing_if = "Presence::is_absent")]
            age: Presence<u32>,
        }

        let data = Data {
            name: "Alice".to_string(),
            age: Presence::Some(30),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert_eq!(json, r#"{"name":"Alice","age":30}"#);

        let data = Data {
            name: "Bob".to_string(),
            age: Presence::Null,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert_eq!(json, r#"{"name":"Bob","age":null}"#);

        let data = Data {
            name: "Charlie".to_string(),
            age: Presence::Absent,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert_eq!(json, r#"{"name":"Charlie"}"#);
    }
}
