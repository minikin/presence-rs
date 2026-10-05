use super::Presence;

impl<T> Presence<T> {
    /// Returns `true` if the presence is [`Absent`].
    ///
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
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
    /// use presence_rs::Presence;
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
    /// use presence_rs::Presence;
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

    /// Returns `true` if the field is defined (present in the structure).
    ///
    /// Returns `true` for [`Some`] or [`Null`] (field exists), `false` for [`Absent`] (field missing).
    /// This follows IPLD schema semantics where a field can be present with a null value.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x = Presence::Some(42);
    /// assert!(x.is_defined());
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert!(y.is_defined());  // Null means field is present but null
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert!(!z.is_defined());  // Absent means field is not in structure
    /// ```
    #[inline]
    pub const fn is_defined(&self) -> bool {
        !matches!(self, Presence::Absent)
    }

    /// Returns `true` if the value is "nullish" (null-like).
    ///
    /// Returns `true` for [`Null`] or [`Absent`], `false` for [`Some`].
    /// Useful for detecting any kind of "empty" or "missing" state.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x = Presence::Some(42);
    /// assert!(!x.is_nullish());
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert!(y.is_nullish());
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert!(z.is_nullish());
    /// ```
    #[inline]
    pub const fn is_nullish(&self) -> bool {
        !matches!(self, Presence::Some(_))
    }

    /// Returns `true` if the presence is [`Some`] and the value inside of it matches a predicate.
    ///
    /// [`Some`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
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
    /// use presence_rs::Presence;
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
    /// "Nullish" means `Null` or `Absent`, as in [`is_nullish`](Presence::is_nullish).
    ///
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x: Presence<u32> = Presence::Some(2);
    /// assert_eq!(x.is_nullish_or(|x| x > 1), true);
    ///
    /// let x: Presence<u32> = Presence::Some(0);
    /// assert_eq!(x.is_nullish_or(|x| x > 1), false);
    ///
    /// let x: Presence<u32> = Presence::Null;
    /// assert_eq!(x.is_nullish_or(|x| x > 1), true);
    ///
    /// let x: Presence<u32> = Presence::Absent;
    /// assert_eq!(x.is_nullish_or(|x| x > 1), true);
    /// ```
    #[inline]
    pub fn is_nullish_or(self, f: impl FnOnce(T) -> bool) -> bool {
        match self {
            Presence::Some(val) => f(val),
            Presence::Null | Presence::Absent => true,
        }
    }

    /// Returns `true` if the presence is `Null` or `Absent`, or the value inside matches a
    /// predicate.
    #[deprecated(
        since = "0.3.0",
        note = "renamed to `is_nullish_or`; it is also `true` for `Absent`"
    )]
    #[inline]
    pub fn is_null_or(self, f: impl FnOnce(T) -> bool) -> bool {
        self.is_nullish_or(f)
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
    /// use presence_rs::Presence;
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

    /// Returns `true` if the presence contains no value (is [`Null`] or [`Absent`]).
    ///
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x: Presence<i32> = Presence::Some(42);
    /// assert!(!x.is_empty());
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert!(y.is_empty());
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert!(z.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Presence::Null | Presence::Absent)
    }
}
