//! Acceptance tests: one test per spec scenario, named after the scenario.

use presence_rs::Presence;
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

// Spec 01 — shared borrow

#[test]
fn iterating_a_borrowed_some_yields_a_reference_to_its_value() {
    // Given a Presence::Some(42)
    let presence = Presence::Some(42);

    // When it is iterated with `for x in &presence`
    let mut seen = Vec::new();
    for x in &presence {
        seen.push(x);
    }

    // Then the loop body runs exactly once with x == &42
    assert_eq!(seen, vec![&42]);
    // And the presence is still usable afterwards and equals Presence::Some(42)
    assert_eq!(presence, Presence::Some(42));
}

#[test]
fn iterating_a_borrowed_null_yields_nothing() {
    // Given a Presence::<i32>::Null
    let presence = Presence::<i32>::Null;

    // When it is iterated with `for x in &presence`
    let mut runs = 0;
    for _ in &presence {
        runs += 1;
    }

    // Then the loop body never runs
    assert_eq!(runs, 0);
    // And the presence still equals Presence::Null
    assert_eq!(presence, Presence::Null);
}

#[test]
fn iterating_a_borrowed_absent_yields_nothing() {
    // Given a Presence::<i32>::Absent
    let presence = Presence::<i32>::Absent;

    // When it is iterated with `for x in &presence`
    let mut runs = 0;
    for _ in &presence {
        runs += 1;
    }

    // Then the loop body never runs
    assert_eq!(runs, 0);
    // And the presence still equals Presence::Absent
    assert_eq!(presence, Presence::Absent);
}

#[test]
fn a_borrowed_presence_is_accepted_where_into_iterator_is_required() {
    // Given a function generic over `I: IntoIterator` that collects its items into a Vec
    fn collect_items<I: IntoIterator>(items: I) -> Vec<I::Item> {
        items.into_iter().collect()
    }

    // When it is called with `&Presence::Some(7)`, `&Presence::<i32>::Null`
    // and `&Presence::<i32>::Absent`
    let some = collect_items(&Presence::Some(7));
    let null = collect_items(&Presence::<i32>::Null);
    let absent = collect_items(&Presence::<i32>::Absent);

    // Then it returns vec![&7], an empty Vec and an empty Vec respectively
    assert_eq!(some, vec![&7]);
    assert!(null.is_empty());
    assert!(absent.is_empty());
}

// Spec 01 — mutable borrow

#[test]
fn iterating_a_mutably_borrowed_some_allows_modifying_its_value() {
    // Given a mutable Presence::Some(42)
    let mut presence = Presence::Some(42);

    // When it is iterated with `for x in &mut presence` and the body sets *x = 100
    let mut runs = 0;
    for x in &mut presence {
        *x = 100;
        runs += 1;
    }

    // Then the loop body runs exactly once
    assert_eq!(runs, 1);
    // And the presence afterwards equals Presence::Some(100)
    assert_eq!(presence, Presence::Some(100));
}

#[test]
fn iterating_a_mutably_borrowed_null_or_absent_yields_nothing_and_leaves_the_state_unchanged() {
    // Given a mutable Presence::<i32>::Null and a mutable Presence::<i32>::Absent
    let mut null = Presence::<i32>::Null;
    let mut absent = Presence::<i32>::Absent;

    // When each is iterated with `for x in &mut presence`
    let mut runs = 0;
    for x in &mut null {
        *x = 100;
        runs += 1;
    }
    for x in &mut absent {
        *x = 100;
        runs += 1;
    }

    // Then the loop body never runs for either
    assert_eq!(runs, 0);
    // And they still equal Presence::Null and Presence::Absent respectively
    assert_eq!(null, Presence::Null);
    assert_eq!(absent, Presence::Absent);
}

// Spec 01 — equivalence and lint

#[test]
fn the_clippy_suppression_is_no_longer_needed() {
    // Given the crate with both impls in place
    let source = include_str!("../src/presence.rs");

    // When its source is searched for the `iter_without_into_iter` expectation
    let suppressions = source.matches("iter_without_into_iter").count();

    // Then none is left, so pedantic clippy runs on `iter()` / `iter_mut()` unsuppressed
    assert_eq!(suppressions, 0);
}

// Spec 02 — round-trip with both attributes

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct WithBothAttributes {
    name: String,
    #[serde(default, skip_serializing_if = "Presence::is_absent")]
    age: Presence<u32>,
}

