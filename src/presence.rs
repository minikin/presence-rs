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
