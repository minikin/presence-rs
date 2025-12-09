use presence_rs::presence::Presence;

#[test]
fn test_is_absent() {
    let absent: Presence<i32> = Presence::Absent;
    let null: Presence<i32> = Presence::Null;
    let some = Presence::Some(42);

    assert!(absent.is_absent());
    assert!(!null.is_absent());
    assert!(!some.is_absent());
}

#[test]
fn test_is_null() {
    let absent: Presence<i32> = Presence::Absent;
    let null: Presence<i32> = Presence::Null;
    let some = Presence::Some(42);

    assert!(!absent.is_null());
    assert!(null.is_null());
    assert!(!some.is_null());
}

#[test]
fn test_is_present() {
    let absent: Presence<i32> = Presence::Absent;
    let null: Presence<i32> = Presence::Null;
    let some = Presence::Some(42);

    assert!(!absent.is_present());
    assert!(!null.is_present());
    assert!(some.is_present());
}

#[test]
fn test_is_defined() {
    let absent: Presence<i32> = Presence::Absent;
    let null: Presence<i32> = Presence::Null;
    let some = Presence::Some(42);

    assert!(!absent.is_defined());
    assert!(null.is_defined());
    assert!(some.is_defined());
}

#[test]
fn test_is_nullish() {
    let absent: Presence<i32> = Presence::Absent;
    let null: Presence<i32> = Presence::Null;
    let some = Presence::Some(42);

    assert!(absent.is_nullish());
    assert!(null.is_nullish());
    assert!(!some.is_nullish());
}

#[test]
fn test_default() {
    let default: Presence<i32> = Presence::default();
    assert!(default.is_absent());
}

#[test]
fn test_equality() {
    assert_eq!(Presence::Some(42), Presence::Some(42));
    assert_ne!(Presence::Some(42), Presence::Some(43));
    assert_eq!(Presence::<i32>::Null, Presence::<i32>::Null);
    assert_eq!(Presence::<i32>::Absent, Presence::<i32>::Absent);
    assert_ne!(Presence::<i32>::Null, Presence::<i32>::Absent);
    assert_ne!(Presence::Some(42), Presence::<i32>::Null);
}

#[test]
fn test_clone() {
    let some = Presence::Some(String::from("hello"));
    let cloned = some.clone();
    assert_eq!(some, cloned);

    let null: Presence<String> = Presence::Null;
    let cloned_null = null.clone();
    assert_eq!(null, cloned_null);
}

#[test]
fn test_debug() {
    let some = Presence::Some(42);
    let debug_str = format!("{:?}", some);
    assert!(debug_str.contains("Some"));
    assert!(debug_str.contains("42"));

    let null: Presence<i32> = Presence::Null;
    let debug_str = format!("{:?}", null);
    assert!(debug_str.contains("Null"));

    let absent: Presence<i32> = Presence::Absent;
    let debug_str = format!("{:?}", absent);
    assert!(debug_str.contains("Absent"));
}
