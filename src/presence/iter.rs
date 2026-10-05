use core::iter::FusedIterator;

use super::Presence;

impl<T> Presence<T> {
    /// Returns an iterator over the possibly contained value.
    ///
    /// The iterator yields one value if the presence is [`Some`], otherwise none.
    ///
    /// [`Some`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
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
            inner: IntoIter {
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
    /// use presence_rs::Presence;
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
            inner: IntoIter {
                presence: self.as_mut(),
            },
        }
    }
}

// Iterator implementation
impl<T> IntoIterator for Presence<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Returns a consuming iterator over the possibly contained value.
    ///
    /// The iterator yields one value if the presence is [`Some`], otherwise none.
    ///
    /// [`Some`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
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
        IntoIter { presence: self }
    }
}

impl<'a, T> IntoIterator for &'a Presence<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    /// Returns an iterator over a reference to the possibly contained value.
    ///
    /// The iterator yields one reference if the presence is [`Some`], otherwise none.
    /// This is what makes `for x in &presence` work; it is equivalent to [`iter`].
    ///
    /// [`Some`]: Presence::Some
    /// [`iter`]: Presence::iter
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x = Presence::Some(42);
    /// let mut seen = Vec::new();
    /// for v in &x {
    ///     seen.push(*v);
    /// }
    /// assert_eq!(seen, vec![42]);
    /// assert_eq!(x, Presence::Some(42)); // still usable
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert_eq!((&y).into_iter().next(), None);
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert_eq!((&z).into_iter().next(), None);
    /// ```
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Presence<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    /// Returns an iterator over a mutable reference to the possibly contained value.
    ///
    /// The iterator yields one mutable reference if the presence is [`Some`], otherwise none.
    /// This is what makes `for x in &mut presence` work; it is equivalent to [`iter_mut`].
    ///
    /// [`Some`]: Presence::Some
    /// [`iter_mut`]: Presence::iter_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let mut x = Presence::Some(42);
    /// for v in &mut x {
    ///     *v = 100;
    /// }
    /// assert_eq!(x, Presence::Some(100));
    ///
    /// let mut y: Presence<i32> = Presence::Null;
    /// assert_eq!((&mut y).into_iter().next(), None);
    /// assert_eq!(y, Presence::Null);
    ///
    /// let mut z: Presence<i32> = Presence::Absent;
    /// assert_eq!((&mut z).into_iter().next(), None);
    /// assert_eq!(z, Presence::Absent);
    /// ```
    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

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
/// use presence_rs::Presence;
///
/// let x = Presence::Some(42);
/// let mut iter = x.into_iter();
/// assert_eq!(iter.next(), Some(42));
/// assert_eq!(iter.next(), None);
/// ```
#[derive(Clone, Debug)]
pub struct IntoIter<A> {
    presence: Presence<A>,
}

impl<A> Iterator for IntoIter<A> {
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

impl<A> DoubleEndedIterator for IntoIter<A> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.presence.take() {
            Presence::Some(val) => Some(val),
            Presence::Null | Presence::Absent => None,
        }
    }
}

impl<A> ExactSizeIterator for IntoIter<A> {
    #[inline]
    fn len(&self) -> usize {
        self.presence.len()
    }
}

impl<A> FusedIterator for IntoIter<A> {}

/// An iterator over a reference to the `Some` variant of a `Presence`.
///
/// This struct is created by the [`iter`] method on [`Presence`], or by iterating
/// over `&Presence<T>` through its [`IntoIterator`] implementation.
///
/// [`iter`]: Presence::iter
/// [`Presence`]: Presence
///
/// # Examples
///
/// ```
/// use presence_rs::Presence;
///
/// let x = Presence::Some(42);
/// let mut iter = x.iter();
/// assert_eq!(iter.next(), Some(&42));
/// assert_eq!(iter.next(), None);
/// ```
#[derive(Debug, Clone)]
pub struct Iter<'a, A> {
    inner: IntoIter<&'a A>,
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

