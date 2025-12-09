use presence_rs::presence::Presence;

#[test]
fn test_is_some_and() {
    let some = Presence::Some(5);
    assert!(some.clone().is_some_and(|x| x > 3));
    assert!(!some.is_some_and(|x| x > 10));

    let null: Presence<i32> = Presence::Null;
    assert!(!null.is_some_and(|x| x > 3));

    let absent: Presence<i32> = Presence::Absent;
    assert!(!absent.is_some_and(|x| x > 3));
}

#[test]
fn test_is_absent_or() {
    let some = Presence::Some(5);
    // Returns true if Some(val) AND predicate matches
    assert!(some.clone().is_absent_or(|x| x > 3));
    assert!(!some.is_absent_or(|x| x > 10));

    let null: Presence<i32> = Presence::Null;
    // Returns false for Null
    assert!(!null.is_absent_or(|x| x > 3));

    let absent: Presence<i32> = Presence::Absent;
    // Returns true for Absent
    assert!(absent.is_absent_or(|x| x > 3));
}

#[test]
fn test_is_null_or() {
    let some = Presence::Some(5);
    // Returns true if Some(val) AND predicate matches
    assert!(some.clone().is_null_or(|x| x > 3));
    assert!(!some.is_null_or(|x| x > 10));

    let null: Presence<i32> = Presence::Null;
    // Returns true for Null
    assert!(null.is_null_or(|x| x > 3));

    let absent: Presence<i32> = Presence::Absent;
    // Returns true for Absent as well
    assert!(absent.is_null_or(|x| x > 3));
}

#[test]
fn test_get_or_insert() {
    let mut some = Presence::Some(5);
    assert_eq!(*some.get_or_insert(10), 5);
    assert_eq!(some, Presence::Some(5));

    let mut null: Presence<i32> = Presence::Null;
    assert_eq!(*null.get_or_insert(10), 10);
    assert_eq!(null, Presence::Some(10));

    let mut absent: Presence<i32> = Presence::Absent;
    assert_eq!(*absent.get_or_insert(10), 10);
    assert_eq!(absent, Presence::Some(10));
}

#[test]
fn test_get_or_insert_with() {
    let mut some = Presence::Some(5);
    assert_eq!(*some.get_or_insert_with(|| 10), 5);
    assert_eq!(some, Presence::Some(5));

    let mut null: Presence<i32> = Presence::Null;
    assert_eq!(*null.get_or_insert_with(|| 10), 10);
    assert_eq!(null, Presence::Some(10));

    let mut absent: Presence<i32> = Presence::Absent;
    assert_eq!(*absent.get_or_insert_with(|| 10), 10);
    assert_eq!(absent, Presence::Some(10));
}

#[test]
fn test_take() {
    let mut some = Presence::Some(5);
    assert_eq!(some.take(), Presence::Some(5));
    assert_eq!(some, Presence::Absent);

    let mut null: Presence<i32> = Presence::Null;
    assert_eq!(null.take(), Presence::Null);
    assert_eq!(null, Presence::Absent);

    let mut absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.take(), Presence::Absent);
    assert_eq!(absent, Presence::Absent);
}

#[test]
fn test_replace() {
    let mut some = Presence::Some(5);
    assert_eq!(some.replace(10), Presence::Some(5));
    assert_eq!(some, Presence::Some(10));

    let mut null: Presence<i32> = Presence::Null;
    assert_eq!(null.replace(10), Presence::Null);
    assert_eq!(null, Presence::Some(10));

    let mut absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.replace(10), Presence::Absent);
    assert_eq!(absent, Presence::Some(10));
}

#[test]
fn test_insert() {
    let mut some = Presence::Some(5);
    assert_eq!(*some.insert(10), 10);
    assert_eq!(some, Presence::Some(10));

    let mut null: Presence<i32> = Presence::Null;
    assert_eq!(*null.insert(10), 10);
    assert_eq!(null, Presence::Some(10));

    let mut absent: Presence<i32> = Presence::Absent;
    assert_eq!(*absent.insert(10), 10);
    assert_eq!(absent, Presence::Some(10));
}

#[test]
fn test_copied() {
    let some = Presence::Some(&5);
    assert_eq!(some.copied(), Presence::Some(5));

    let null: Presence<&i32> = Presence::Null;
    assert_eq!(null.copied(), Presence::Null);

    let absent: Presence<&i32> = Presence::Absent;
    assert_eq!(absent.copied(), Presence::Absent);
}

#[test]
fn test_cloned() {
    let some = Presence::Some(&String::from("hello"));
    assert_eq!(some.cloned(), Presence::Some(String::from("hello")));

    let null: Presence<&String> = Presence::Null;
    assert_eq!(null.cloned(), Presence::Null);

    let absent: Presence<&String> = Presence::Absent;
    assert_eq!(absent.cloned(), Presence::Absent);
}

#[test]
fn test_inspect() {
    let mut called = false;
    let some = Presence::Some(5);
    some.inspect(|&x| {
        called = true;
        assert_eq!(x, 5);
    });
    assert!(called);

    called = false;
    let null: Presence<i32> = Presence::Null;
    null.inspect(|_| {
        called = true;
    });
    assert!(!called);

    called = false;
    let absent: Presence<i32> = Presence::Absent;
    absent.inspect(|_| {
        called = true;
    });
    assert!(!called);
}
