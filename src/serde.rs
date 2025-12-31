//! Serde serialization and deserialization support for [`Presence<T>`].
//!
//! This module provides `Serialize` and `Deserialize` implementations for `Presence<T>`,
//! enabling seamless JSON and other format support.
//!
//! # Important: Round-Trip Preservation
//!
//! To correctly preserve all three states on round-trip, use both attributes:
//! - `#[serde(default)]` — deserializes missing fields as `Absent`
//! - `#[serde(skip_serializing_if = "Presence::is_absent")]` — omits `Absent` fields from output
//!
//! Without `skip_serializing_if`, `Absent` serializes as `null` and becomes `Null` after round-trip.
//!
//! # Serialization Behavior
//!
//! - `Some(value)` → Serializes the value directly
//! - `Null` → Serializes as `null`
//! - `Absent` → Serializes as `null` (use `skip_serializing_if` to omit the field)
//!
//! # Deserialization Behavior
//!
//! - `value` → `Some(value)`
//! - `null` → `Null`
//! - Missing field → `Absent` (only when field has `#[serde(default)]`)
//!
//! # Examples
//!
//! ## Basic Serialization
//!
//! ```
//! # #[cfg(feature = "serde")] {
//! use presence_rs::Presence;
//!
//! let some = Presence::Some(42);
//! let json = serde_json::to_string(&some).unwrap();
//! assert_eq!(json, "42");
//!
//! let null: Presence<i32> = Presence::Null;
//! let json = serde_json::to_string(&null).unwrap();
//! assert_eq!(json, "null");
//! # }
//! ```
//!
//! ## Struct with Presence Fields
//!
//! ```
//! # #[cfg(feature = "serde")] {
//! use presence_rs::Presence;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct User {
//!     name: String,
//!     #[serde(skip_serializing_if = "Presence::is_absent")]
//!     age: Presence<u32>,
//! }
//!
//! // Some(30) - field present with value
//! let user = User { name: "Alice".into(), age: Presence::Some(30) };
//! let json = serde_json::to_string(&user).unwrap();
//! assert_eq!(json, r#"{"name":"Alice","age":30}"#);
//!
//! // Null - field present but null
//! let user = User { name: "Bob".into(), age: Presence::Null };
//! let json = serde_json::to_string(&user).unwrap();
//! assert_eq!(json, r#"{"name":"Bob","age":null}"#);
//!
//! // Absent - field omitted from JSON
//! let user = User { name: "Charlie".into(), age: Presence::Absent };
//! let json = serde_json::to_string(&user).unwrap();
//! assert_eq!(json, r#"{"name":"Charlie"}"#);
//! # }
//! ```

use crate::presence::Presence;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl<T: Serialize> Serialize for Presence<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Presence::Some(value) => serializer.serialize_some(value),
            Presence::Null => serializer.serialize_none(),
            Presence::Absent => serializer.serialize_none(),
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
    use super::*;

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
