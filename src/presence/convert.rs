use super::Presence;

impl<T> Presence<T> {
    /// Converts to `Option<T>`, treating both [`Null`] and [`Absent`] as `None`.
    ///
    /// This is the "optional" representation where only concrete values matter.
    ///
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x = Presence::Some(42);
    /// assert_eq!(x.to_optional(), Some(42));
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert_eq!(y.to_optional(), None);
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert_eq!(z.to_optional(), None);
    /// ```
    #[inline]
    #[must_use = "Returns the converted Option"]
    pub fn to_optional(self) -> Option<T> {
        match self {
            Presence::Some(value) => Some(value),
            Presence::Null | Presence::Absent => None,
        }
    }

    /// Converts to `Option<Option<T>>`, preserving all three states.
    ///
    /// - [`Absent`] → `None`
    /// - [`Null`] → `Some(None)`
    /// - [`Some(v)`] → `Some(Some(v))`
    ///
    /// This is the "nullable" representation that preserves the distinction
    /// between absent and explicitly null.
    ///
    /// [`Absent`]: Presence::Absent
    /// [`Null`]: Presence::Null
    /// [`Some(v)`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x = Presence::Some(42);
    /// assert_eq!(x.to_nullable(), Some(Some(42)));
    ///
    /// let y: Presence<i32> = Presence::Null;
    /// assert_eq!(y.to_nullable(), Some(None));
    ///
    /// let z: Presence<i32> = Presence::Absent;
    /// assert_eq!(z.to_nullable(), None);
    /// ```
    #[inline]
    #[must_use = "Returns the converted nested Option"]
    pub fn to_nullable(self) -> Option<Option<T>> {
        match self {
            Presence::Some(value) => Some(Some(value)),
            Presence::Null => Some(None),
            Presence::Absent => None,
        }
    }

    /// Creates from `Option<T>`, treating `None` as [`Absent`].
    ///
    /// This is the "optional" representation where `None` means the field is absent.
    ///
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let opt = Some(42);
    /// assert_eq!(Presence::from_optional(opt), Presence::Some(42));
    ///
    /// let opt: Option<i32> = None;
    /// assert_eq!(Presence::from_optional(opt), Presence::Absent);
    /// ```
    #[inline]
    pub fn from_optional(opt: Option<T>) -> Self {
        match opt {
            Some(value) => Presence::Some(value),
            None => Presence::Absent,
        }
    }

    /// Creates from `Option<Option<T>>`, preserving all three states.
    ///
    /// - `None` → [`Absent`]
    /// - `Some(None)` → [`Null`]
    /// - `Some(Some(v))` → [`Some(v)`]
    ///
    /// This is the "nullable" representation that distinguishes between
    /// absent and explicitly null.
    ///
    /// [`Absent`]: Presence::Absent
    /// [`Null`]: Presence::Null
    /// [`Some(v)`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let opt = Some(Some(42));
    /// assert_eq!(Presence::from_nullable(opt), Presence::Some(42));
    ///
    /// let opt: Option<Option<i32>> = Some(None);
    /// assert_eq!(Presence::from_nullable(opt), Presence::Null);
    ///
    /// let opt: Option<Option<i32>> = None;
    /// assert_eq!(Presence::from_nullable(opt), Presence::Absent);
    /// ```
    #[inline]
    pub fn from_nullable(opt: Option<Option<T>>) -> Self {
        match opt {
            Some(Some(value)) => Presence::Some(value),
            Some(None) => Presence::Null,
            None => Presence::Absent,
        }
    }

