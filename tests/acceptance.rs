//! Acceptance tests: one test per spec scenario, named after the scenario.

use presence_rs::Presence;
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

fn persisted_config() -> ProptestConfig {
    ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "proptest-regressions",
        ))),
        ..ProptestConfig::default()
    }
}

fn any_presence() -> impl Strategy<Value = Presence<i32>> {
    prop_oneof![
        any::<i32>().prop_map(Presence::Some),
        Just(Presence::Some(0)),
        Just(Presence::Null),
        Just(Presence::Absent),
    ]
}

fn nullish_presence() -> impl Strategy<Value = Presence<i32>> {
    prop_oneof![Just(Presence::Null), Just(Presence::Absent)]
}

/// The state of a presence with its value erased, so states of different types compare.
fn state<T>(presence: &Presence<T>) -> Presence<()> {
    presence.as_ref().map(|_| ())
}

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
    assert_eq!(null, Vec::<&i32>::new());
    assert_eq!(absent, Vec::<&i32>::new());
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
    #![proptest_config(persisted_config())]

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

// Spec 03 — self decides and first Some wins

proptest! {
    #![proptest_config(persisted_config())]

    #[test]
    fn a_null_or_absent_receiver_passes_through_the_self_decides_combinators_unchanged(
        p in nullish_presence(),
        q in any_presence(),
    ) {
        // Given any presence p that is Null or Absent
        // And any presence q and any function f
        let expected = state(&p);

        // When p.map(f), p.and(q), p.and_then(f) and p.filter(f) are evaluated
        let mapped = p.map(|x| i64::from(x) * 2);
        let anded = p.and(q);
        let and_thened = p.and_then(|x| Presence::Some(x.to_string()));
        let filtered = p.filter(|_| true);

        // Then each result is in the same state as p
        prop_assert_eq!(state(&mapped), expected);
        prop_assert_eq!(state(&anded), expected);
        prop_assert_eq!(state(&and_thened), expected);
        prop_assert_eq!(state(&filtered), expected);
        // And an outer Null or Absent of type Presence<Presence<T>> flattens to the same state
        let nested: Presence<Presence<i32>> = p.map(|_| q);
        prop_assert_eq!(state(&nested.flatten()), expected);
        // (copied, cloned, transpose and unzip follow the same rule)
        prop_assert_eq!(state(&p.as_ref().copied()), expected);
        prop_assert_eq!(state(&p.as_ref().cloned()), expected);
        let transposed = p.map(Ok::<i32, ()>).transpose();
        prop_assert_eq!(transposed.map(|t| state(&t)), Ok(expected));
        let (left, right) = p.map(|x| (x, x)).unzip();
        prop_assert_eq!((state(&left), state(&right)), (expected, expected));
    }

    #[test]
    fn and_on_some_returns_the_argument_unchanged(x in any::<i32>(), q in any_presence()) {
        // Given p = Some(x) for any x and any presence q
        let p = Presence::Some(x);

        // When p.and(q) is evaluated
        let result = p.and(q);

        // Then the result equals q
        prop_assert_eq!(result, q);
    }

    #[test]
    fn or_returns_self_when_some_otherwise_the_argument_unchanged(
        p in any_presence(),
        q in any_presence(),
    ) {
        // Given any presences p and q
        // When p.or(q) is evaluated
        let result = p.or(q);

        // Then the result is p if p is Some, otherwise exactly q
        let expected = if p.is_present() { p } else { q };
        prop_assert_eq!(result, expected);
        prop_assert_eq!(p.or_else(|| q), expected);
        // And Null.or(Absent) is Absent and Absent.or(Null) is Null
        prop_assert_eq!(Presence::<i32>::Null.or(Presence::Absent), Presence::Absent);
        prop_assert_eq!(Presence::<i32>::Absent.or(Presence::Null), Presence::Null);
    }
}

// Spec 03 — Absent over Null over Some

