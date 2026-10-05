use super::Presence;

impl<T> Presence<T> {
    /// Converts from `&Presence<T>` to `Presence<&T>`.
    ///
    /// Produces a new `Presence`, containing a reference into the original, leaving
    /// the original in place.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
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
    /// use presence_rs::Presence;
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
    /// use presence_rs::Presence;
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
    pub const fn as_pin_ref(self: core::pin::Pin<&Self>) -> Presence<core::pin::Pin<&T>> {
        match core::pin::Pin::get_ref(self) {
            // SAFETY: `val` points into `self`, which is pinned, so it never moves either.
            Presence::Some(val) => unsafe { Presence::Some(core::pin::Pin::new_unchecked(val)) },
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
    /// use presence_rs::Presence;
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
    pub const fn as_pin_mut(self: core::pin::Pin<&mut Self>) -> Presence<core::pin::Pin<&mut T>> {
        // SAFETY: the `&mut Self` from `get_unchecked_mut` is only used to reach `val`, never to
        // move the `Presence`; `val` points into the pinned `self`, so re-pinning it is sound.
        unsafe {
            match core::pin::Pin::get_unchecked_mut(self) {
                Presence::Some(val) => Presence::Some(core::pin::Pin::new_unchecked(val)),
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
    /// use presence_rs::Presence;
    ///
    /// let x = Presence::Some(42);
    /// assert_eq!(x.as_slice(), &[42]);
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert!(y.as_slice().is_empty());
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert!(z.as_slice().is_empty());
    /// ```
    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        match self {
            Presence::Some(val) => core::slice::from_ref(val),
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
    /// use presence_rs::Presence;
    ///
    /// let mut x = Presence::Some(42);
    /// let slice = x.as_mut_slice();
    /// if let Some(first) = slice.first_mut() {
    ///     *first = 100;
    /// }
    /// assert_eq!(x, Presence::Some(100));
    ///
    /// let mut y: Presence<i32> = Presence::Null;
    /// assert!(y.as_mut_slice().is_empty());
    ///
    /// let mut z: Presence<i32> = Presence::Absent;
    /// assert!(z.as_mut_slice().is_empty());
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            Presence::Some(val) => core::slice::from_mut(val),
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
    /// use presence_rs::Presence;
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
        T: core::ops::Deref,
    {
        match self.as_ref() {
            Presence::Some(val) => Presence::Some(&**val),
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
    /// use presence_rs::Presence;
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
        T: core::ops::DerefMut,
    {
        match self.as_mut() {
            Presence::Some(val) => Presence::Some(&mut **val),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }
}

impl<T> Presence<&T> {
    /// Maps a `Presence<&T>` to a `Presence<T>` by copying the contents of the
    /// presence.
    ///
    /// State rule: [self decides](mod@crate::presence#combining-states): a `Null` or `Absent` receiver keeps its state.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x = 12;
    /// let opt_x = Presence::Some(&x);
    /// assert_eq!(opt_x, Presence::Some(&12));
    /// let copied = opt_x.copied();
    /// assert_eq!(copied, Presence::Some(12));
    ///
    /// let y: Presence<&i32> = Presence::Null;
    /// assert_eq!(y.copied(), Presence::Null);
    ///
    /// let z: Presence<&i32> = Presence::Absent;
    /// assert_eq!(z.copied(), Presence::Absent);
    /// ```
    #[inline]
    pub const fn copied(self) -> Presence<T>
    where
        T: Copy,
    {
        match self {
            Presence::Some(&val) => Presence::Some(val),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Maps a `Presence<&T>` to a `Presence<T>` by cloning the contents of the
    /// presence.
    ///
    /// State rule: [self decides](mod@crate::presence#combining-states): a `Null` or `Absent` receiver keeps its state.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x = 12;
    /// let opt_x = Presence::Some(&x);
    /// assert_eq!(opt_x, Presence::Some(&12));
    /// let cloned = opt_x.cloned();
    /// assert_eq!(cloned, Presence::Some(12));
    ///
    /// let y: Presence<&i32> = Presence::Null;
    /// assert_eq!(y.cloned(), Presence::Null);
    ///
    /// let z: Presence<&i32> = Presence::Absent;
    /// assert_eq!(z.cloned(), Presence::Absent);
    /// ```
    #[inline]
    pub fn cloned(self) -> Presence<T>
    where
        T: Clone,
    {
        match self {
            Presence::Some(val) => Presence::Some(val.clone()),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }
}

impl<T> Presence<&mut T> {
    /// Maps a `Presence<&mut T>` to a `Presence<T>` by copying the contents of the
    /// presence.
    ///
    /// State rule: [self decides](mod@crate::presence#combining-states): a `Null` or `Absent` receiver keeps its state.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let mut x = 12;
    /// let opt_x = Presence::Some(&mut x);
    /// assert_eq!(opt_x, Presence::Some(&mut 12));
    /// let copied = opt_x.copied();
    /// assert_eq!(copied, Presence::Some(12));
    ///
    /// let mut y: Presence<&mut i32> = Presence::Null;
    /// assert_eq!(y.copied(), Presence::Null);
    ///
    /// let mut z: Presence<&mut i32> = Presence::Absent;
    /// assert_eq!(z.copied(), Presence::Absent);
    /// ```
    #[inline]
    pub const fn copied(self) -> Presence<T>
    where
        T: Copy,
    {
        match self {
            Presence::Some(&mut val) => Presence::Some(val),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }

    /// Maps a `Presence<&mut T>` to a `Presence<T>` by cloning the contents of the
    /// presence.
    ///
    /// State rule: [self decides](mod@crate::presence#combining-states): a `Null` or `Absent` receiver keeps its state.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let mut x = 12;
    /// let opt_x = Presence::Some(&mut x);
    /// assert_eq!(opt_x, Presence::Some(&mut 12));
    /// let cloned = opt_x.cloned();
    /// assert_eq!(cloned, Presence::Some(12));
    ///
    /// let mut y: Presence<&mut i32> = Presence::Null;
    /// assert_eq!(y.cloned(), Presence::Null);
    ///
    /// let mut z: Presence<&mut i32> = Presence::Absent;
    /// assert_eq!(z.cloned(), Presence::Absent);
    /// ```
    #[inline]
    pub fn cloned(self) -> Presence<T>
    where
        T: Clone,
    {
        match self {
            Presence::Some(val) => Presence::Some(val.clone()),
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }
}