impl<A> DoubleEndedIterator for Iter<'_, A> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<A> ExactSizeIterator for Iter<'_, A> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<A> FusedIterator for Iter<'_, A> {}

/// An iterator over a mutable reference to the `Some` variant of a `Presence`.
///
/// This struct is created by the [`iter_mut`] method on [`Presence`], or by iterating
/// over `&mut Presence<T>` through its [`IntoIterator`] implementation.
///
/// [`iter_mut`]: Presence::iter_mut
/// [`Presence`]: Presence
///
/// # Examples
///
/// ```
/// use presence_rs::Presence;
///
/// let mut x = Presence::Some(42);
/// for v in x.iter_mut() {
///     *v = 100;
/// }
/// assert_eq!(x, Presence::Some(100));
/// ```
#[derive(Debug)]
pub struct IterMut<'a, A> {
    inner: IntoIter<&'a mut A>,
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

impl<A> DoubleEndedIterator for IterMut<'_, A> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<A> ExactSizeIterator for IterMut<'_, A> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<A> FusedIterator for IterMut<'_, A> {}

/// Feeds the `Some` values of `iter` to `combine` without buffering them, then applies
/// the `Absent` over `Null` over `Some` rule to the result.
///
/// `combine` sees the values up to the first `Null` or `Absent`, and may stop earlier
/// (collecting into `Option` stops at the first `None`). Unless an `Absent` was seen,
/// the rest of `iter` is then read until an `Absent` turns up, because a later `Null`
/// or `Absent` still decides the result.
fn stream<T, R, I>(iter: I, combine: impl FnOnce(Values<'_, I::IntoIter>) -> R) -> Presence<R>
where
    I: IntoIterator<Item = Presence<T>>,
{
    let mut iter = iter.into_iter();
    let mut state = Presence::Some(());
    let combined = combine(Values {
        iter: &mut iter,
        state: &mut state,
    });
    if !state.is_absent() {
        for item in iter {
            match item {
                Presence::Some(_) => {}
                Presence::Null => state = Presence::Null,
                Presence::Absent => {
                    state = Presence::Absent;
                    break;
                }
            }
        }
    }
    state.map(|()| combined)
}

/// The `Some` values of a stream of presences, ending at the first `Null` or `Absent`,
/// which it records in `state`.
struct Values<'a, I> {
    iter: &'a mut I,
    state: &'a mut Presence<()>,
}

impl<T, I: Iterator<Item = Presence<T>>> Iterator for Values<'_, I> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if !self.state.is_present() {
            return None;
        }
        match self.iter.next()? {
            Presence::Some(value) => Some(value),
            stop => {
                *self.state = stop.map(|_| ());
                None
            }
        }
    }
}

impl<A, V: FromIterator<A>> FromIterator<Presence<A>> for Presence<V> {
    /// Collects an iterator of `Presence<A>` into `Presence<V>`.
    ///
    /// State rule: [`Absent` over `Null` over `Some`](mod@crate::presence#combining-states), the same
    /// rule as [`Presence::zip`].
    ///
    /// Returns `Absent` if any element is `Absent`.
    /// Returns `Null` if any element is `Null` (and none are `Absent`).
    /// Returns `Some(collection)` only if all elements are `Some`.
    ///
    /// The values are passed to `V::from_iter` as they arrive, without buffering, so a `V`
    /// is always built — from the values before the first `Null` or `Absent`, possibly
    /// none of them — and then discarded if the result is `Null` or `Absent`.
    ///
    /// Reading stops at the first `Absent`. After a `Null` the rest of the input is still
    /// read, because a later `Absent` decides the result, so an endless iterator that never
    /// yields `Absent` never finishes.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let v = vec![Presence::Some(1), Presence::Some(2), Presence::Some(3)];
    /// let result: Presence<Vec<i32>> = v.into_iter().collect();
    /// assert_eq!(result, Presence::Some(vec![1, 2, 3]));
    ///
    /// let v = vec![Presence::Some(1), Presence::Null, Presence::Some(3)];
    /// let result: Presence<Vec<i32>> = v.into_iter().collect();
    /// assert_eq!(result, Presence::Null);
    ///
    /// let v = vec![Presence::Some(1), Presence::Absent, Presence::Some(3)];
    /// let result: Presence<Vec<i32>> = v.into_iter().collect();
    /// assert_eq!(result, Presence::Absent);
    ///
    /// let v = vec![Presence::Some(1), Presence::Absent, Presence::Null];
    /// let result: Presence<Vec<i32>> = v.into_iter().collect();
    /// assert_eq!(result, Presence::Absent);  // Absent takes precedence
    /// ```
    fn from_iter<I: IntoIterator<Item = Presence<A>>>(iter: I) -> Self {
        stream(iter, |values| values.collect())
    }
}