    /// Converts from `Presence<T>` to `Option<Option<T>>`; the same as
    /// [`to_nullable`](Presence::to_nullable).
    #[deprecated(since = "0.3.0", note = "use `to_nullable`, which does the same")]
    #[inline]
    pub fn to_nested_option(self) -> Option<Option<T>> {
        self.to_nullable()
    }

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
    /// use presence_rs::Presence;
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
    ///
    /// # Errors
    ///
    /// Returns `Err(err)` if the presence is [`Null`] or [`Absent`].
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
    /// use presence_rs::Presence;
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
    ///
    /// # Errors
    ///
    /// Returns `Err(err())` if the presence is [`Null`] or [`Absent`].
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
}

impl<T, E> Presence<Result<T, E>> {
    /// Transposes a `Presence` of a [`Result`] into a [`Result`] of a `Presence`.
    ///
    /// State rule: [self decides](mod@crate::presence#combining-states): a `Null` or `Absent` receiver keeps its state.
    ///
    /// [`Absent`]: Presence::Absent
    /// [`Null`]: Presence::Null
    /// [Ok]: Result::Ok
    /// [Err]: Result::Err
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// #[derive(Debug, Eq, PartialEq)]
    /// struct SomeErr;
    ///
    /// let x: Presence<Result<i32, SomeErr>> = Presence::Some(Ok(5));
    /// let y: Result<Presence<i32>, SomeErr> = Ok(Presence::Some(5));
    /// assert_eq!(x.transpose(), y);
    ///
    /// let x: Presence<Result<i32, SomeErr>> = Presence::Some(Err(SomeErr));
    /// let y: Result<Presence<i32>, SomeErr> = Err(SomeErr);
    /// assert_eq!(x.transpose(), y);
    ///
    /// let x: Presence<Result<i32, SomeErr>> = Presence::Null;
    /// let y: Result<Presence<i32>, SomeErr> = Ok(Presence::Null);
    /// assert_eq!(x.transpose(), y);
    ///
    /// let x: Presence<Result<i32, SomeErr>> = Presence::Absent;
    /// let y: Result<Presence<i32>, SomeErr> = Ok(Presence::Absent);
    /// assert_eq!(x.transpose(), y);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err(e)` if the presence is `Some(Err(e))`.
    #[inline]
    #[must_use = "this returns the transposed result, without modifying the original"]
    pub fn transpose(self) -> Result<Presence<T>, E> {
        match self {
            Presence::Some(Ok(v)) => Ok(Presence::Some(v)),
            Presence::Some(Err(e)) => Err(e),
            Presence::Null => Ok(Presence::Null),
            Presence::Absent => Ok(Presence::Absent),
        }
    }
}

impl<T> From<T> for Presence<T> {
    /// Converts a value of type `T` into `Presence::Some(T)`.
    ///
    /// This applies to `Option` values too: converting `None` into a
    /// `Presence<Option<U>>` gives `Some(None)`, not `Null` or `Absent`. To map an
    /// `Option` onto the states, use [`from_optional`](Presence::from_optional)
    /// (`None` becomes `Absent`) or [`from_nullable`](Presence::from_nullable) for
    /// `Option<Option<T>>`.
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let none: Option<i32> = None;
    /// let wrapped: Presence<Option<i32>> = none.into();
    /// assert_eq!(wrapped, Presence::Some(None));
    ///
    /// let absent: Presence<i32> = Presence::from_optional(None);
    /// assert_eq!(absent, Presence::Absent);
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x: Presence<i32> = 42.into();
    /// assert_eq!(x, Presence::Some(42));
    ///
    /// let s: Presence<String> = "hello".to_string().into();
    /// assert_eq!(s, Presence::Some("hello".to_string()));
    /// ```
    #[inline]
    fn from(value: T) -> Self {
        Presence::Some(value)
    }
}

