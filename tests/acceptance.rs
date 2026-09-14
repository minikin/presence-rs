//! Acceptance tests: one test per spec scenario, named after the scenario.

use presence_rs::Presence;

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
