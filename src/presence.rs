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

    /// Returns true if the field is absent
    /// # Examples
    ///
    #[inline]
    pub const fn is_absent(&self) -> bool {
        matches!(self, Presence::Absent)
    }

    /// Returns true if the value is null
    /// # Examples
    ///
    #[inline]
    pub const fn is_null(&self) -> bool {
        matches!(self, Presence::Null)
    }

    /// Returns true if a concrete value is present.
    /// # Examples
    ///
    #[inline]
    pub const fn is_present(&self) -> bool {
        matches!(self, Presence::Some(_))
    }

    /// Converts from `&Presence<T>` to `Presence<&T>`.
    /// # Examples
    ///
    #[inline]
    pub const fn as_ref(&self) -> Presence<&T> {
        match *self {
            Presence::Some(ref val) => Presence::Some(val),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Converts from `&mut Presence<T>` to `Presence<&mut T>`.
    /// # Examples
    ///
    #[inline]
    pub const fn as_mut(&mut self) -> Presence<&mut T> {
        match *self {
            Presence::Some(ref mut val) => Presence::Some(val),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Convert Presence<T> to Option<Option<T>> for interop
    /// # Examples
    ///
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

    /// Takes the value out of the Presence, leaving Absent in its place.
    /// # Examples
    ///
    #[inline]
    pub const fn take(&mut self) -> Presence<T> {
        let mut slot = Presence::Absent;
        std::mem::swap(self, &mut slot);
        slot
    }

    /// Returns the number of elements in the Presence (0, 1, or possibly 0 for Null/Absent).
    /// This is primarily for iterator support.
    /// # Examples
    ///
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
    /// # Examples
    ///
    #[inline]
    pub const fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: Item {
                presence: self.as_ref(),
            },
        }
    }

    /// Returns a mutable iterator over the possibly contained value.
    /// # Examples
    ///
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
    /// Returns the default Presence value, which is Absent
    /// # Examples
    ///
    fn default() -> Presence<T> {
        Presence::Absent
    }
}

// Iterator implementation
impl<T> IntoIterator for Presence<T> {
    type Item = T;
    type IntoIter = Item<T>;

    /// Creates an iterator that yields the contained value if present
    /// # Examples
    ///
    fn into_iter(self) -> Self::IntoIter {
        Item { presence: self }
    }
}

/////////////////////////////////////////////////////////////////////////////
// The Presence Iterators
//////////////////////////////////////////////////////////////////////////
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
