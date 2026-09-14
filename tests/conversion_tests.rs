use presence_rs::Presence;

#[test]
fn test_to_optional() {
    let some = Presence::Some(42);
    assert_eq!(some.to_optional(), Some(42));

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.to_optional(), None);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.to_optional(), None);
}

#[test]
fn test_to_nullable() {
    let some = Presence::Some(42);
    assert_eq!(some.to_nullable(), Some(Some(42)));

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.to_nullable(), Some(None));

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.to_nullable(), None);
}

#[test]
fn test_from_optional() {
    let some_opt = Some(42);
    let presence = Presence::from_optional(some_opt);
    assert_eq!(presence, Presence::Some(42));

    let none_opt: Option<i32> = None;
    let presence = Presence::from_optional(none_opt);
    assert_eq!(presence, Presence::Absent);
}

#[test]
fn test_from_nullable() {
    let some_some: Option<Option<i32>> = Some(Some(42));
    let presence = Presence::from_nullable(some_some);
    assert_eq!(presence, Presence::Some(42));

    let some_none: Option<Option<i32>> = Some(None);
    let presence = Presence::from_nullable(some_none);
    assert_eq!(presence, Presence::Null);

    let none: Option<Option<i32>> = None;
    let presence = Presence::from_nullable(none);
    assert_eq!(presence, Presence::Absent);
}

#[test]
fn test_from_option_option_trait() {
    let some_some: Option<Option<i32>> = Some(Some(42));
    let presence: Presence<i32> = some_some.into();
    assert_eq!(presence, Presence::Some(42));

    let some_none: Option<Option<i32>> = Some(None);
    let presence: Presence<i32> = some_none.into();
    assert_eq!(presence, Presence::Null);

    let none: Option<Option<i32>> = None;
    let presence: Presence<i32> = none.into();
    assert_eq!(presence, Presence::Absent);
}

#[test]
fn test_from_value_trait() {
    let value = 42;
    let presence: Presence<i32> = value.into();
    assert_eq!(presence, Presence::Some(42));

    let string_value = String::from("hello");
    let presence: Presence<String> = string_value.into();
    assert_eq!(presence, Presence::Some(String::from("hello")));
}

#[test]
fn test_as_ref() {
    let some = Presence::Some(42);
    let ref_presence = some.as_ref();
    assert_eq!(ref_presence, Presence::Some(&42));

    let null: Presence<i32> = Presence::Null;
    let ref_presence = null.as_ref();
    assert_eq!(ref_presence, Presence::Null);

    let absent: Presence<i32> = Presence::Absent;
    let ref_presence = absent.as_ref();
    assert_eq!(ref_presence, Presence::Absent);
}

#[test]
fn test_as_mut() {
    let mut some = Presence::Some(42);
    if let Presence::Some(val) = some.as_mut() {
        *val = 100;
    }
    assert_eq!(some, Presence::Some(100));

    let mut null: Presence<i32> = Presence::Null;
    let mut_presence = null.as_mut();
    assert_eq!(mut_presence, Presence::Null);
}

#[test]
fn test_as_slice() {
    let some = Presence::Some(42);
    assert_eq!(some.as_slice(), &[42]);

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.as_slice(), &[]);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.as_slice(), &[]);
}

#[test]
fn test_as_mut_slice() {
    let mut some = Presence::Some(42);
    let slice = some.as_mut_slice();
    slice[0] = 100;
    assert_eq!(some, Presence::Some(100));

    let mut null: Presence<i32> = Presence::Null;
    assert_eq!(null.as_mut_slice(), &mut []);
}

#[test]
fn test_round_trip_conversions() {
    // Test that converting back and forth preserves state
    let original = Presence::Some(42);
    let nullable = original.to_nullable();
    let back = Presence::from_nullable(nullable);
    assert_eq!(original, back);

    let original: Presence<i32> = Presence::Null;
    let nullable = original.to_nullable();
    let back = Presence::from_nullable(nullable);
    assert_eq!(original, back);

    let original: Presence<i32> = Presence::Absent;
    let nullable = original.to_nullable();
    let back = Presence::from_nullable(nullable);
    assert_eq!(original, back);
}

#[test]
fn test_to_nested_option() {
    let some = Presence::Some(42);
    assert_eq!(some.to_nested_option(), Some(Some(42)));

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.to_nested_option(), Some(None));

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.to_nested_option(), None);
}

#[test]
fn test_into_option_option_trait() {
    let some: Option<Option<i32>> = Presence::Some(42).into();
    assert_eq!(some, Some(Some(42)));

    let null: Option<Option<i32>> = Presence::<i32>::Null.into();
    assert_eq!(null, Some(None));

    let absent: Option<Option<i32>> = Presence::<i32>::Absent.into();
    assert_eq!(absent, None);
}

#[test]
fn test_ok_or() {
    assert_eq!(Presence::Some(42).ok_or("missing"), Ok(42));
    assert_eq!(Presence::<i32>::Null.ok_or("missing"), Err("missing"));
    assert_eq!(Presence::<i32>::Absent.ok_or("missing"), Err("missing"));
}

#[test]
fn test_ok_or_else() {
    assert_eq!(Presence::Some(42).ok_or_else(|| "missing"), Ok(42));
    assert_eq!(
        Presence::<i32>::Null.ok_or_else(|| "missing"),
        Err("missing")
    );
    assert_eq!(
        Presence::<i32>::Absent.ok_or_else(|| "missing"),
        Err("missing")
    );
}

#[test]
fn test_as_deref() {
    let some = Presence::Some(String::from("hello"));
    assert_eq!(some.as_deref(), Presence::Some("hello"));

    let null: Presence<String> = Presence::Null;
    assert_eq!(null.as_deref(), Presence::Null);

    let absent: Presence<String> = Presence::Absent;
    assert_eq!(absent.as_deref(), Presence::Absent);
}

#[test]
fn test_as_deref_mut() {
    let mut some = Presence::Some(String::from("hello"));
    if let Presence::Some(s) = some.as_deref_mut() {
        s.make_ascii_uppercase();
    }
    assert_eq!(some, Presence::Some(String::from("HELLO")));

    let mut null: Presence<String> = Presence::Null;
    assert_eq!(null.as_deref_mut(), Presence::Null);

    let mut absent: Presence<String> = Presence::Absent;
    assert_eq!(absent.as_deref_mut(), Presence::Absent);
}

#[test]
fn test_as_pin_ref() {
    let some = Presence::Some(42);
    assert_eq!(
        std::pin::Pin::new(&some).as_pin_ref().map(|p| *p),
        Presence::Some(42)
    );

    let null: Presence<i32> = Presence::Null;
    assert_eq!(
        std::pin::Pin::new(&null).as_pin_ref().map(|p| *p),
        Presence::Null
    );

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(
        std::pin::Pin::new(&absent).as_pin_ref().map(|p| *p),
        Presence::Absent
    );
}

#[test]
fn test_as_pin_mut() {
    let mut some = Presence::Some(42);
    if let Presence::Some(mut val) = std::pin::Pin::new(&mut some).as_pin_mut() {
        *val = 100;
    }
    assert_eq!(some, Presence::Some(100));

    let mut null: Presence<i32> = Presence::Null;
    assert!(std::pin::Pin::new(&mut null).as_pin_mut().is_null());

    let mut absent: Presence<i32> = Presence::Absent;
    assert!(std::pin::Pin::new(&mut absent).as_pin_mut().is_absent());
}
