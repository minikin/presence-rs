/// Represents the presence/form of a value in schemas:
/// - Absent: field not in serialized data  {}
/// - Null:   field present but null       {"field": null}
/// - Some:   field present with value     {"field": value}
///
/// Cardinality for bool: 2 (base) + 1 (optional) + 1 (nullable) = 4 states
use std::{fmt, iter::FusedIterator};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Presence<T> {
    /// Field/key is absent from the structure
    Absent,
    /// Field/key is present but the value is null
    Null,
    /// Field/key is present with a concrete value
    Some(T),
}

/////////////////////////////////////////////////////////////////////////////
// Type implementation
/////////////////////////////////////////////////////////////////////////////
impl<T> Presence<T> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the presence is [`Absent`].
    ///
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<i32> = Presence::Absent;
    /// assert!(x.is_absent());
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert!(!y.is_absent());
    ///
    /// let z: Presence<i32> = Presence::Some(42);
    /// assert!(!z.is_absent());
    /// ```
    #[inline]
    pub const fn is_absent(&self) -> bool {
        matches!(self, Presence::Absent)
    }

    /// Returns `true` if the presence is [`Null`].
    ///
    /// [`Null`]: Presence::Null
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<i32> = Presence::Null;
    /// assert!(x.is_null());
    ///
    /// let y: Presence<i32> = Presence::Absent;
    /// assert!(!y.is_null());
    ///
    /// let z: Presence<i32> = Presence::Some(42);
    /// assert!(!z.is_null());
    /// ```
    #[inline]
    pub const fn is_null(&self) -> bool {
        matches!(self, Presence::Null)
    }

    /// Returns `true` if the presence is a [`Some`] value.
    ///
    /// [`Some`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<i32> = Presence::Some(42);
    /// assert!(x.is_present());
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert!(!y.is_present());
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert!(!z.is_present());
    /// ```
    #[inline]
    pub const fn is_present(&self) -> bool {
        matches!(self, Presence::Some(_))
    }

    /// Returns `true` if the presence is [`Some`] and the value inside of it matches a predicate.
    ///
    /// [`Some`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<u32> = Presence::Some(2);
    /// assert_eq!(x.is_some_and(|x| x > 1), true);
    ///
    /// let x: Presence<u32> = Presence::Some(0);
    /// assert_eq!(x.is_some_and(|x| x > 1), false);
    ///
    /// let x: Presence<u32> = Presence::Null;
    /// assert_eq!(x.is_some_and(|x| x > 1), false);
    ///
    /// let x: Presence<u32> = Presence::Absent;
    /// assert_eq!(x.is_some_and(|x| x > 1), false);
    /// ```
    #[inline]
    pub fn is_some_and(self, f: impl FnOnce(T) -> bool) -> bool {
        match self {
            Presence::Some(val) => f(val),
            Presence::Null | Presence::Absent => false,
        }
    }

    /// Returns `true` if the presence is [`Absent`] or the value inside matches a predicate.
    ///
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<u32> = Presence::Some(2);
    /// assert_eq!(x.is_absent_or(|x| x > 1), true);
    ///
    /// let x: Presence<u32> = Presence::Some(0);
    /// assert_eq!(x.is_absent_or(|x| x > 1), false);
    ///
    /// let x: Presence<u32> = Presence::Null;
    /// assert_eq!(x.is_absent_or(|x| x > 1), false);
    ///
    /// let x: Presence<u32> = Presence::Absent;
    /// assert_eq!(x.is_absent_or(|x| x > 1), true);
    /// ```
    #[inline]
    pub fn is_absent_or(self, f: impl FnOnce(T) -> bool) -> bool {
        match self {
            Presence::Some(val) => f(val),
            Presence::Null => false,
            Presence::Absent => true,
        }
    }

    /// Returns `true` if the presence is [`Null`] or [`Absent`], or the value inside matches a predicate.
    ///
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<u32> = Presence::Some(2);
    /// assert_eq!(x.is_null_or(|x| x > 1), true);
    ///
    /// let x: Presence<u32> = Presence::Some(0);
    /// assert_eq!(x.is_null_or(|x| x > 1), false);
    ///
    /// let x: Presence<u32> = Presence::Null;
    /// assert_eq!(x.is_null_or(|x| x > 1), true);
    ///
    /// let x: Presence<u32> = Presence::Absent;
    /// assert_eq!(x.is_null_or(|x| x > 1), true);
    /// ```
    #[inline]
    pub fn is_null_or(self, f: impl FnOnce(T) -> bool) -> bool {
        match self {
            Presence::Some(val) => f(val),
            Presence::Null | Presence::Absent => true,
        }
    }

    /// Converts from `&Presence<T>` to `Presence<&T>`.
    ///
    /// Produces a new `Presence`, containing a reference into the original, leaving
    /// the original in place.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some(42);
    /// assert_eq!(x.as_ref(), Presence::Some(&42));
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert_eq!(y.as_ref(), Presence::Null);
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert_eq!(z.as_ref(), Presence::Absent);
    /// ```
    #[inline]
    pub const fn as_ref(&self) -> Presence<&T> {
        match *self {
            Presence::Some(ref val) => Presence::Some(val),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Converts from `&mut Presence<T>` to `Presence<&mut T>`.
    ///
    /// Produces a new `Presence`, containing a mutable reference into the original,
    /// leaving the original in place.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let mut x = Presence::Some(42);
    /// match x.as_mut() {
    ///     Presence::Some(v) => *v = 100,
    ///     _ => {}
    /// }
    /// assert_eq!(x, Presence::Some(100));
    ///
    /// let mut y: Presence<i32> = Presence::Null;
    /// assert_eq!(y.as_mut(), Presence::Null);
    ///
    /// let mut z: Presence<i32> = Presence::Absent;
    /// assert_eq!(z.as_mut(), Presence::Absent);
    /// ```
    #[inline]
    pub const fn as_mut(&mut self) -> Presence<&mut T> {
        match *self {
            Presence::Some(ref mut val) => Presence::Some(val),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Converts from `Pin<&Presence<T>>` to `Presence<Pin<&T>>`.
    ///
    /// This is useful when you have a pinned presence and want to get a presence
    /// of pinned references to the inner value.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    /// use std::pin::Pin;
    ///
    /// let x = Presence::Some(42);
    /// let pinned = Pin::new(&x);
    /// let result = pinned.as_pin_ref();
    /// assert!(matches!(result, Presence::Some(_)));
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// let pinned = Pin::new(&y);
    /// assert_eq!(pinned.as_pin_ref(), Presence::Null);
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// let pinned = Pin::new(&z);
    /// assert_eq!(pinned.as_pin_ref(), Presence::Absent);
    /// ```
    #[inline]
    pub const fn as_pin_ref(self: std::pin::Pin<&Self>) -> Presence<std::pin::Pin<&T>> {
        match std::pin::Pin::get_ref(self) {
            Presence::Some(val) => unsafe {
                Presence::Some(std::pin::Pin::new_unchecked(val))
            },
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Converts from `Pin<&mut Presence<T>>` to `Presence<Pin<&mut T>>`.
    ///
    /// This is useful when you have a pinned mutable presence and want to get a presence
    /// of pinned mutable references to the inner value.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    /// use std::pin::Pin;
    ///
    /// let mut x = Presence::Some(42);
    /// let mut pinned = Pin::new(&mut x);
    /// match pinned.as_mut().as_pin_mut() {
    ///     Presence::Some(mut v) => {
    ///         *v = 100;
    ///     }
    ///     _ => {}
    /// }
    /// assert_eq!(x, Presence::Some(100));
    ///
    /// let mut y: Presence<i32> = Presence::Null;
    /// let mut pinned = Pin::new(&mut y);
    /// assert_eq!(pinned.as_mut().as_pin_mut(), Presence::Null);
    ///
    /// let mut z: Presence<i32> = Presence::Absent;
    /// let mut pinned = Pin::new(&mut z);
    /// assert_eq!(pinned.as_mut().as_pin_mut(), Presence::Absent);
    /// ```
    #[inline]
    pub const fn as_pin_mut(self: std::pin::Pin<&mut Self>) -> Presence<std::pin::Pin<&mut T>> {
        unsafe {
            match std::pin::Pin::get_unchecked_mut(self) {
                Presence::Some(val) => Presence::Some(std::pin::Pin::new_unchecked(val)),
                Presence::Null => Presence::Null,
                Presence::Absent => Presence::Absent,
            }
        }
    }

    /// Returns a slice containing the value if the presence is [`Some`].
    ///
    /// Returns an empty slice for [`Null`] or [`Absent`] variants.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some(42);
    /// assert_eq!(x.as_slice(), &[42]);
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert_eq!(y.as_slice(), &[]);
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert_eq!(z.as_slice(), &[]);
    /// ```
    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        match self {
            Presence::Some(val) => std::slice::from_ref(val),
            Presence::Null | Presence::Absent => &[],
        }
    }

    /// Returns a mutable slice containing the value if the presence is [`Some`].
    ///
    /// Returns an empty mutable slice for [`Null`] or [`Absent`] variants.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let mut x = Presence::Some(42);
    /// let slice = x.as_mut_slice();
    /// if let Some(first) = slice.first_mut() {
    ///     *first = 100;
    /// }
    /// assert_eq!(x, Presence::Some(100));
    ///
    /// let mut y: Presence<i32> = Presence::Null;
    /// assert_eq!(y.as_mut_slice(), &mut []);
    ///
    /// let mut z: Presence<i32> = Presence::Absent;
    /// assert_eq!(z.as_mut_slice(), &mut []);
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            Presence::Some(val) => std::slice::from_mut(val),
            Presence::Null | Presence::Absent => &mut [],
        }
    }

    /// Converts from `&Presence<T>` to `Presence<&T::Target>`.
    ///
    /// Leaves [`Null`] and [`Absent`] values unchanged.
    ///
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<String> = Presence::Some("hello".to_string());
    /// assert_eq!(x.as_deref(), Presence::Some("hello"));
    ///
    /// let y: Presence<String> = Presence::Null;
    /// assert_eq!(y.as_deref(), Presence::Null);
    ///
    /// let z: Presence<String> = Presence::Absent;
    /// assert_eq!(z.as_deref(), Presence::Absent);
    /// ```
    #[inline]
    pub fn as_deref(&self) -> Presence<&T::Target>
    where
        T: std::ops::Deref,
    {
        match self.as_ref() {
            Presence::Some(val) => Presence::Some(val.deref()),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Converts from `&mut Presence<T>` to `Presence<&mut T::Target>`.
    ///
    /// Leaves [`Null`] and [`Absent`] values unchanged.
    ///
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let mut x: Presence<String> = Presence::Some("hello".to_string());
    /// match x.as_deref_mut() {
    ///     Presence::Some(v) => v.make_ascii_uppercase(),
    ///     _ => {}
    /// }
    /// assert_eq!(x, Presence::Some("HELLO".to_string()));
    ///
    /// let mut y: Presence<String> = Presence::Null;
    /// assert_eq!(y.as_deref_mut(), Presence::Null);
    ///
    /// let mut z: Presence<String> = Presence::Absent;
    /// assert_eq!(z.as_deref_mut(), Presence::Absent);
    /// ```
    #[inline]
    pub fn as_deref_mut(&mut self) -> Presence<&mut T::Target>
    where
        T: std::ops::DerefMut,
    {
        match self.as_mut() {
            Presence::Some(val) => Presence::Some(val.deref_mut()),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Converts from `Presence<T>` to `Option<Option<T>>` for interoperability.
    ///
    /// This is useful when you need to work with code that uses nested `Option`s
    /// to represent the same three-state concept as `Presence`.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<i32> = Presence::Some(42);
    /// assert_eq!(x.to_nested_option(), Some(Some(42)));
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert_eq!(y.to_nested_option(), Some(None));
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert_eq!(z.to_nested_option(), None);
    /// ```
    #[inline]
    pub fn to_nested_option(self) -> Option<Option<T>> {
        match self {
            Presence::Absent => None,
            Presence::Null => Some(None),
            Presence::Some(val) => Some(Some(val)),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Getting to contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns the contained [`Some`] value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is [`Null`] or [`Absent`] with a custom panic message provided by `msg`.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some("value");
    /// assert_eq!(x.expect("should have a value"), "value");
    /// ```
    ///
    /// ```should_panic
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<&str> = Presence::Null;
    /// x.expect("the value was null"); // panics with `the value was null`
    /// ```
    ///
    /// ```should_panic
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<&str> = Presence::Absent;
    /// x.expect("the value was absent"); // panics with `the value was absent`
    /// ```
    #[inline]
    #[track_caller]
    pub fn expect(self, msg: &str) -> T {
        match self {
            Presence::Some(val) => val,
            Presence::Null => panic!("{}: value was Null", msg),
            Presence::Absent => panic!("{}: value was Absent", msg),
        }
    }

    /// Returns the contained [`Some`] value, consuming the `self` value.
    ///
    /// Because this function may panic, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the [`Null`] and [`Absent`]
    /// cases explicitly, or call [`unwrap_or`], [`unwrap_or_else`], or
    /// [`unwrap_or_default`].
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    /// [`unwrap_or`]: Presence::unwrap_or
    /// [`unwrap_or_else`]: Presence::unwrap_or_else
    /// [`unwrap_or_default`]: Presence::unwrap_or_default
    ///
    /// # Panics
    ///
    /// Panics if the value is [`Null`] or [`Absent`].
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some("air");
    /// assert_eq!(x.unwrap(), "air");
    /// ```
    ///
    /// ```should_panic
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<&str> = Presence::Null;
    /// x.unwrap(); // panics
    /// ```
    ///
    /// ```should_panic
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<&str> = Presence::Absent;
    /// x.unwrap(); // panics
    /// ```
    #[inline]
    #[track_caller]
    pub fn unwrap(self) -> T {
        match self {
            Presence::Some(val) => val,
            Presence::Null => panic!("called `Presence::unwrap()` on a `Null` value"),
            Presence::Absent => panic!("called `Presence::unwrap()` on an `Absent` value"),
        }
    }

    /// Returns the contained [`Some`] value or a provided default.
    ///
    /// Arguments passed to `unwrap_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`unwrap_or_else`],
    /// which is lazily evaluated.
    ///
    /// [`Some`]: Presence::Some
    /// [`unwrap_or_else`]: Presence::unwrap_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some("value");
    /// assert_eq!(x.unwrap_or("default"), "value");
    ///
    /// let y: Presence<&str> = Presence::Null;
    /// assert_eq!(y.unwrap_or("default"), "default");
    ///
    /// let z: Presence<&str> = Presence::Absent;
    /// assert_eq!(z.unwrap_or("default"), "default");
    /// ```
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Presence::Some(val) => val,
            Presence::Null | Presence::Absent => default,
        }
    }

    /// Returns the contained [`Some`] value or computes it from a closure.
    ///
    /// [`Some`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some(2);
    /// assert_eq!(x.unwrap_or_else(|| 10), 2);
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert_eq!(y.unwrap_or_else(|| 10), 10);
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert_eq!(z.unwrap_or_else(|| 10), 10);
    /// ```
    #[inline]
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Presence::Some(val) => val,
            Presence::Null | Presence::Absent => f(),
        }
    }

    /// Returns the contained [`Some`] value or a default.
    ///
    /// Consumes the `self` argument then, if [`Some`], returns the contained
    /// value, otherwise if [`Null`] or [`Absent`], returns the [default value] for that
    /// type.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    /// [default value]: Default::default
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<i32> = Presence::Some(42);
    /// assert_eq!(x.unwrap_or_default(), 42);
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert_eq!(y.unwrap_or_default(), 0);
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert_eq!(z.unwrap_or_default(), 0);
    /// ```
    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Presence::Some(val) => val,
            Presence::Null | Presence::Absent => Default::default(),
        }
    }

    /// Takes the value out of the `Presence`, leaving [`Absent`] in its place.
    ///
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let mut x = Presence::Some(42);
    /// let y = x.take();
    /// assert_eq!(x, Presence::Absent);
    /// assert_eq!(y, Presence::Some(42));
    ///
    /// let mut z: Presence<i32> = Presence::Null;
    /// let w = z.take();
    /// assert_eq!(z, Presence::Absent);
    /// assert_eq!(w, Presence::Null);
    /// ```
    #[inline]
    pub const fn take(&mut self) -> Presence<T> {
        let mut slot = Presence::Absent;
        std::mem::swap(self, &mut slot);
        slot
    }

    /// Inserts `value` into the presence, then returns a mutable reference to it.
    ///
    /// If the presence already contained a value, the old value is dropped.
    ///
    /// See also [`get_or_insert`], which doesn't update the value if
    /// the presence is [`Some`].
    ///
    /// [`Some`]: Presence::Some
    /// [`get_or_insert`]: Presence::get_or_insert
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let mut opt = Presence::Null;
    /// let val = opt.insert(1);
    /// assert_eq!(*val, 1);
    /// assert_eq!(opt.unwrap(), 1);
    ///
    /// let val = opt.insert(2);
    /// assert_eq!(*val, 2);
    /// *val = 3;
    /// assert_eq!(opt.unwrap(), 3);
    /// ```
    #[inline]
    pub fn insert(&mut self, value: T) -> &mut T {
        *self = Presence::Some(value);
        match self {
            Presence::Some(v) => v,
            _ => unreachable!(),
        }
    }

    /// Inserts `value` into the presence if it is [`Null`] or [`Absent`], then
    /// returns a mutable reference to the contained value.
    ///
    /// See also [`insert`], which updates the value even if
    /// the presence already contains [`Some`].
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    /// [`insert`]: Presence::insert
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let mut x = Presence::Null;
    ///
    /// {
    ///     let y: &mut u32 = x.get_or_insert(5);
    ///     assert_eq!(y, &5);
    ///
    ///     *y = 7;
    /// }
    ///
    /// assert_eq!(x, Presence::Some(7));
    /// ```
    #[inline]
    pub fn get_or_insert(&mut self, value: T) -> &mut T {
        if matches!(self, Presence::Null | Presence::Absent) {
            *self = Presence::Some(value);
        }
        match self {
            Presence::Some(v) => v,
            _ => unreachable!(),
        }
    }

    /// Inserts the default value into the presence if it is [`Null`] or [`Absent`], then
    /// returns a mutable reference to the contained value.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let mut x: Presence<u32> = Presence::Null;
    /// let y: &mut u32 = x.get_or_insert_default();
    /// assert_eq!(y, &0);
    ///
    /// let mut x = Presence::Some(10);
    /// let y: &mut u32 = x.get_or_insert_default();
    /// assert_eq!(y, &10);
    /// ```
    #[inline]
    pub fn get_or_insert_default(&mut self) -> &mut T
    where
        T: Default,
    {
        self.get_or_insert_with(Default::default)
    }

    /// Inserts a value computed from `f` into the presence if it is [`Null`] or [`Absent`],
    /// then returns a mutable reference to the contained value.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let mut x = Presence::Null;
    /// let y: &mut u32 = x.get_or_insert_with(|| 5);
    /// assert_eq!(y, &5);
    ///
    /// let mut x = Presence::Some(10);
    /// let y: &mut u32 = x.get_or_insert_with(|| 15);
    /// assert_eq!(y, &10);
    /// ```
    #[inline]
    pub fn get_or_insert_with<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        if matches!(self, Presence::Null | Presence::Absent) {
            *self = Presence::Some(f());
        }
        match self {
            Presence::Some(v) => v,
            _ => unreachable!(),
        }
    }

    /// Returns the number of elements in the `Presence`.
    ///
    /// This returns `1` if the presence contains a [`Some`] value, and `0` for
    /// [`Null`] or [`Absent`]. This is primarily used for iterator support.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<i32> = Presence::Some(42);
    /// assert_eq!(x.len(), 1);
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert_eq!(y.len(), 0);
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert_eq!(z.len(), 0);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        match self {
            Presence::Some(_) => 1,
            Presence::Null | Presence::Absent => 0,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Transforming contained values
    /////////////////////////////////////////////////////////////////////////

    /// Maps a `Presence<T>` to `Presence<U>` by applying a function to a contained value.
    ///
    /// Leaves [`Null`] and [`Absent`] values unchanged.
    ///
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some("hello");
    /// assert_eq!(x.map(|s| s.len()), Presence::Some(5));
    ///
    /// let y: Presence<&str> = Presence::Null;
    /// assert_eq!(y.map(|s| s.len()), Presence::Null);
    ///
    /// let z: Presence<&str> = Presence::Absent;
    /// assert_eq!(z.map(|s| s.len()), Presence::Absent);
    /// ```
    #[inline]
    pub fn map<U, F>(self, f: F) -> Presence<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Presence::Some(val) => Presence::Some(f(val)),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Calls the provided closure with the contained value (if [`Some`]).
    ///
    /// Returns the original presence unchanged.
    ///
    /// [`Some`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some(4)
    ///     .inspect(|x| println!("got: {}", x))
    ///     .map(|x| x * 2);
    /// assert_eq!(x, Presence::Some(8));
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// let result = y.inspect(|x| println!("got: {}", x));
    /// assert_eq!(result, Presence::Null);
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// let result = z.inspect(|x| println!("got: {}", x));
    /// assert_eq!(result, Presence::Absent);
    /// ```
    #[inline]
    pub fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Presence::Some(ref val) = self {
            f(val);
        }
        self
    }

    /// Returns the provided default result (if [`Null`] or [`Absent`]),
    /// or applies a function to the contained value (if [`Some`]).
    ///
    /// Arguments passed to `map_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`map_or_else`],
    /// which is lazily evaluated.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    /// [`map_or_else`]: Presence::map_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some("foo");
    /// assert_eq!(x.map_or(42, |v| v.len()), 3);
    ///
    /// let y: Presence<&str> = Presence::Null;
    /// assert_eq!(y.map_or(42, |v| v.len()), 42);
    ///
    /// let z: Presence<&str> = Presence::Absent;
    /// assert_eq!(z.map_or(42, |v| v.len()), 42);
    /// ```
    #[inline]
    pub fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Presence::Some(val) => f(val),
            Presence::Null | Presence::Absent => default,
        }
    }

    /// Computes a default function result (if [`Null`] or [`Absent`]),
    /// or applies a different function to the contained value (if [`Some`]).
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some("foo");
    /// assert_eq!(x.map_or_else(|| 42, |v| v.len()), 3);
    ///
    /// let y: Presence<&str> = Presence::Null;
    /// assert_eq!(y.map_or_else(|| 42, |v| v.len()), 42);
    ///
    /// let z: Presence<&str> = Presence::Absent;
    /// assert_eq!(z.map_or_else(|| 42, |v| v.len()), 42);
    /// ```
    #[inline]
    pub fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(T) -> U,
    {
        match self {
            Presence::Some(val) => f(val),
            Presence::Null | Presence::Absent => default(),
        }
    }

    /// Maps a `Presence<T>` to `U` by applying a function to a contained value,
    /// or returns the default value of `U` if [`Null`] or [`Absent`].
    ///
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some("foo");
    /// assert_eq!(x.map_or_default(|v| v.len()), 3);
    ///
    /// let y: Presence<&str> = Presence::Null;
    /// assert_eq!(y.map_or_default(|v| v.len()), 0);
    ///
    /// let z: Presence<&str> = Presence::Absent;
    /// assert_eq!(z.map_or_default(|v| v.len()), 0);
    /// ```
    #[inline]
    pub fn map_or_default<U, F>(self, f: F) -> U
    where
        F: FnOnce(T) -> U,
        U: Default,
    {
        match self {
            Presence::Some(val) => f(val),
            Presence::Null | Presence::Absent => Default::default(),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Result conversions
    /////////////////////////////////////////////////////////////////////////

    /// Transforms the `Presence<T>` into a [`Result<T, E>`], mapping [`Some(v)`] to
    /// [`Ok(v)`] and [`Null`] or [`Absent`] to [`Err(err)`].
    ///
    /// Arguments passed to `ok_or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`ok_or_else`], which is
    /// lazily evaluated.
    ///
    /// [`Some(v)`]: Presence::Some
    /// [`Ok(v)`]: Ok
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    /// [`Err(err)`]: Err
    /// [`ok_or_else`]: Presence::ok_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some("foo");
    /// assert_eq!(x.ok_or(0), Ok("foo"));
    ///
    /// let y: Presence<&str> = Presence::Null;
    /// assert_eq!(y.ok_or(0), Err(0));
    ///
    /// let z: Presence<&str> = Presence::Absent;
    /// assert_eq!(z.ok_or(0), Err(0));
    /// ```
    #[inline]
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Presence::Some(val) => Ok(val),
            Presence::Null | Presence::Absent => Err(err),
        }
    }

    /// Transforms the `Presence<T>` into a [`Result<T, E>`], mapping [`Some(v)`] to
    /// [`Ok(v)`] and [`Null`] or [`Absent`] to [`Err(err())`].
    ///
    /// [`Some(v)`]: Presence::Some
    /// [`Ok(v)`]: Ok
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    /// [`Err(err())`]: Err
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some("foo");
    /// assert_eq!(x.ok_or_else(|| 0), Ok("foo"));
    ///
    /// let y: Presence<&str> = Presence::Null;
    /// assert_eq!(y.ok_or_else(|| 0), Err(0));
    ///
    /// let z: Presence<&str> = Presence::Absent;
    /// assert_eq!(z.ok_or_else(|| 0), Err(0));
    /// ```
    #[inline]
    pub fn ok_or_else<E, F>(self, err: F) -> Result<T, E>
    where
        F: FnOnce() -> E,
    {
        match self {
            Presence::Some(val) => Ok(val),
            Presence::Null | Presence::Absent => Err(err()),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Boolean operations on the values, eager and lazy
    /////////////////////////////////////////////////////////////////////////

    /// Returns [`Absent`] or [`Null`] if the presence is [`Absent`] or [`Null`], otherwise returns `optb`.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some(2);
    /// let y: Presence<&str> = Presence::Null;
    /// assert_eq!(x.and(y), Presence::Null);
    ///
    /// let x: Presence<u32> = Presence::Null;
    /// let y = Presence::Some("foo");
    /// assert_eq!(x.and(y), Presence::Null);
    ///
    /// let x = Presence::Some(2);
    /// let y = Presence::Some("foo");
    /// assert_eq!(x.and(y), Presence::Some("foo"));
    ///
    /// let x: Presence<u32> = Presence::Absent;
    /// let y = Presence::Some("foo");
    /// assert_eq!(x.and(y), Presence::Absent);
    /// ```
    #[inline]
    pub fn and<U>(self, optb: Presence<U>) -> Presence<U> {
        match self {
            Presence::Some(_) => optb,
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Returns [`Absent`] or [`Null`] if the presence is [`Absent`] or [`Null`], otherwise calls `f` with the
    /// wrapped value and returns the result.
    ///
    /// Some languages call this operation flatmap.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// fn sq_then_to_string(x: u32) -> Presence<String> {
    ///     Presence::Some((x * x).to_string())
    /// }
    ///
    /// assert_eq!(Presence::Some(2).and_then(sq_then_to_string), Presence::Some(4.to_string()));
    /// assert_eq!(Presence::Null.and_then(sq_then_to_string), Presence::Null);
    /// assert_eq!(Presence::Absent.and_then(sq_then_to_string), Presence::Absent);
    /// ```
    #[inline]
    pub fn and_then<U, F>(self, f: F) -> Presence<U>
    where
        F: FnOnce(T) -> Presence<U>,
    {
        match self {
            Presence::Some(val) => f(val),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Returns [`Absent`] if the presence is [`Absent`], [`Null`] if the presence is [`Null`],
    /// and returns the presence unchanged if the predicate returns `true`, otherwise returns [`Absent`].
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// fn is_even(n: &i32) -> bool {
    ///     n % 2 == 0
    /// }
    ///
    /// assert_eq!(Presence::Some(4).filter(is_even), Presence::Some(4));
    /// assert_eq!(Presence::Some(3).filter(is_even), Presence::Absent);
    /// assert_eq!(Presence::Null.filter(is_even), Presence::Null);
    /// assert_eq!(Presence::Absent.filter(is_even), Presence::Absent);
    /// ```
    #[inline]
    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: FnOnce(&T) -> bool,
    {
        match self {
            Presence::Some(ref val) if predicate(val) => self,
            Presence::Some(_) => Presence::Absent,
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Returns the presence if it contains a value, otherwise returns `optb`.
    ///
    /// Arguments passed to `or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`or_else`], which is
    /// lazily evaluated.
    ///
    /// [`or_else`]: Presence::or_else
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some(2);
    /// let y = Presence::Null;
    /// assert_eq!(x.or(y), Presence::Some(2));
    ///
    /// let x = Presence::Null;
    /// let y = Presence::Some(100);
    /// assert_eq!(x.or(y), Presence::Some(100));
    ///
    /// let x = Presence::Some(2);
    /// let y = Presence::Some(100);
    /// assert_eq!(x.or(y), Presence::Some(2));
    ///
    /// let x: Presence<i32> = Presence::Null;
    /// let y = Presence::Null;
    /// assert_eq!(x.or(y), Presence::Null);
    ///
    /// let x: Presence<i32> = Presence::Absent;
    /// let y = Presence::Null;
    /// assert_eq!(x.or(y), Presence::Null);
    /// ```
    #[inline]
    pub fn or(self, optb: Presence<T>) -> Presence<T> {
        match self {
            Presence::Some(_) => self,
            Presence::Null | Presence::Absent => optb,
        }
    }

    /// Returns the presence if it contains a value, otherwise calls `f` and
    /// returns the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// fn nobody() -> Presence<&'static str> { Presence::Null }
    /// fn vikings() -> Presence<&'static str> { Presence::Some("vikings") }
    ///
    /// assert_eq!(Presence::Some("barbarians").or_else(vikings), Presence::Some("barbarians"));
    /// assert_eq!(Presence::Null.or_else(vikings), Presence::Some("vikings"));
    /// assert_eq!(Presence::Null.or_else(nobody), Presence::Null);
    /// assert_eq!(Presence::Absent.or_else(vikings), Presence::Some("vikings"));
    /// ```
    #[inline]
    pub fn or_else<F>(self, f: F) -> Presence<T>
    where
        F: FnOnce() -> Presence<T>,
    {
        match self {
            Presence::Some(_) => self,
            Presence::Null | Presence::Absent => f(),
        }
    }

    /// Returns [`Some`] if exactly one of `self`, `optb` is [`Some`], otherwise returns [`Absent`] or [`Null`].
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some(2);
    /// let y: Presence<i32> = Presence::Null;
    /// assert_eq!(x.xor(y), Presence::Some(2));
    ///
    /// let x: Presence<i32> = Presence::Null;
    /// let y = Presence::Some(2);
    /// assert_eq!(x.xor(y), Presence::Some(2));
    ///
    /// let x = Presence::Some(2);
    /// let y = Presence::Some(2);
    /// assert_eq!(x.xor(y), Presence::Absent);
    ///
    /// let x: Presence<i32> = Presence::Null;
    /// let y: Presence<i32> = Presence::Null;
    /// assert_eq!(x.xor(y), Presence::Null);
    ///
    /// let x: Presence<i32> = Presence::Absent;
    /// let y: Presence<i32> = Presence::Null;
    /// assert_eq!(x.xor(y), Presence::Absent);
    /// ```
    #[inline]
    pub fn xor(self, optb: Presence<T>) -> Presence<T> {
        match (self, optb) {
            (Presence::Some(a), Presence::Null | Presence::Absent) => Presence::Some(a),
            (Presence::Null | Presence::Absent, Presence::Some(b)) => Presence::Some(b),
            (Presence::Some(_), Presence::Some(_)) => Presence::Absent,
            (Presence::Absent, _) | (_, Presence::Absent) => Presence::Absent,
            (Presence::Null, Presence::Null) => Presence::Null,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Iterator constructors
    /////////////////////////////////////////////////////////////////////////

    /// Returns an iterator over the possibly contained value.
    ///
    /// The iterator yields one value if the presence is [`Some`], otherwise none.
    ///
    /// [`Some`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some(42);
    /// let mut iter = x.iter();
    /// assert_eq!(iter.next(), Some(&42));
    /// assert_eq!(iter.next(), None);
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// let mut iter = y.iter();
    /// assert_eq!(iter.next(), None);
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// let mut iter = z.iter();
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub const fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: Item {
                presence: self.as_ref(),
            },
        }
    }

    /// Returns a mutable iterator over the possibly contained value.
    ///
    /// The iterator yields one mutable reference if the presence is [`Some`], otherwise none.
    ///
    /// [`Some`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let mut x = Presence::Some(42);
    /// for v in x.iter_mut() {
    ///     *v = 100;
    /// }
    /// assert_eq!(x, Presence::Some(100));
    ///
    /// let mut y: Presence<i32> = Presence::Null;
    /// let mut iter = y.iter_mut();
    /// assert_eq!(iter.next(), None);
    ///
    /// let mut z: Presence<i32> = Presence::Absent;
    /// let mut iter = z.iter_mut();
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            inner: Item {
                presence: self.as_mut(),
            },
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Transforming contained values
    /////////////////////////////////////////////////////////////////////////
}

/// Display implementation
impl<T: fmt::Display> fmt::Display for Presence<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Presence::Absent => write!(f, "(absent)"),
            Presence::Null => write!(f, "null"),
            Presence::Some(val) => write!(f, "{}", val),
        }
    }
}

// Default implementation
impl<T> Default for Presence<T> {
    /// Returns the default `Presence` value, which is [`Absent`].
    ///
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x: Presence<i32> = Default::default();
    /// assert_eq!(x, Presence::Absent);
    /// ```
    fn default() -> Presence<T> {
        Presence::Absent
    }
}

// Iterator implementation
impl<T> IntoIterator for Presence<T> {
    type Item = T;
    type IntoIter = Item<T>;

    /// Returns a consuming iterator over the possibly contained value.
    ///
    /// The iterator yields one value if the presence is [`Some`], otherwise none.
    ///
    /// [`Some`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::presence::Presence;
    ///
    /// let x = Presence::Some(42);
    /// let v: Vec<_> = x.into_iter().collect();
    /// assert_eq!(v, vec![42]);
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// let v: Vec<_> = y.into_iter().collect();
    /// assert_eq!(v, vec![]);
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// let v: Vec<_> = z.into_iter().collect();
    /// assert_eq!(v, vec![]);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        Item { presence: self }
    }
}

/////////////////////////////////////////////////////////////////////////////
// The Presence Iterators
//////////////////////////////////////////////////////////////////////////

/// An iterator that moves out of a `Presence`.
///
/// This struct is created by the [`into_iter`] method on [`Presence`] (provided
/// by the [`IntoIterator`] trait).
///
/// [`into_iter`]: IntoIterator::into_iter
/// [`Presence`]: Presence
///
/// # Examples
///
/// ```
/// use presence_rs::presence::Presence;
///
/// let x = Presence::Some(42);
/// let mut iter = x.into_iter();
/// assert_eq!(iter.next(), Some(42));
/// assert_eq!(iter.next(), None);
/// ```
#[derive(Clone, Debug)]
pub struct Item<A> {
    presence: Presence<A>,
}

impl<A> Iterator for Item<A> {
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.presence.take() {
            Presence::Some(val) => Some(val),
            Presence::Null | Presence::Absent => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<A> DoubleEndedIterator for Item<A> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.presence.take() {
            Presence::Some(val) => Some(val),
            Presence::Null | Presence::Absent => None,
        }
    }
}

impl<A> ExactSizeIterator for Item<A> {
    #[inline]
    fn len(&self) -> usize {
        self.presence.len()
    }
}

impl<A> FusedIterator for Item<A> {}

/// An iterator over a reference to the `Some` variant of a `Presence`.
///
/// This struct is created by the [`iter`] method on [`Presence`].
///
/// [`iter`]: Presence::iter
/// [`Presence`]: Presence
///
/// # Examples
///
/// ```
/// use presence_rs::presence::Presence;
///
/// let x = Presence::Some(42);
/// let mut iter = x.iter();
/// assert_eq!(iter.next(), Some(&42));
/// assert_eq!(iter.next(), None);
/// ```
#[derive(Debug, Clone)]
pub struct Iter<'a, A> {
    inner: Item<&'a A>,
}

impl<'a, A> Iterator for Iter<'a, A> {
    type Item = &'a A;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, A> DoubleEndedIterator for Iter<'a, A> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a, A> ExactSizeIterator for Iter<'a, A> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<A> FusedIterator for Iter<'_, A> {}

/// An iterator over a mutable reference to the `Some` variant of a `Presence`.
///
/// This struct is created by the [`iter_mut`] method on [`Presence`].
///
/// [`iter_mut`]: Presence::iter_mut
/// [`Presence`]: Presence
///
/// # Examples
///
/// ```
/// use presence_rs::presence::Presence;
///
/// let mut x = Presence::Some(42);
/// for v in x.iter_mut() {
///     *v = 100;
/// }
/// assert_eq!(x, Presence::Some(100));
/// ```
#[derive(Debug)]
pub struct IterMut<'a, A> {
    inner: Item<&'a mut A>,
}

impl<'a, A> Iterator for IterMut<'a, A> {
    type Item = &'a mut A;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, A> DoubleEndedIterator for IterMut<'a, A> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a, A> ExactSizeIterator for IterMut<'a, A> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<A> FusedIterator for IterMut<'_, A> {}
