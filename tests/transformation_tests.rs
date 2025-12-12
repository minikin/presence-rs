use presence_rs::presence::Presence;

#[test]
fn test_map() {
    let some = Presence::Some(5);
    assert_eq!(some.map(|x| x * 2), Presence::Some(10));

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.map(|x| x * 2), Presence::Null);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.map(|x| x * 2), Presence::Absent);
}

#[test]
fn test_map_defined() {
    let some = Presence::Some(5);
    assert_eq!(some.map_defined(|x| x * 2), Presence::Some(10));

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.map_defined(|x| x * 2), Presence::Null);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.map_defined(|x| x * 2), Presence::Absent);
}

#[test]
fn test_map_or() {
    let some = Presence::Some(5);
    assert_eq!(some.map_or(42, |x| x * 2), 10);

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.map_or(42, |x| x * 2), 42);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.map_or(42, |x| x * 2), 42);
}

#[test]
fn test_map_or_else() {
    let some = Presence::Some(5);
    assert_eq!(some.map_or_else(|| 42, |x| x * 2), 10);

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.map_or_else(|| 42, |x| x * 2), 42);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.map_or_else(|| 42, |x| x * 2), 42);
}

#[test]
fn test_and() {
    let some = Presence::Some(5);
    assert_eq!(some.and(Presence::Some(10)), Presence::Some(10));
    assert_eq!(some.and(Presence::<i32>::Null), Presence::Null);
    assert_eq!(some.and(Presence::<i32>::Absent), Presence::Absent);

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.and(Presence::Some(10)), Presence::Null);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.and(Presence::Some(10)), Presence::Absent);
}

#[test]
fn test_and_then() {
    let some = Presence::Some(5);
    assert_eq!(some.and_then(|x| Presence::Some(x * 2)), Presence::Some(10));
    assert_eq!(some.and_then(|_| Presence::<i32>::Null), Presence::Null);

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.and_then(|x| Presence::Some(x * 2)), Presence::Null);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.and_then(|x| Presence::Some(x * 2)), Presence::Absent);
}

#[test]
fn test_filter() {
    let some = Presence::Some(5);
    assert_eq!(some.filter(|&x| x > 3), Presence::Some(5));
    assert_eq!(some.filter(|&x| x > 10), Presence::Absent);

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.filter(|&x| x > 3), Presence::Null);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.filter(|&x| x > 3), Presence::Absent);
}

#[test]
fn test_or() {
    let some = Presence::Some(5);
    assert_eq!(some.or(Presence::Some(10)), Presence::Some(5));
    assert_eq!(some.or(Presence::Null), Presence::Some(5));

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.or(Presence::Some(10)), Presence::Some(10));

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.or(Presence::Some(10)), Presence::Some(10));
}

#[test]
fn test_or_else() {
    let some = Presence::Some(5);
    assert_eq!(some.or_else(|| Presence::Some(10)), Presence::Some(5));

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.or_else(|| Presence::Some(10)), Presence::Some(10));

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.or_else(|| Presence::Some(10)), Presence::Some(10));
}

#[test]
fn test_xor() {
    let some1 = Presence::Some(5);
    let some2 = Presence::Some(10);
    let null: Presence<i32> = Presence::Null;
    let absent: Presence<i32> = Presence::Absent;

    assert_eq!(some1.xor(some2), Presence::Absent);
    assert_eq!(some1.xor(null), Presence::Some(5));
    assert_eq!(some1.xor(absent), Presence::Some(5));
    assert_eq!(null.xor(absent), Presence::Absent);
}

#[test]
fn test_unwrap() {
    let some = Presence::Some(42);
    assert_eq!(some.unwrap(), 42);
}

#[test]
#[should_panic(expected = "called `Presence::unwrap()` on")]
fn test_unwrap_on_null_panics() {
    let null: Presence<i32> = Presence::Null;
    null.unwrap();
}

#[test]
#[should_panic(expected = "called `Presence::unwrap()` on")]
fn test_unwrap_on_absent_panics() {
    let absent: Presence<i32> = Presence::Absent;
    absent.unwrap();
}

#[test]
fn test_unwrap_or() {
    let some = Presence::Some(42);
    assert_eq!(some.unwrap_or(100), 42);

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.unwrap_or(100), 100);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.unwrap_or(100), 100);
}

#[test]
fn test_unwrap_or_else() {
    let some = Presence::Some(42);
    assert_eq!(some.unwrap_or_else(|| 100), 42);

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.unwrap_or_else(|| 100), 100);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.unwrap_or_else(|| 100), 100);
}

#[test]
fn test_unwrap_or_default() {
    let some = Presence::Some(42);
    assert_eq!(some.unwrap_or_default(), 42);

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.unwrap_or_default(), 0);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.unwrap_or_default(), 0);
}

#[test]
fn test_unwrap_or_null_default() {
    let some = Presence::Some(42);
    assert_eq!(some.unwrap_or_null_default(10, 20), 42);

    let null: Presence<i32> = Presence::Null;
    assert_eq!(null.unwrap_or_null_default(10, 20), 20);

    let absent: Presence<i32> = Presence::Absent;
    assert_eq!(absent.unwrap_or_null_default(10, 20), 10);
}

#[test]
fn test_expect() {
    let some = Presence::Some(42);
    assert_eq!(some.expect("should have a value"), 42);
}

#[test]
#[should_panic(expected = "custom panic message")]
fn test_expect_on_null_panics() {
    let null: Presence<i32> = Presence::Null;
    null.expect("custom panic message");
}

#[test]
fn test_flatten() {
    let nested_some = Presence::Some(Presence::Some(42));
    assert_eq!(nested_some.flatten(), Presence::Some(42));

    let nested_null = Presence::Some(Presence::<i32>::Null);
    assert_eq!(nested_null.flatten(), Presence::Null);

    let nested_absent = Presence::Some(Presence::<i32>::Absent);
    assert_eq!(nested_absent.flatten(), Presence::Absent);

    let outer_null: Presence<Presence<i32>> = Presence::Null;
    assert_eq!(outer_null.flatten(), Presence::Null);

    let outer_absent: Presence<Presence<i32>> = Presence::Absent;
    assert_eq!(outer_absent.flatten(), Presence::Absent);
}

#[test]
fn test_zip() {
    let some1 = Presence::Some(5);
    let some2 = Presence::Some(10);
    assert_eq!(some1.zip(some2), Presence::Some((5, 10)));

    let some = Presence::Some(5);
    let null: Presence<i32> = Presence::Null;
    assert_eq!(some.zip(null), Presence::Null);

    let null1: Presence<i32> = Presence::Null;
    let null2: Presence<i32> = Presence::Null;
    assert_eq!(null1.zip(null2), Presence::Null);
}

#[test]
fn test_unzip() {
    let paired = Presence::Some((5, 10));
    let (first, second) = paired.unzip();
    assert_eq!(first, Presence::Some(5));
    assert_eq!(second, Presence::Some(10));

    let null: Presence<(i32, i32)> = Presence::Null;
    let (first, second) = null.unzip();
    assert_eq!(first, Presence::Null);
    assert_eq!(second, Presence::Null);

    let absent: Presence<(i32, i32)> = Presence::Absent;
    let (first, second) = absent.unzip();
    assert_eq!(first, Presence::Absent);
    assert_eq!(second, Presence::Absent);
}
