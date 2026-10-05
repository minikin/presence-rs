use super::Presence;

impl<T> Presence<T> {
    /// Returns the contained [`Some`] value, `absent_default` for [`Absent`] or
    /// `null_default` for [`Null`].
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    #[deprecated(
        since = "0.3.0",
        note = "renamed to `unwrap_or_absent_or_null`, which takes the `Absent` default first"
    )]
    #[inline]
    pub fn unwrap_or_null_default(self, absent_default: T, null_default: T) -> T {
        self.unwrap_or_absent_or_null(absent_default, null_default)
    }

    /// Returns the contained [`Some`] value, `absent` for [`Absent`] or `null` for
    /// [`Null`].
    ///
    /// This is useful when you need to handle the two "empty" states differently,
    /// such as in IPLD schemas where null and absent have distinct meanings.
    ///
    /// The defaults are taken in the order the name gives them, `Absent` first, as in
    /// the ordering `Absent < Null < Some(_)`. Arguments passed to
    /// `unwrap_or_absent_or_null` are eagerly evaluated. If you are passing the result
    /// of a function call, it is recommended to use [`unwrap_or_else_absent_or_null`],
    /// which is lazily evaluated.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    /// [`unwrap_or_else_absent_or_null`]: Presence::unwrap_or_else_absent_or_null
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// assert_eq!(Presence::Some(42).unwrap_or_absent_or_null(-1, -2), 42);
    /// assert_eq!(Presence::<i32>::Absent.unwrap_or_absent_or_null(-1, -2), -1);
    /// assert_eq!(Presence::<i32>::Null.unwrap_or_absent_or_null(-1, -2), -2);
    /// ```
    #[inline]
    #[must_use = "if you don't need the returned value, use `if let` or `match` instead"]
    pub fn unwrap_or_absent_or_null(self, absent: T, null: T) -> T {
        match self {
            Presence::Some(value) => value,
            Presence::Null => null,
            Presence::Absent => absent,
        }
    }

    /// Returns the contained [`Some`] value, or computes it from `absent` for [`Absent`]
    /// or from `null` for [`Null`].
    ///
    /// Only the closure for the state met is called, and neither is called for `Some`.
    /// This is the lazy form of [`unwrap_or_absent_or_null`], and takes the closures in
    /// the same order, `Absent` first.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    /// [`unwrap_or_absent_or_null`]: Presence::unwrap_or_absent_or_null
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let absent = || -1;
    /// let null = || -2;
    ///
    /// assert_eq!(Presence::Some(42).unwrap_or_else_absent_or_null(absent, null), 42);
    /// assert_eq!(Presence::Absent.unwrap_or_else_absent_or_null(absent, null), -1);
    /// assert_eq!(Presence::Null.unwrap_or_else_absent_or_null(absent, null), -2);
    /// ```
    #[inline]
    #[must_use = "if you don't need the returned value, use `if let` or `match` instead"]
    pub fn unwrap_or_else_absent_or_null<FA, FN>(self, absent: FA, null: FN) -> T
    where
        FA: FnOnce() -> T,
        FN: FnOnce() -> T,
    {
        match self {
            Presence::Some(value) => value,
            Presence::Null => null(),
            Presence::Absent => absent(),
        }
    }

    /// Applies this presence to an `Option` field as a PATCH would, returning what it
    /// replaced.
    ///
    /// - `Absent` leaves `target` unchanged and returns `None`.
    /// - `Null` clears `target` and returns its previous value, like [`Option::take`].
    /// - `Some(value)` sets `target` to `Some(value)` and returns its previous value, like
    ///   [`Option::replace`].
    ///
    /// A `None` return therefore means either that nothing changed or that `target` was
    /// already `None`. See [Patching](mod@crate::presence#patching).
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let mut nickname = Some(String::from("neo"));
    ///
    /// assert_eq!(Presence::Absent.apply_to(&mut nickname), None);
    /// assert_eq!(nickname.as_deref(), Some("neo"));
    ///
    /// let previous = Presence::Some(String::from("trinity")).apply_to(&mut nickname);
    /// assert_eq!(previous.as_deref(), Some("neo"));
    /// assert_eq!(nickname.as_deref(), Some("trinity"));
    ///
    /// let previous = Presence::Null.apply_to(&mut nickname);
    /// assert_eq!(previous.as_deref(), Some("trinity"));
    /// assert_eq!(nickname, None);
    /// ```
    #[inline]
    pub fn apply_to(self, target: &mut Option<T>) -> Option<T> {
        match self {
            Presence::Absent => None,
            Presence::Null => target.take(),
            Presence::Some(value) => target.replace(value),
        }
    }

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
    /// use presence_rs::Presence;
    ///
    /// let x = Presence::Some("value");
    /// assert_eq!(x.expect("should have a value"), "value");
    /// ```
    ///
    /// ```should_panic
    /// use presence_rs::Presence;
    ///
    /// let x: Presence<&str> = Presence::Null;
    /// x.expect("the value was null"); // panics with `the value was null`
    /// ```
    ///
    /// ```should_panic
    /// use presence_rs::Presence;
    ///
    /// let x: Presence<&str> = Presence::Absent;
    /// x.expect("the value was absent"); // panics with `the value was absent`
    /// ```
    #[inline]
    #[track_caller]
    pub fn expect(self, msg: &str) -> T {
        match self {
            Presence::Some(val) => val,
            Presence::Null => panic!("{msg}: value was Null"),
            Presence::Absent => panic!("{msg}: value was Absent"),
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
    /// use presence_rs::Presence;
    ///
    /// let x = Presence::Some("air");
    /// assert_eq!(x.unwrap(), "air");
    /// ```
    ///
    /// ```should_panic
    /// use presence_rs::Presence;
    ///
    /// let x: Presence<&str> = Presence::Null;
    /// x.unwrap(); // panics
    /// ```
    ///
    /// ```should_panic
    /// use presence_rs::Presence;
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
    /// use presence_rs::Presence;
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
    #[must_use = "if you don't need the returned value, use `if let` or `match` instead"]
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
    /// use presence_rs::Presence;
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
    #[must_use = "If you don't need the returned value, use `if let` or `match` instead"]
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
    /// use presence_rs::Presence;
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
    #[must_use = "If you don't need the returned value, use `if let` or `match` instead"]
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
    /// use presence_rs::Presence;
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
        core::mem::swap(self, &mut slot);
        slot
    }

    /// Takes the value out of the `Presence` if the predicate returns `true`,
    /// leaving [`Absent`] in its place.
    ///
    /// [`Absent`]: Presence::Absent
    ///
    /// When nothing is taken the result is `Absent`, not `Null`, and the presence keeps its
    /// state, `Null` included: in PATCH terms `Absent` means "leave the field unchanged",
    /// the safe default, while `Null` would mean "clear it".
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let mut x = Presence::Some(42);
    /// let old = x.take_if(|v| *v == 42);
    /// assert_eq!(x, Presence::Absent);
    /// assert_eq!(old, Presence::Some(42));
    ///
    /// let mut y = Presence::Some(10);
    /// let old = y.take_if(|v| *v == 42);
    /// assert_eq!(y, Presence::Some(10));
    /// assert_eq!(old, Presence::Absent);
    ///
    /// let mut z: Presence<i32> = Presence::Null;
    /// let old = z.take_if(|v| *v == 42);
    /// assert_eq!(z, Presence::Null);
    /// assert_eq!(old, Presence::Absent);
    /// ```
    #[inline]
    pub fn take_if<P>(&mut self, predicate: P) -> Presence<T>
    where
        P: FnOnce(&T) -> bool,
    {
        match self {
            Presence::Some(val) if predicate(val) => self.take(),
            _ => Presence::Absent,
        }
    }

    /// Replaces the actual value in the `Presence` by the value given in parameter,
    /// returning the old value if present, leaving a [`Some`] in its place.
    ///
    /// [`Some`]: Presence::Some
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let mut x = Presence::Some(2);
    /// let old = x.replace(5);
    /// assert_eq!(x, Presence::Some(5));
    /// assert_eq!(old, Presence::Some(2));
    ///
    /// let mut y = Presence::Null;
    /// let old = y.replace(3);
    /// assert_eq!(y, Presence::Some(3));
    /// assert_eq!(old, Presence::Null);
    ///
    /// let mut z: Presence<i32> = Presence::Absent;
    /// let old = z.replace(7);
    /// assert_eq!(z, Presence::Some(7));
    /// assert_eq!(old, Presence::Absent);
    /// ```
    #[inline]
    pub fn replace(&mut self, value: T) -> Presence<T> {
        core::mem::replace(self, Presence::Some(value))
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
    /// use presence_rs::Presence;
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
    /// use presence_rs::Presence;
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
    /// use presence_rs::Presence;
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
    /// use presence_rs::Presence;
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

    /// Maps a `Presence<T>` to `Presence<U>` by applying a function to a contained value.
    ///
    /// State rule: [self decides](mod@crate::presence#combining-states): a `Null` or `Absent` receiver keeps its state.
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
    #[must_use = "Returns the mapped value"]
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
    /// use presence_rs::Presence;
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
    /// use presence_rs::Presence;
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
    #[must_use = "Returns the mapped value or default"]
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
    /// use presence_rs::Presence;
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
    #[must_use = "Returns the mapped value or computed default"]
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
    /// use presence_rs::Presence;
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

    /// Returns [`Absent`] or [`Null`] if the presence is [`Absent`] or [`Null`], otherwise returns `optb`.
    ///
    /// State rule: [self decides](mod@crate::presence#combining-states): a `Null` or `Absent` receiver keeps its state.
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
    #[must_use = "Returns the logical AND result"]
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
    /// State rule: [self decides](mod@crate::presence#combining-states): a `Null` or `Absent` receiver keeps its state.
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
    /// use presence_rs::Presence;
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
    #[must_use = "Returns the result of the closure"]
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
    /// State rule: [self decides](mod@crate::presence#combining-states): a `Null` or `Absent` receiver keeps its state.
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// A `Some` that fails the predicate becomes `Absent`, not `Null`: in PATCH terms
    /// `Absent` means "leave the field unchanged", the safe default, while `Null` would
    /// mean "clear it".
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
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
    #[must_use = "Returns the filtered value"]
    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: FnOnce(&T) -> bool,
    {
        match self {
            Presence::Some(ref val) if predicate(val) => self,
            Presence::Some(_) | Presence::Absent => Presence::Absent,
            Presence::Null => Presence::Null,
        }
    }

    /// Returns the presence if it contains a value, otherwise returns `optb`.
    ///
    /// State rule: [first `Some` wins](mod@crate::presence#combining-states): when `self` isn't `Some`, `optb` is returned whatever its state.
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
    /// use presence_rs::Presence;
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
    #[must_use = "Returns the logical OR result"]
    pub fn or(self, optb: Presence<T>) -> Presence<T> {
        match self {
            Presence::Some(_) => self,
            Presence::Null | Presence::Absent => optb,
        }
    }

    /// Returns the presence if it contains a value, otherwise calls `f` and
    /// returns the result.
    ///
    /// State rule: [first `Some` wins](mod@crate::presence#combining-states): when `self` isn't `Some`, the result of `f` is returned whatever its state.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
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
    #[must_use = "Returns the value or computed alternative"]
    pub fn or_else<F>(self, f: F) -> Presence<T>
    where
        F: FnOnce() -> Presence<T>,
    {
        match self {
            Presence::Some(_) => self,
            Presence::Null | Presence::Absent => f(),
        }
    }

    /// Composes two patches: returns `later`, unless it is [`Absent`], in which case
    /// returns `self`.
    ///
    /// State rule: [last non-`Absent` wins](mod@crate::presence#combining-states). Applying
    /// `earlier.merge(later)` with [`apply_to`](Presence::apply_to) leaves the target in
    /// the same state as applying `earlier` and then `later`, so a series of PATCH
    /// requests can be folded into one (the value `apply_to` returns can differ). `Absent`
    /// changes nothing on either side, and `merge` is associative.
    ///
    /// [`Absent`]: Presence::Absent
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let set = Presence::Some(1);
    /// let clear: Presence<i32> = Presence::Null;
    /// let keep: Presence<i32> = Presence::Absent;
    ///
    /// assert_eq!(set.merge(clear), Presence::Null);
    /// assert_eq!(clear.merge(set), Presence::Some(1));
    /// assert_eq!(set.merge(keep), Presence::Some(1));
    ///
    /// let folded = [set, keep, clear, Presence::Some(3), keep]
    ///     .into_iter()
    ///     .fold(Presence::Absent, Presence::merge);
    /// assert_eq!(folded, Presence::Some(3));
    /// ```
    #[inline]
    #[must_use = "Returns the composed patch"]
    pub fn merge(self, later: Presence<T>) -> Presence<T> {
        match later {
            Presence::Absent => self,
            later => later,
        }
    }

    /// Returns [`Some`] if exactly one of `self`, `optb` is [`Some`], otherwise returns [`Absent`] or [`Null`].
    ///
    /// [`Some`]: Presence::Some
    /// [`Null`]: Presence::Null
    /// [`Absent`]: Presence::Absent
    ///
    /// State rule: [`xor`](mod@crate::presence#combining-states). Two `Some`s give `Absent`, not `Null`:
    /// in PATCH terms `Absent` means "leave the field unchanged", the safe default, while
    /// `Null` would mean "clear it".
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
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
    #[must_use = "Returns the logical XOR result"]
    pub fn xor(self, optb: Presence<T>) -> Presence<T> {
        match (self, optb) {
            (Presence::Some(a), Presence::Null | Presence::Absent) => Presence::Some(a),
            (Presence::Null | Presence::Absent, Presence::Some(b)) => Presence::Some(b),
            (Presence::Some(_), Presence::Some(_))
            | (Presence::Absent, _)
            | (_, Presence::Absent) => Presence::Absent,
            (Presence::Null, Presence::Null) => Presence::Null,
        }
    }

    /// Zips `self` with another `Presence`.
    ///
    /// If `self` is `Some(s)` and `other` is `Some(o)`, this method returns `Some((s, o))`.
    /// Otherwise it returns `Absent` if either is `Absent`, and `Null` if neither is
    /// `Absent` but at least one is `Null`.
    ///
    /// State rule: [`Absent` over `Null` over `Some`](mod@crate::presence#combining-states): the state of
    /// the result does not depend on the order of the arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x = Presence::Some(1);
    /// let y = Presence::Some("hi");
    /// let z: Presence<i32> = Presence::Null;
    ///
    /// assert_eq!(x.zip(y), Presence::Some((1, "hi")));
    /// assert_eq!(x.zip(z), Presence::Null);
    ///
    /// let a: Presence<i32> = Presence::Absent;
    /// let b = Presence::Some("hello");
    /// assert_eq!(a.zip(b), Presence::Absent);
    ///
    /// let c: Presence<i32> = Presence::Null;
    /// let d: Presence<&str> = Presence::Null;
    /// assert_eq!(c.zip(d), Presence::Null);
    /// ```
    #[inline]
    #[must_use = "this returns the zipped tuple, without modifying the originals"]
    pub fn zip<U>(self, other: Presence<U>) -> Presence<(T, U)> {
        match (self, other) {
            (Presence::Some(a), Presence::Some(b)) => Presence::Some((a, b)),
            (Presence::Absent, _) | (_, Presence::Absent) => Presence::Absent,
            (Presence::Null, _) | (_, Presence::Null) => Presence::Null,
        }
    }

    /// Zips `self` and another `Presence` with function `f`.
    ///
    /// State rule: [`Absent` over `Null` over `Some`](mod@crate::presence#combining-states), as for
    /// [`zip`](Presence::zip).
    ///
    /// If `self` is `Some(s)` and `other` is `Some(o)`, this method returns `Some(f(s, o))`.
    /// Otherwise it returns `Absent` if either is `Absent`, and `Null` if neither is
    /// `Absent` but at least one is `Null`.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Point {
    ///     x: f64,
    ///     y: f64,
    /// }
    ///
    /// impl Point {
    ///     fn new(x: f64, y: f64) -> Self {
    ///         Point { x, y }
    ///     }
    /// }
    ///
    /// let x = Presence::Some(17.5);
    /// let y = Presence::Some(42.7);
    ///
    /// assert_eq!(x.zip_with(y, Point::new), Presence::Some(Point { x: 17.5, y: 42.7 }));
    ///
    /// let z: Presence<f64> = Presence::Null;
    /// assert_eq!(x.zip_with(z, Point::new), Presence::Null);
    ///
    /// let a: Presence<f64> = Presence::Absent;
    /// assert_eq!(a.zip_with(y, Point::new), Presence::Absent);
    /// ```
    #[inline]
    pub fn zip_with<U, F, R>(self, other: Presence<U>, f: F) -> Presence<R>
    where
        F: FnOnce(T, U) -> R,
    {
        self.zip(other).map(|(a, b)| f(a, b))
    }

    /// Combines `self` and another `Presence` with function `f`; the same as
    /// [`zip_with`](Presence::zip_with), including its state rule.
    #[deprecated(since = "0.3.0", note = "use `zip_with`, which does the same")]
    #[inline]
    pub fn reduce<U, R, F>(self, other: Presence<U>, f: F) -> Presence<R>
    where
        F: FnOnce(T, U) -> R,
    {
        self.zip_with(other, f)
    }

    /// Unzips a presence containing a tuple of two values.
    ///
    /// State rule: [self decides](mod@crate::presence#combining-states): a `Null` or `Absent` receiver keeps its state.
    ///
    /// If `self` is `Some((a, b))`, this method returns `(Some(a), Some(b))`.
    /// Otherwise, returns `(Null, Null)` if `self` is `Null`, or `(Absent, Absent)` if `self` is `Absent`.
    ///
    /// # Examples
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x = Presence::Some((1, "hi"));
    /// let y: Presence<(i32, &str)> = Presence::Null;
    /// let z: Presence<(i32, &str)> = Presence::Absent;
    ///
    /// assert_eq!(x.unzip(), (Presence::Some(1), Presence::Some("hi")));
    /// assert_eq!(y.unzip(), (Presence::Null, Presence::Null));
    /// assert_eq!(z.unzip(), (Presence::Absent, Presence::Absent));
    /// ```
    #[inline]
    pub fn unzip<A, B>(self) -> (Presence<A>, Presence<B>)
    where
        T: Into<(A, B)>,
    {
        match self {
            Presence::Some(val) => {
                let (a, b) = val.into();
                (Presence::Some(a), Presence::Some(b))
            }
            Presence::Null => (Presence::Null, Presence::Null),
            Presence::Absent => (Presence::Absent, Presence::Absent),
        }
    }
}

impl<T> Presence<Presence<T>> {
    /// Converts from `Presence<Presence<T>>` to `Presence<T>`.
    ///
    /// State rule: [self decides](mod@crate::presence#combining-states): a `Null` or `Absent` receiver keeps its state.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x: Presence<Presence<i32>> = Presence::Some(Presence::Some(6));
    /// assert_eq!(Presence::Some(6), x.flatten());
    ///
    /// let x: Presence<Presence<i32>> = Presence::Some(Presence::Null);
    /// assert_eq!(Presence::Null, x.flatten());
    ///
    /// let x: Presence<Presence<i32>> = Presence::Some(Presence::Absent);
    /// assert_eq!(Presence::Absent, x.flatten());
    ///
    /// let x: Presence<Presence<i32>> = Presence::Null;
    /// assert_eq!(Presence::Null, x.flatten());
    ///
    /// let x: Presence<Presence<i32>> = Presence::Absent;
    /// assert_eq!(Presence::Absent, x.flatten());
    /// ```
    ///
    /// Flattening multiple layers:
    ///
    /// ```
    /// use presence_rs::Presence;
    ///
    /// let x: Presence<Presence<Presence<i32>>> = Presence::Some(Presence::Some(Presence::Some(6)));
    /// assert_eq!(Presence::Some(Presence::Some(6)), x.flatten());
    /// assert_eq!(Presence::Some(6), x.flatten().flatten());
    /// ```
    #[inline]
    #[must_use = "Returns the flattened value"]
    pub fn flatten(self) -> Presence<T> {
        match self {
            Presence::Some(inner) => inner,
            Presence::Null => Presence::Null,
            Presence::Absent => Presence::Absent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Presence;
    use proptest::prelude::*;

    fn any_presence() -> impl Strategy<Value = Presence<i32>> {
        prop_oneof![
            any::<i32>().prop_map(Presence::Some),
            Just(Presence::Null),
            Just(Presence::Absent),
        ]
    }

    proptest! {
        #[test]
        fn unwrap_or_absent_or_null_with_one_default_is_unwrap_or(
            presence in any_presence(),
            default in any::<i32>(),
        ) {
            prop_assert_eq!(
                presence.unwrap_or_absent_or_null(default, default),
                presence.unwrap_or(default)
            );
        }
    }
}
