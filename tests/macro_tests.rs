use presence_rs::presence;
use presence_rs::presence::Presence;

#[test]
fn test_presence_macro_absent() {
    let p: Presence<i32> = presence!();
    assert!(p.is_absent());
    assert_eq!(p, Presence::Absent);
}

#[test]
fn test_presence_macro_null() {
    let p: Presence<i32> = presence!(null);
    assert!(p.is_null());
    assert_eq!(p, Presence::Null);
}

#[test]
fn test_presence_macro_some_literal() {
    let p = presence!(42);
    assert!(p.is_present());
    assert_eq!(p, Presence::Some(42));
}

#[test]
fn test_presence_macro_some_string() {
    let p = presence!("hello");
    assert_eq!(p, Presence::Some("hello"));
}

#[test]
fn test_presence_macro_some_expression() {
    let p = presence!(2 + 2);
    assert_eq!(p, Presence::Some(4));
}

#[test]
fn test_presence_macro_some_owned_string() {
    let p = presence!("hello".to_string());
    assert_eq!(p, Presence::Some("hello".to_string()));
}

#[test]
fn test_presence_macro_some_vec() {
    let p = presence!(vec![1, 2, 3]);
    assert_eq!(p, Presence::Some(vec![1, 2, 3]));
}

#[test]
fn test_presence_macro_some_variable() {
    let value = 42;
    let p = presence!(value);
    assert_eq!(p, Presence::Some(42));
}

#[test]
fn test_presence_macro_some_function_call() {
    fn get_value() -> i32 {
        100
    }
    let p = presence!(get_value());
    assert_eq!(p, Presence::Some(100));
}

#[test]
fn test_presence_macro_some_struct() {
    #[derive(Debug, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    let p = presence!(Point { x: 10, y: 20 });
    assert_eq!(p, Presence::Some(Point { x: 10, y: 20 }));
}

#[test]
fn test_presence_macro_some_tuple() {
    let p = presence!((1, "hello", true));
    assert_eq!(p, Presence::Some((1, "hello", true)));
}

#[test]
fn test_presence_macro_in_match() {
    let p = presence!(42);
    match p {
        Presence::Some(val) => assert_eq!(val, 42),
        _ => panic!("Should be Some"),
    }

    let null: Presence<i32> = presence!(null);
    match null {
        Presence::Null => {}
        _ => panic!("Should be Null"),
    }

    let absent: Presence<i32> = presence!();
    match absent {
        Presence::Absent => {}
        _ => panic!("Should be Absent"),
    }
}

#[test]
fn test_presence_macro_type_inference() {
    // The type should be inferred from the value
    let p = presence!(42);
    let _: i32 = p.unwrap_or(0); // This proves the type is Presence<i32>

    let p = presence!("hello");
    let _: &str = p.unwrap_or(""); // This proves the type is Presence<&str>
}

#[test]
fn test_presence_macro_with_clone() {
    let original = String::from("hello");
    let p = presence!(original.clone());
    assert_eq!(p, Presence::Some(String::from("hello")));
    // original is still valid
    assert_eq!(original, "hello");
}

#[test]
fn test_presence_macro_nested() {
    let p = presence!(presence!(42));
    assert_eq!(p, Presence::Some(Presence::Some(42)));
}

#[test]
fn test_presence_macro_in_struct() {
    struct Container {
        value: Presence<i32>,
    }

    let c = Container {
        value: presence!(100),
    };
    assert_eq!(c.value, Presence::Some(100));

    let c2 = Container {
        value: presence!(null),
    };
    assert_eq!(c2.value, Presence::Null);

    let c3 = Container { value: presence!() };
    assert_eq!(c3.value, Presence::Absent);
}

#[test]
fn test_presence_macro_in_vec() {
    let v = [presence!(1), presence!(2), presence!(null), presence!()];
    assert_eq!(v[0], Presence::Some(1));
    assert_eq!(v[1], Presence::Some(2));
    assert_eq!(v[2], Presence::Null);
    assert_eq!(v[3], Presence::Absent);
}

#[test]
fn test_presence_macro_arithmetic() {
    let a = 10;
    let b = 5;
    let p = presence!(a * b + 2);
    assert_eq!(p, Presence::Some(52));
}

#[test]
fn test_presence_macro_method_chain() {
    let p = presence!("  hello  ".trim().to_uppercase());
    assert_eq!(p, Presence::Some("HELLO".to_string()));
}