impl<T, U> core::iter::Product<Presence<U>> for Presence<T>
where
    T: core::iter::Product<U>,
{
    /// Computes the product of an iterator of `Presence<U>` values.
    ///
    /// State rule: [`Absent` over `Null` over `Some`](mod@crate::presence#combining-states), the same
    /// rule as [`Presence::zip`].
    ///
    /// Returns `Absent` if any element is `Absent`.
    /// Returns `Null` if any element is `Null` (and none are `Absent`).
    /// Returns `Some(product)` only if all elements are `Some`.
    ///
    /// The values are combined as they arrive, without buffering, so values that come
    /// before a `Null` or `Absent` are still combined before the result is discarded.
    /// As with `Option`, an overflow among them panics in debug builds even though the
    /// result would be `Null` or `Absent`.
    ///
    /// Reading stops at the first `Absent`. After a `Null` the rest of the input is still
    /// read, because a later `Absent` decides the result, so an endless iterator that never
    /// yields `Absent` never finishes.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let v = vec![Presence::Some(2), Presence::Some(3), Presence::Some(4)];
    /// let result: Presence<i32> = v.into_iter().product();
    /// assert_eq!(result, Presence::Some(24));
    ///
    /// let v = vec![Presence::Some(2), Presence::Null, Presence::Some(4)];
    /// let result: Presence<i32> = v.into_iter().product();
    /// assert_eq!(result, Presence::Null);
    ///
    /// let v = vec![Presence::Some(2), Presence::Absent, Presence::Some(4)];
    /// let result: Presence<i32> = v.into_iter().product();
    /// assert_eq!(result, Presence::Absent);
    ///
    /// let empty: Vec<Presence<i32>> = vec![];
    /// let result: Presence<i32> = empty.into_iter().product();
    /// assert_eq!(result, Presence::Some(1));  // Identity element for multiplication
    /// ```
    fn product<I: Iterator<Item = Presence<U>>>(iter: I) -> Self {
        stream(iter, |values| values.product())
    }
}