impl<'a, T> From<&'a Presence<T>> for Presence<&'a T> {
    /// Converts from `&Presence<T>` to `Presence<&T>`, the same as
    /// [`as_ref`](Presence::as_ref). `Null` and `Absent` keep their state.
    ///
    /// As with `Option`, name the target type: `Presence::from(&p)` alone could also mean
    /// the blanket `From<T>`, which wraps the reference in `Some`.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let name = Presence::Some(String::from("Ada"));
    /// let borrowed: Presence<&String> = Presence::from(&name);
    /// assert_eq!(borrowed.map(String::len), Presence::Some(3));
    ///
    /// let null: Presence<String> = Presence::Null;
    /// let borrowed: Presence<&String> = Presence::from(&null);
    /// assert_eq!(borrowed, Presence::Null);
    ///
    /// let absent: Presence<String> = Presence::Absent;
    /// let borrowed: Presence<&String> = (&absent).into();
    /// assert_eq!(borrowed, Presence::Absent);
    /// ```
    #[inline]
    fn from(presence: &'a Presence<T>) -> Presence<&'a T> {
        presence.as_ref()
    }
}

impl<'a, T> From<&'a mut Presence<T>> for Presence<&'a mut T> {
    /// Converts from `&mut Presence<T>` to `Presence<&mut T>`, the same as
    /// [`as_mut`](Presence::as_mut). `Null` and `Absent` keep their state.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let mut count = Presence::Some(41);
    /// if let Presence::Some(value) = Presence::<&mut i32>::from(&mut count) {
    ///     *value += 1;
    /// }
    /// assert_eq!(count, Presence::Some(42));
    ///
    /// let mut null: Presence<i32> = Presence::Null;
    /// let borrowed: Presence<&mut i32> = (&mut null).into();
    /// assert_eq!(borrowed, Presence::Null);
    ///
    /// let mut absent: Presence<i32> = Presence::Absent;
    /// let borrowed: Presence<&mut i32> = (&mut absent).into();
    /// assert_eq!(borrowed, Presence::Absent);
    /// ```
    #[inline]
    fn from(presence: &'a mut Presence<T>) -> Presence<&'a mut T> {
        presence.as_mut()
    }
}

impl<T> From<Option<Option<T>>> for Presence<T> {
    /// Converts a nested `Option<Option<T>>` into `Presence<T>`.
    ///
    /// - `None` → `Absent`
    /// - `Some(None)` → `Null`
    /// - `Some(Some(v))` → `Some(v)`
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x: Option<Option<i32>> = Some(Some(42));
    /// let p: Presence<i32> = x.into();
    /// assert_eq!(p, Presence::Some(42));
    ///
    /// let x: Option<Option<i32>> = Some(None);
    /// let p: Presence<i32> = x.into();
    /// assert_eq!(p, Presence::Null);
    ///
    /// let x: Option<Option<i32>> = None;
    /// let p: Presence<i32> = x.into();
    /// assert_eq!(p, Presence::Absent);
    /// ```
    #[inline]
    fn from(opt: Option<Option<T>>) -> Self {
        match opt {
            None => Presence::Absent,
            Some(None) => Presence::Null,
            Some(Some(value)) => Presence::Some(value),
        }
    }
}

impl<T> From<Presence<T>> for Option<Option<T>> {
    /// Converts a `Presence<T>` into a nested `Option<Option<T>>`.
    ///
    /// - `Absent` → `None`
    /// - `Null` → `Some(None)`
    /// - `Some(v)` → `Some(Some(v))`
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let p = Presence::Some(42);
    /// let opt: Option<Option<i32>> = p.into();
    /// assert_eq!(opt, Some(Some(42)));
    ///
    /// let p: Presence<i32> = Presence::Null;
    /// let opt: Option<Option<i32>> = p.into();
    /// assert_eq!(opt, Some(None));
    ///
    /// let p: Presence<i32> = Presence::Absent;
    /// let opt: Option<Option<i32>> = p.into();
    /// assert_eq!(opt, None);
    /// ```
    #[inline]
    fn from(presence: Presence<T>) -> Self {
        match presence {
            Presence::Absent => None,
            Presence::Null => Some(None),
            Presence::Some(value) => Some(Some(value)),
        }
    }
}