/// The precedence rule written out once: Absent if any input is Absent, otherwise
/// Null if any is Null, otherwise Some.
fn precedence_state(states: &[Presence<()>]) -> Presence<()> {
    if states.contains(&Presence::Absent) {
        Presence::Absent
    } else if states.contains(&Presence::Null) {
        Presence::Null
    } else {
        Presence::Some(())
    }
}

fn small_presence() -> impl Strategy<Value = Presence<i64>> {
    prop_oneof![
        (-9_i64..=9).prop_map(Presence::Some),
        Just(Presence::Null),
        Just(Presence::Absent),
    ]
}

proptest! {
    #![proptest_config(persisted_config())]

    #[test]
    fn zip_gives_absent_precedence_over_null_and_null_over_some(
        p in any_presence(),
        q in any_presence(),
    ) {
        // Given any presences p and q
        // When p.zip(q) is evaluated
        let zipped = p.zip(q);

        // Then the result is Some((a, b)) if both are Some
        if let (Presence::Some(a), Presence::Some(b)) = (p, q) {
            prop_assert_eq!(zipped, Presence::Some((a, b)));
        }
        // And otherwise Absent if either is Absent
        // And otherwise Null
        prop_assert_eq!(state(&zipped), precedence_state(&[state(&p), state(&q)]));
        prop_assert_eq!(state(&p.zip_with(q, i32::wrapping_add)), state(&zipped));
        // And swapping p and q swaps the tuple but not the state
        prop_assert_eq!(q.zip(p), zipped.map(|(a, b)| (b, a)));
        // (zip and collect share one definition of the rule)
        let collected: Presence<Vec<i32>> = [p, q].into_iter().collect();
        prop_assert_eq!(state(&collected), state(&zipped));
    }

    #[test]
    fn collect_sum_and_product_use_the_same_precedence_as_zip(
        items in proptest::collection::vec(small_presence(), 0..6),
    ) {
        // Given any list of presences
        let states: Vec<Presence<()>> = items.iter().map(state).collect();
        let values: Vec<i64> = items.iter().filter_map(|p| p.to_optional()).collect();

        // When it is collected into Presence<Vec<_>>, summed and multiplied
        let collected: Presence<Vec<i64>> = items.iter().copied().collect();
        let sum: Presence<i64> = items.iter().copied().sum();
        let product: Presence<i64> = items.iter().copied().product();

        // Then each result is Absent if any element is Absent
        // And otherwise Null if any element is Null
        // And otherwise Some of the collected values, sum and product
        let expected = precedence_state(&states);
        prop_assert_eq!(collected, expected.map(|()| values.clone()));
        prop_assert_eq!(sum, expected.map(|()| values.iter().sum()));
        prop_assert_eq!(product, expected.map(|()| values.iter().product()));
    }
}

// Spec 03 — Absent results of xor, filter and take_if

proptest! {
    #![proptest_config(persisted_config())]

    #[test]
    fn xor_returns_the_single_some_null_for_two_nulls_and_absent_otherwise(
        p in any_presence(),
        q in any_presence(),
    ) {
        // Given any presences p and q
        // When p.xor(q) is evaluated
        let result = p.xor(q);

        // Then the result is the Some side if exactly one side is Some
        // And Null if both are Null
        // And Absent in every other case, including Some xor Some
        let expected = match (p, q) {
            (Presence::Some(_), Presence::Null | Presence::Absent) => p,
            (Presence::Null | Presence::Absent, Presence::Some(_)) => q,
            (Presence::Null, Presence::Null) => Presence::Null,
            _ => Presence::Absent,
        };
        prop_assert_eq!(result, expected);
    }
}

#[test]
fn filter_turns_a_failing_some_into_absent() {
    // Given Some(3) and a predicate that rejects it
    let presence = Presence::Some(3);

    // When filter is applied
    let result = presence.filter(|x| x % 2 == 0);

    // Then the result is Absent, not Null
    assert_eq!(result, Presence::Absent);
}

