#![cfg(feature = "serde")]

use presence_rs::Presence;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestStruct {
    #[serde(default, skip_serializing_if = "Presence::is_absent")]
    field: Presence<i32>,
}

#[test]
fn test_serialize_some() {
    let data = TestStruct {
        field: Presence::Some(42),
    };
    let json = serde_json::to_string(&data).unwrap();
    assert_eq!(json, r#"{"field":42}"#);
}

#[test]
fn test_serialize_null() {
    let data = TestStruct {
        field: Presence::Null,
    };
    let json = serde_json::to_string(&data).unwrap();
    assert_eq!(json, r#"{"field":null}"#);
}

#[test]
fn test_serialize_absent() {
    let data = TestStruct {
        field: Presence::Absent,
    };
    let json = serde_json::to_string(&data).unwrap();
    assert_eq!(json, r#"{}"#);
}

#[test]
fn test_deserialize_some() {
    let json = r#"{"field":42}"#;
    let data: TestStruct = serde_json::from_str(json).unwrap();
    assert_eq!(data.field, Presence::Some(42));
}

#[test]
fn test_deserialize_null() {
    let json = r#"{"field":null}"#;
    let data: TestStruct = serde_json::from_str(json).unwrap();
    assert_eq!(data.field, Presence::Null);
}

#[test]
fn test_deserialize_absent() {
    let json = r#"{}"#;
    let data: TestStruct = serde_json::from_str(json).unwrap();
    assert_eq!(data.field, Presence::Absent);
}

#[test]
fn test_round_trip_some() {
    let original = TestStruct {
        field: Presence::Some(42),
    };
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: TestStruct = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_round_trip_null() {
    let original = TestStruct {
        field: Presence::Null,
    };
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: TestStruct = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_round_trip_absent() {
    let original = TestStruct {
        field: Presence::Absent,
    };
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: TestStruct = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MultiFieldStruct {
    #[serde(default, skip_serializing_if = "Presence::is_absent")]
    field1: Presence<String>,
    #[serde(default, skip_serializing_if = "Presence::is_absent")]
    field2: Presence<i32>,
    #[serde(default, skip_serializing_if = "Presence::is_absent")]
    field3: Presence<bool>,
}

#[test]
fn test_mixed_presence_states() {
    let data = MultiFieldStruct {
        field1: Presence::Some("hello".to_string()),
        field2: Presence::Null,
        field3: Presence::Absent,
    };
    let json = serde_json::to_string(&data).unwrap();
    assert_eq!(json, r#"{"field1":"hello","field2":null}"#);

    let deserialized: MultiFieldStruct = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.field1, Presence::Some("hello".to_string()));
    assert_eq!(deserialized.field2, Presence::Null);
    assert_eq!(deserialized.field3, Presence::Absent);
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct NestedStruct {
    #[serde(default, skip_serializing_if = "Presence::is_absent")]
    inner: Presence<InnerStruct>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct InnerStruct {
    value: i32,
}

#[test]
fn test_nested_presence() {
    let data = NestedStruct {
        inner: Presence::Some(InnerStruct { value: 42 }),
    };
    let json = serde_json::to_string(&data).unwrap();
    assert_eq!(json, r#"{"inner":{"value":42}}"#);

    let deserialized: NestedStruct = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, data);
}

#[test]
fn test_nested_presence_null() {
    let data = NestedStruct {
        inner: Presence::Null,
    };
    let json = serde_json::to_string(&data).unwrap();
    assert_eq!(json, r#"{"inner":null}"#);

    let deserialized: NestedStruct = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, data);
}

#[test]
fn test_nested_presence_absent() {
    let data = NestedStruct {
        inner: Presence::Absent,
    };
    let json = serde_json::to_string(&data).unwrap();
    assert_eq!(json, r#"{}"#);

    let deserialized: NestedStruct = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, data);
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct VecPresenceStruct {
    items: Vec<Presence<i32>>,
}

#[test]
fn test_vec_of_presence() {
    let data = VecPresenceStruct {
        items: vec![
            Presence::Some(1),
            Presence::Null,
            Presence::Some(3),
            Presence::Absent,
        ],
    };
    let json = serde_json::to_string(&data).unwrap();
    // Note: Absent in a vec serializes as null since elements can't be "missing" in an array
    assert_eq!(json, r#"{"items":[1,null,3,null]}"#);

    let deserialized: VecPresenceStruct = serde_json::from_str(&json).unwrap();
    // After round-trip, Absent becomes Null (expected behavior in arrays)
    assert_eq!(deserialized.items[0], Presence::Some(1));
    assert_eq!(deserialized.items[1], Presence::Null);
    assert_eq!(deserialized.items[2], Presence::Some(3));
    assert_eq!(deserialized.items[3], Presence::Null);
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct OptionalPresence {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    field: Option<Presence<i32>>,
}

#[test]
fn test_option_of_presence() {
    // Option<Presence<T>> allows for even more states
    let data1 = OptionalPresence {
        field: Some(Presence::Some(42)),
    };
    let json1 = serde_json::to_string(&data1).unwrap();
    assert_eq!(json1, r#"{"field":42}"#);
    let deserialized1: OptionalPresence = serde_json::from_str(&json1).unwrap();
    assert_eq!(data1, deserialized1);

    let data2 = OptionalPresence {
        field: Some(Presence::Null),
    };
    let json2 = serde_json::to_string(&data2).unwrap();
    assert_eq!(json2, r#"{"field":null}"#);
    let deserialized2: OptionalPresence = serde_json::from_str(&json2).unwrap();
    // When deserializing {"field":null} into Option<Presence<T>>, we get None
    // because serde deserializes null as None for Option
    assert_eq!(deserialized2.field, None);

    let data3 = OptionalPresence { field: None };
    let json3 = serde_json::to_string(&data3).unwrap();
    assert_eq!(json3, r#"{}"#);
    let deserialized3: OptionalPresence = serde_json::from_str(&json3).unwrap();
    assert_eq!(data3, deserialized3);
}
