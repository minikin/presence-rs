use presence_rs::Presence;

#[test]
fn test_into_iter() {
    let some = Presence::Some(42);
    let mut iter = some.into_iter();
    assert_eq!(iter.next(), Some(42));
    assert_eq!(iter.next(), None);

    let null: Presence<i32> = Presence::Null;
    let mut iter = null.into_iter();
    assert_eq!(iter.next(), None);

    let absent: Presence<i32> = Presence::Absent;
    let mut iter = absent.into_iter();
    assert_eq!(iter.next(), None);
}

#[test]
fn test_iter() {
    let some = Presence::Some(42);
    let mut iter = some.iter();
    assert_eq!(iter.next(), Some(&42));
    assert_eq!(iter.next(), None);

    let null: Presence<i32> = Presence::Null;
    let mut iter = null.iter();
    assert_eq!(iter.next(), None);

    let absent: Presence<i32> = Presence::Absent;
    let mut iter = absent.iter();
    assert_eq!(iter.next(), None);
}

#[test]
fn test_iter_mut() {
    let mut some = Presence::Some(42);
    for val in some.iter_mut() {
        *val *= 2;
    }
    assert_eq!(some, Presence::Some(84));

    let mut null: Presence<i32> = Presence::Null;
    let mut count = 0;
    for _ in null.iter_mut() {
        count += 1;
    }
    assert_eq!(count, 0);
}

#[test]
fn test_exact_size_iterator() {
    let some = Presence::Some(42);
    let iter = some.iter();
    assert_eq!(iter.len(), 1);

    let null: Presence<i32> = Presence::Null;
    let iter = null.iter();
    assert_eq!(iter.len(), 0);

    let absent: Presence<i32> = Presence::Absent;
    let iter = absent.iter();
    assert_eq!(iter.len(), 0);
}

#[test]
fn test_double_ended_iterator() {
    let some = Presence::Some(42);
    let mut iter = some.iter();
    assert_eq!(iter.next_back(), Some(&42));
    assert_eq!(iter.next_back(), None);

    let null: Presence<i32> = Presence::Null;
    let mut iter = null.iter();
    assert_eq!(iter.next_back(), None);
}

#[test]
fn test_collect() {
    let some = Presence::Some(42);
    let collected: Vec<i32> = some.into_iter().collect();
    assert_eq!(collected, vec![42]);

    let null: Presence<i32> = Presence::Null;
    let collected: Vec<i32> = null.into_iter().collect();
    assert_eq!(collected, Vec::<i32>::new());

    let absent: Presence<i32> = Presence::Absent;
    let collected: Vec<i32> = absent.into_iter().collect();
    assert_eq!(collected, Vec::<i32>::new());
}

#[test]
fn test_for_loop() {
    let some = Presence::Some(42);
    let mut values = Vec::new();
    for val in some {
        values.push(val);
    }
    assert_eq!(values, vec![42]);

    let null: Presence<i32> = Presence::Null;
    let mut values = Vec::new();
    for val in null {
        values.push(val);
    }
    assert_eq!(values, Vec::<i32>::new());
}

#[test]
fn test_for_loop_with_references() {
    let some = Presence::Some(42);
    let mut values = Vec::new();
    for val in some.iter() {
        values.push(*val);
    }
    assert_eq!(values, vec![42]);
}

#[test]
fn test_iterator_chain() {
    let presence1 = Presence::Some(1);
    let presence2 = Presence::Some(2);
    let presence3: Presence<i32> = Presence::Null;

    let collected: Vec<i32> = presence1
        .into_iter()
        .chain(presence2)
        .chain(presence3)
        .collect();

    assert_eq!(collected, vec![1, 2]);
}

#[test]
fn test_iterator_map() {
    let some = Presence::Some(5);
    let doubled: Vec<i32> = some.into_iter().map(|x| x * 2).collect();
    assert_eq!(doubled, vec![10]);
}

#[test]
fn test_iterator_filter() {
    let some1 = Presence::Some(5);
    let filtered1: Vec<i32> = some1.into_iter().filter(|&x| x > 3).collect();
    assert_eq!(filtered1, vec![5]);

    let some2 = Presence::Some(5);
    let filtered2: Vec<i32> = some2.into_iter().filter(|&x| x > 10).collect();
    assert_eq!(filtered2, Vec::<i32>::new());
}

#[test]
fn test_from_iterator() {
    let values = vec![Presence::Some(1), Presence::Some(2), Presence::Null];
    let collected: Vec<i32> = values.into_iter().flatten().collect();
    assert_eq!(collected, vec![1, 2]);
}

#[test]
fn test_size_hint() {
    let some = Presence::Some(42);
    let iter = some.iter();
    assert_eq!(iter.size_hint(), (1, Some(1)));

    let null: Presence<i32> = Presence::Null;
    let iter = null.iter();
    assert_eq!(iter.size_hint(), (0, Some(0)));

    let absent: Presence<i32> = Presence::Absent;
    let iter = absent.iter();
    assert_eq!(iter.size_hint(), (0, Some(0)));
}