fn round_trip(user: &WithBothAttributes) -> (String, WithBothAttributes) {
    let json = serde_json::to_string(user).expect("serialization succeeds");
    let back = serde_json::from_str(&json).expect("deserialization succeeds");
    (json, back)
}

#[test]
fn a_some_field_round_trips_with_both_attributes() {
    // Given a struct whose `age` field has both attributes
    // And age is Presence::Some(30)
    let user = WithBothAttributes {
        name: "Alice".into(),
        age: Presence::Some(30),
    };

    // When it is serialized to JSON and deserialized back
    let (json, back) = round_trip(&user);

    // Then the JSON is {"name":"Alice","age":30}
    assert_eq!(json, r#"{"name":"Alice","age":30}"#);
    // And the deserialized age equals Presence::Some(30)
    assert_eq!(back.age, Presence::Some(30));
}

#[test]
fn a_null_field_round_trips_with_both_attributes() {
    // Given a struct whose `age` field has both attributes
    // And age is Presence::Null
    let user = WithBothAttributes {
        name: "Bob".into(),
        age: Presence::Null,
    };

    // When it is serialized to JSON and deserialized back
    let (json, back) = round_trip(&user);

    // Then the JSON is {"name":"Bob","age":null}
    assert_eq!(json, r#"{"name":"Bob","age":null}"#);
    // And the deserialized age equals Presence::Null
    assert_eq!(back.age, Presence::Null);
}

#[test]
fn an_absent_field_round_trips_with_both_attributes() {
    // Given a struct whose `age` field has both attributes
    // And age is Presence::Absent
    let user = WithBothAttributes {
        name: "Charlie".into(),
        age: Presence::Absent,
    };

    // When it is serialized to JSON and deserialized back
    let (json, back) = round_trip(&user);

    // Then the JSON is {"name":"Charlie"} with no age key
    assert_eq!(json, r#"{"name":"Charlie"}"#);
    // And the deserialized age equals Presence::Absent
    assert_eq!(back.age, Presence::Absent);
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("proptest-regressions"))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn every_presence_value_survives_a_round_trip_with_both_attributes(
        age in prop_oneof![
            any::<u32>().prop_map(Presence::Some),
            Just(Presence::Null),
            Just(Presence::Absent),
        ]
    ) {
        // Given a struct whose `age` field has both attributes
        // And age is any Presence<u32> (Some of any value, Null, or Absent)
        let user = WithBothAttributes { name: "Dana".into(), age };

        // When it is serialized to JSON and deserialized back
        let (_, back) = round_trip(&user);

        // Then the deserialized struct equals the original
        prop_assert_eq!(back, user);
    }
}

// Spec 02 — lossy configurations

#[test]
fn a_missing_field_without_serde_default_deserializes_as_null() {
    // Given a struct whose `age` field has skip_serializing_if but no #[serde(default)]
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct WithoutDefault {
        name: String,
        #[serde(skip_serializing_if = "Presence::is_absent")]
        age: Presence<u32>,
    }

    // When the JSON {"name":"Charlie"} is deserialized
    let result = serde_json::from_str::<WithoutDefault>(r#"{"name":"Charlie"}"#);

    // Then deserialization succeeds
    let user = result.expect("a missing Presence field is not an error");
    // And age equals Presence::Null, not Presence::Absent
    assert_eq!(user.age, Presence::Null);
}

#[test]
fn an_absent_field_without_skip_serializing_if_comes_back_as_null() {
    // Given a struct whose `age` field has #[serde(default)] but no skip_serializing_if
    // And age is Presence::Absent
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct WithoutSkip {
        name: String,
        #[serde(default)]
        age: Presence<u32>,
    }
    let user = WithoutSkip {
        name: "Charlie".into(),
        age: Presence::Absent,
    };

    // When it is serialized to JSON and deserialized back
    let json = serde_json::to_string(&user).expect("serialization succeeds");
    let back: WithoutSkip = serde_json::from_str(&json).expect("deserialization succeeds");

    // Then the JSON is {"name":"Charlie","age":null}
    assert_eq!(json, r#"{"name":"Charlie","age":null}"#);
    // And the deserialized age equals Presence::Null
    assert_eq!(back.age, Presence::Null);
}