impl<T, U> core::iter::Sum<Presence<U>> for Presence<T>
where
    T: core::iter::Sum<U>,
{
    /// Computes the sum of an iterator of `Presence<U>` values.
    ///
    /// State rule: [`Absent` over `Null` over `Some`](mod@crate::presence#combining-states), the same
    /// rule as [`Presence::zip`].
    ///
    /// Returns `Absent` if any element is `Absent`.
    /// Returns `Null` if any element is `Null` (and none are `Absent`).
    /// Returns `Some(sum)` only if all elements are `Some`.
    ///
    /// The values are combined as they arrive, without buffering, so values that come
    /// before a `Null` or `Absent` are still combined before the result is discarded.
    /// As with `Option`, an overflow among them panics in debug builds even though the
    /// result would be `Null` or `Absent`.
    ///
    /// Reading stops at the first `Absent`. After a `Null` the rest of the input is still
    /// read, because a later `Absent` decides the result, so an endless iterator that never
    /// yields `Absent` never finishes.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let v = vec![Presence::Some(1), Presence::Some(2), Presence::Some(3)];
    /// let result: Presence<i32> = v.into_iter().sum();
    /// assert_eq!(result, Presence::Some(6));
    ///
    /// let v = vec![Presence::Some(1), Presence::Null, Presence::Some(3)];
    /// let result: Presence<i32> = v.into_iter().sum();
    /// assert_eq!(result, Presence::Null);
    ///
    /// let v = vec![Presence::Some(1), Presence::Absent, Presence::Some(3)];
    /// let result: Presence<i32> = v.into_iter().sum();
    /// assert_eq!(result, Presence::Absent);
    ///
    /// let empty: Vec<Presence<i32>> = vec![];
    /// let result: Presence<i32> = empty.into_iter().sum();
    /// assert_eq!(result, Presence::Some(0));  // Identity element for addition
    /// ```
    fn sum<I: Iterator<Item = Presence<U>>>(iter: I) -> Self {
        stream(iter, |values| values.sum())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::{Presence, Values};
    use proptest::prelude::*;
    use std::vec::Vec;

    #[test]
    fn values_stays_ended_after_a_null_even_when_polled_again() {
        let mut source = [Presence::Some(1), Presence::Null, Presence::Some(2)].into_iter();
        let mut state = Presence::Some(());
        let mut values = Values {
            iter: &mut source,
            state: &mut state,
        };

        assert_eq!(values.next(), Some(1));
        assert_eq!(values.next(), None);
        assert_eq!(values.next(), None);
        assert_eq!(state, Presence::Null);
        assert_eq!(source.next(), Some(Presence::Some(2)));
    }

    #[test]
    fn stream_finds_a_null_after_the_target_stopped_reading() {
        let items = [Presence::Some(0), Presence::Null];

        let collected: Presence<Option<Vec<i64>>> = items
            .into_iter()
            .map(|p| p.map(|x| (x != 0).then_some(x)))
            .collect();

        assert_eq!(collected, Presence::Null);
    }

    #[test]
    fn stream_finds_an_absent_after_a_null_the_target_never_saw() {
        let items = [Presence::Some(0), Presence::Null, Presence::Absent];

        let collected: Presence<Option<Vec<i64>>> = items
            .into_iter()
            .map(|p| p.map(|x| (x != 0).then_some(x)))
            .collect();

        assert_eq!(collected, Presence::Absent);
    }

    #[test]
    fn values_stays_ended_after_an_absent_even_when_polled_again() {
        let mut source = [Presence::Some(1), Presence::Absent, Presence::Null].into_iter();
        let mut state = Presence::Some(());
        let mut values = Values {
            iter: &mut source,
            state: &mut state,
        };

        assert_eq!(values.next(), Some(1));
        assert_eq!(values.next(), None);
        assert_eq!(values.next(), None);
        assert_eq!(state, Presence::Absent);
        assert_eq!(source.next(), Some(Presence::Null));
    }

    fn any_presence() -> impl Strategy<Value = Presence<i32>> {
        prop_oneof![
            any::<i32>().prop_map(Presence::Some),
            Just(Presence::Null),
            Just(Presence::Absent),
        ]
    }

    proptest! {
        #[test]
        fn reference_iteration_matches_iter_and_iter_mut(presence in any_presence()) {
            let expected_len = usize::from(presence.is_present());

            let by_ref = (&presence).into_iter();
            let by_iter = presence.iter();
            prop_assert_eq!(by_ref.len(), expected_len);
            prop_assert_eq!(by_ref.len(), by_iter.len());
            prop_assert_eq!(by_ref.size_hint(), by_iter.size_hint());
            prop_assert_eq!(by_ref.collect::<Vec<_>>(), by_iter.collect::<Vec<_>>());

            let mut for_ref_mut = presence;
            let mut for_iter_mut = presence;
            let by_ref_mut = (&mut for_ref_mut).into_iter();
            let by_iter_mut = for_iter_mut.iter_mut();
            prop_assert_eq!(by_ref_mut.len(), expected_len);
            prop_assert_eq!(by_ref_mut.size_hint(), (expected_len, Some(expected_len)));
            prop_assert_eq!(by_ref_mut.size_hint(), by_iter_mut.size_hint());
            prop_assert_eq!(
                by_ref_mut.map(|v| *v).collect::<Vec<_>>(),
                by_iter_mut.map(|v| *v).collect::<Vec<_>>()
            );
        }
    }
}
