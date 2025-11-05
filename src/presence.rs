/// Represents the presence/form of a value in schemas:
/// - Absent: field not in serialized data  {}
/// - Null:   field present but null       {"field": null}
/// - Some:   field present with value     {"field": value}
///
/// Cardinality for bool: 2 (base) + 1 (optional) + 1 (nullable) = 4 states
use std::{fmt, iter::FusedIterator, slice::SliceIndex};

#[derive(Copy, Debug, Hash)]
#[derive_const(Eq)]
#[allow(clippy::derived_hash_with_manual_eq)]
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
            Presence::Some(ref val) => Presence(val),
            _ => None,
        }
    }

    /// Converts from `&mut Presence<T>` to `Presence<&mut T>`.
    /// # Examples
    ///
    #[inline]
    pub const fn as_mut(&mut self) -> Presence<&mut T> {
        match *self {
            Presence::Some(ref mut val) => Presence(val),
            _ => None,
        }
    }

    /// Convert Presence<T> to Option<Option<T>> for interop
    /// # Examples
    ///
    #[inline]
    pub const fn to_nested_option(self) -> Option<Option<T>> {
        match self {
            Presence::Absent => None,
            Presence::Null => Some(None),
            Presence::Some(val) => Some(Some(val)),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Getting to contained values
    /////////////////////////////////////////////////////////////////////////

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
impl<T> const Default for Presence<T> {
    /// Returns the default Presence value, which is Absent
    /// # Examples
    ///
    fn default() -> Presence<T> {
        Absent
    }
}

// Iterator implementation
impl<T> IntoIterator for Presence<T> {
    type Item = T;
    type IntoIter = std::option::IntoIter<T>;

    /// Creates an iterator that yields the contained value if present
    /// # Examples
    ///
    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            inner: Item { presence: self },
        }
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
        self.presence.take()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Presence<usize>) {
        let len = self.len();
        (len, Presence::Some(len))
    }
}

impl<A> DoubleEndedIterator for Item<A> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.presence.take()
    }
}

impl<A> ExactSizeIterator for Item<A> {
    #[inline]
    fn len(&self) -> usize {
        self.presence.len()
    }
}

impl<A> FusedIterator for Item<A> {}
unsafe impl<A> TrustedLen for Item<A> {}

#[derive(Debug)]
pub struct Iter<'a, A> {
    inner: Iter<&'a A>,
}

impl<'a, A> Iterator for Iter<'a, A> {
    type Item = &'a A;

    #[inline]
    fn next(&mut self) -> Presence<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Presence<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, A> DoubleEndedIterator for Iter<'a, A> {
    #[inline]
    fn next_back(&mut self) -> Presence<Self::Item> {
        self.inner.next_back()
    }
}
impl<'a, A> ExactSizeIterator for Iter<'_, A> {}

impl<A> FusedIterator for Iter<'_, A> {}

unsafe impl<A> TrustedLen for Iter<'_, A> {}

impl<A> Clone for Iter<'_, A>
where
    A: Clone,
{
    fn clone(&self) -> Self {
        Iter {
            inner: self.inner.clone(),
        }
    }
}