#[test]
fn take_if_that_takes_nothing_returns_absent_and_leaves_the_presence_unchanged() {
    // Given Some(10), Null and Absent, and a predicate that rejects 10
    let originals = [Presence::Some(10), Presence::Null, Presence::Absent];

    for original in originals {
        let mut presence = original;

        // When take_if is called on each
        let taken = presence.take_if(|x| *x == 42);

        // Then each call returns Absent
        assert_eq!(taken, Presence::Absent);
        // And each presence keeps its original state
        assert_eq!(presence, original);
    }
}

// Spec 03 — owning iterator is IntoIter

#[test]
fn into_iter_returns_presence_into_iter() {
    // Given a Presence::Some(1)
    let presence = Presence::Some(1);

    // When it is converted with into_iter
    let iter: presence_rs::presence::IntoIter<i32> = presence.into_iter();

    // Then the iterator has type presence_rs::presence::IntoIter<i32>
    // And it yields 1 once
    assert_eq!(iter.collect::<Vec<_>>(), vec![1]);
}

// Spec 03 — is_nullish_or and deprecated names

proptest! {
    #![proptest_config(persisted_config())]

    #[test]
    fn is_nullish_or_is_true_for_null_and_absent_and_tests_the_value_of_some(
        p in any_presence(),
        threshold in any::<i32>(),
    ) {
        // Given any presence p and a predicate f
        let f = |x: i32| x > threshold;

        // When p.is_nullish_or(f) is evaluated
        let result = p.is_nullish_or(f);

        // Then it is true for Null and Absent
        // And it equals f(x) for Some(x)
        let expected = match p {
            Presence::Some(x) => f(x),
            Presence::Null | Presence::Absent => true,
        };
        prop_assert_eq!(result, expected);
    }

    #[test]
    #[expect(deprecated, reason = "this scenario pins the deprecated names to their replacements")]
    fn deprecated_names_still_work_and_match_their_replacements(
        p in any_presence(),
        q in any_presence(),
        threshold in any::<i32>(),
    ) {
        // Given any presence p
        // When to_nested_option, reduce, is_null_or and the Item type are used with
        // deprecation allowed
        // Then to_nested_option equals to_nullable
        prop_assert_eq!(p.to_nested_option(), p.to_nullable());
        // And reduce equals zip_with
        prop_assert_eq!(p.reduce(q, i32::wrapping_sub), p.zip_with(q, i32::wrapping_sub));
        // And is_null_or equals is_nullish_or
        let f = |x: i32| x > threshold;
        prop_assert_eq!(p.is_null_or(f), p.is_nullish_or(f));
        // And a value of type Item<T> is the iterator into_iter returns
        let iter: presence_rs::presence::Item<i32> = p.into_iter();
        prop_assert_eq!(iter.collect::<Vec<_>>(), p.to_optional().into_iter().collect::<Vec<_>>());
    }
}

// Spec 03 — ordering and From<T>

fn rank(presence: Presence<i32>) -> u8 {
    match presence {
        Presence::Absent => 0,
        Presence::Null => 1,
        Presence::Some(_) => 2,
    }
}

proptest! {
    #![proptest_config(persisted_config())]

    #[test]
    fn presences_are_ordered_absent_then_null_then_some_by_value(
        p in any_presence(),
        q in any_presence(),
    ) {
        // Given any presences p and q
        // When they are compared
        let ordering = p.cmp(&q);

        // Then Absent < Null < Some(x) for every x
        // And Some(a).cmp(&Some(b)) equals a.cmp(&b)
        let expected = match (p, q) {
            (Presence::Some(a), Presence::Some(b)) => a.cmp(&b),
            _ => rank(p).cmp(&rank(q)),
        };
        prop_assert_eq!(ordering, expected);
        prop_assert_eq!(p.partial_cmp(&q), Some(expected));
    }
}

#[test]
fn converting_an_option_into_a_presence_of_option_wraps_it() {
    // Given None of type Option<i32>
    let option: Option<i32> = None;

    // When it is converted with into() into Presence<Option<i32>>
    let presence: Presence<Option<i32>> = option.into();

    // Then the result is Some(None), not Null or Absent
    assert_eq!(presence, Presence::Some(None));
}
