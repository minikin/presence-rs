//! Three-valued presence semantics for optional and nullable fields.
//!
//! This module provides the [`Presence<T>`] type, a three-valued alternative to Rust's
//! two-valued [`Option<T>`]. Where `Option` distinguishes between "some value" and "no value",
//! `Presence` adds a third state to distinguish between "field not present" and "field
//! present but null". This is particularly useful for:
//!
//! - JSON/IPLD schema validation where `{}`, `{"field": null}`, and `{"field": value}` are distinct
//! - API responses where missing fields have different semantics than explicit nulls
//! - Database operations where NULL and absence have different meanings
//! - Form data where unchecked boxes differ from explicitly set null values
//!
//! # Three States
//!
//! [`Presence<T>`] has three variants:
//!
//! - [`Absent`]: The field/key is not present in the data structure
//!   - JSON: `{}` (field omitted)
//!   - Semantics: Field was never set, doesn't exist in the structure
//!
//! - [`Null`]: The field/key is present but the value is explicitly null
//!   - JSON: `{"field": null}`
//!   - Semantics: Field exists but was explicitly set to null
//!
//! - [`Some(T)`]: The field/key is present with a concrete value
//!   - JSON: `{"field": value}`
//!   - Semantics: Field exists with a meaningful value
//!
//! [`Absent`]: Presence::Absent
//! [`Null`]: Presence::Null
//! [`Some(T)`]: Presence::Some
//!
//! # Comparison with Option
//!
//! While [`Option<Option<T>>`] can represent three states, [`Presence<T>`] provides:
//!
//! - **Clearer semantics**: Named variants instead of nested Options
//! - **Better ergonomics**: Single-level matching instead of nested patterns
//! - **Rich API**: Methods designed for three-valued logic
//! - **IPLD compatibility**: Direct support for IPLD schema semantics
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use presence_rs::Presence;
//!
//! // Creating Presence values
//! let present = Presence::Some(42);
//! let null = Presence::<i32>::Null;
//! let absent = Presence::<i32>::Absent;
//!
//! // Pattern matching
//! match present {
//!     Presence::Some(value) => println!("Got value: {}", value),
//!     Presence::Null => println!("Explicitly null"),
//!     Presence::Absent => println!("Not present"),
//! }
//!
//! // Using query methods
//! assert!(present.is_present());
//! assert!(null.is_defined());      // Null is "defined" (exists in structure)
//! assert!(!absent.is_defined());   // Absent is not defined
//! assert!(null.is_nullish());      // Both Null and Absent are "nullish"
//! ```
//!
//! ## Functional Transformations
//!
//! ```
//! use presence_rs::Presence;
//!
//! let x = Presence::Some(5);
//!
//! // Map transforms Some, preserves Null and Absent
//! assert_eq!(x.map(|v| v * 2), Presence::Some(10));
//!
//! let null: Presence<i32> = Presence::Null;
//! assert_eq!(null.map(|v| v * 2), Presence::Null);
//!
//! // Chaining operations
//! let result = Presence::Some(5)
//!     .map(|x| x * 2)
//!     .filter(|x| x > &5)
//!     .unwrap_or(0);
//! assert_eq!(result, 10);
//! ```
//!
//! ## Conversions
//!
//! ```
//! use presence_rs::Presence;
//!
//! // From Option<Option<T>> (nullable representation)
//! let nested: Option<Option<i32>> = Some(None);
//! let presence: Presence<i32> = nested.into();
//! assert_eq!(presence, Presence::Null);
//!
//! // To Option<Option<T>>
//! let back: Option<Option<i32>> = presence.into();
//! assert_eq!(back, Some(None));
//!
//! // From/to Option<T> (optional representation)
//! let opt = Some(42);
//! let p = Presence::from_optional(opt);
//! assert_eq!(p, Presence::Some(42));
//!
//! let opt2 = p.to_optional();
//! assert_eq!(opt2, Some(42));
//! ```
//!
//! ## Working with Collections
//!
//! ```
//! use presence_rs::Presence;
//!
//! // Collecting - stops at the first Absent; after a Null it still reads on, looking for an Absent
//! let values = vec![Presence::Some(1), Presence::Some(2), Presence::Some(3)];
//! let result: Presence<Vec<i32>> = values.into_iter().collect();
//! assert_eq!(result, Presence::Some(vec![1, 2, 3]));
//!
//! let with_null = vec![Presence::Some(1), Presence::Null, Presence::Some(3)];
//! let result: Presence<Vec<i32>> = with_null.into_iter().collect();
//! assert_eq!(result, Presence::Null);
//!
//! // Sum and Product
//! let nums = vec![Presence::Some(1), Presence::Some(2), Presence::Some(3)];
//! let sum: Presence<i32> = nums.into_iter().sum();
//! assert_eq!(sum, Presence::Some(6));
//! ```
//!
//! ## IPLD Schema Semantics
//!
//! ```
//! use presence_rs::Presence;
//!
//! // Check if field is defined (exists in structure)
//! let null: Presence<i32> = Presence::Null;
//! assert!(null.is_defined());  // true - field exists even though null
//!
//! let absent: Presence<i32> = Presence::Absent;
//! assert!(!absent.is_defined());  // false - field doesn't exist
//!
//! // Different defaults for null vs absent
//! assert_eq!(null.unwrap_or_absent_or_null(1, 2), 2);
//! assert_eq!(absent.unwrap_or_absent_or_null(1, 2), 1);
//! ```
//!
//! # Combining states
//!
//! The methods below follow one of these rules for `Null` and `Absent`:
//!
//! | Rule | Methods | Behavior |
//! | --- | --- | --- |
//! | Self decides | [`map`], [`and`], [`and_then`], [`filter`], [`flatten`], [`unzip`], [`transpose`], `copied`, `cloned` | A `Null` or `Absent` receiver keeps its state; only `Some` looks at the closure or the argument. |
//! | First `Some` wins | [`or`], [`or_else`] | `self` if it is `Some`, otherwise the alternative, whatever its state. |
//! | `Absent` over `Null` over `Some` | [`zip`], [`zip_with`], `collect`, `sum`, `product` | `Absent` if any input is `Absent`, otherwise `Null` if any is `Null`, otherwise `Some`. The state of the result does not depend on the order of the inputs. |
//! | Last non-`Absent` wins | [`merge`] | `later` unless it is `Absent`, in which case `self`; composes PATCH requests. |
//! | `xor` | [`xor`] | The `Some` side if exactly one side is `Some`; `Null` if both are `Null`; otherwise `Absent`, including for two `Some`s. |
//!
//! So `Null.and(Absent)` is `Null` and `Null.or(Absent)` is `Absent`, while
//! `Null.zip(Absent)` is `Absent` whichever side each is on.
//!
//! Where a method has to invent a "nothing" result — [`filter`] rejecting a value,
//! [`xor`] of two `Some`s, [`take_if`] taking nothing — it returns `Absent`. In PATCH terms
//! `Absent` means "leave the field unchanged", the safe default; `Null` would mean "clear
//! it".
//!
//! [`map`]: Presence::map
//! [`and`]: Presence::and
//! [`and_then`]: Presence::and_then
//! [`filter`]: Presence::filter
//! [`flatten`]: Presence::flatten
//! [`unzip`]: Presence::unzip
//! [`transpose`]: Presence::transpose
//! [`or`]: Presence::or
//! [`or_else`]: Presence::or_else
//! [`zip`]: Presence::zip
//! [`zip_with`]: Presence::zip_with
//! [`xor`]: Presence::xor
//! [`merge`]: Presence::merge
//! [`take_if`]: Presence::take_if
//!
//! # Patching
//!
//! In a PATCH request a field left out means "leave it alone", an explicit `null` means
//! "clear it", and a value means "set it". [`apply_to`] applies a presence to an
//! `Option` field of your model with exactly those semantics:
//!
//! ```
//! use presence_rs::Presence;
//!
//! struct User {
//!     email: Option<String>,
//!     phone: Option<String>,
//! }
//!
//! struct UserPatch {
//!     email: Presence<String>,
//!     phone: Presence<String>,
//! }
//!
//! let mut user = User {
//!     email: Some("old@example.com".to_string()),
//!     phone: Some("555-0100".to_string()),
//! };
//! let patch = UserPatch {
//!     email: Presence::Some("new@example.com".to_string()),
//!     phone: Presence::Null,
//! };
//!
//! patch.email.apply_to(&mut user.email);
//! patch.phone.apply_to(&mut user.phone);
//!
//! assert_eq!(user.email.as_deref(), Some("new@example.com"));
//! assert_eq!(user.phone, None);
//! ```
//!
//! [`apply_to`]: Presence::apply_to
//!
//! # Cardinality
//!
//! For a base type with `N` possible values, `Presence<T>` provides `N + 2` states:
//!
//! - `N` states from `Some(value)` where value has type `T`
//! - `1` state from `Null` (explicitly null)
//! - `1` state from `Absent` (not present)
//!
//! For example, `Presence<bool>` has 4 states: `Some(true)`, `Some(false)`, `Null`, `Absent`.
//!
//! # API Organization
//!
//! The API is organized into several categories:
//!
//! - **Querying**: `is_absent()`, `is_null()`, `is_present()`, `is_defined()`, `is_nullish()`,
//!   `is_some_and()`, `is_absent_or()`, `is_nullish_or()`
//! - **Extracting**: `expect()`, `unwrap()`, `unwrap_or()`, `unwrap_or_default()`
//! - **Transforming**: `map()`, `filter()`, `and_then()`, `flatten()`
//! - **Combining**: `and()`, `or()`, `xor()`, `zip()`, `zip_with()`
//! - **Converting**: `to_optional()`, `to_nullable()`, `from_optional()`, `from_nullable()`
//! - **References**: `as_ref()`, `as_mut()`, `as_deref()`, `copied()`, `cloned()`
//! - **Iterating**: `iter()`, `iter_mut()`, `into_iter()`

mod convert;
mod iter;
mod query;
mod refs;
mod transform;

use core::fmt;

pub use iter::{IntoIter, Iter, IterMut};

/// A value that is absent, explicitly null, or present.
///
/// See the [module documentation](self) for the three states and the full API.
///
/// # Ordering
///
/// Presences are ordered `Absent < Null < Some(_)`, and two `Some` values compare by
/// their contents. This order is part of the public API: it will not change without a
/// breaking release.
///
/// ```
/// use presence_rs::Presence;
///
/// assert!(Presence::<i32>::Absent < Presence::Null);
/// assert!(Presence::Null < Presence::Some(i32::MIN));
/// assert!(Presence::Some(1) < Presence::Some(2));
/// ```
///
/// # Serde
///
/// With the `serde` feature, a struct field of this type keeps all three states apart
/// only with both `#[serde(default)]` and
/// `#[serde(skip_serializing_if = "Presence::is_absent")]`. Without `default`, a missing
/// field deserializes as `Null`. See the [crate-level Serde section](crate#serde).
#[must_use = "`Presence` may contain a value that should be used"]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Presence<T> {
    /// Field/key is absent from the structure
    #[doc(alias = "undefined")]
    #[doc(alias = "missing")]
    #[doc(alias = "none")]
    #[doc(alias = "empty")]
    Absent,
    /// Field/key is present but the value is null
    #[doc(alias = "nil")]
    #[doc(alias = "nothing")]
    #[doc(alias = "void")]
    Null,
    /// Field/key is present with a concrete value
    #[doc(alias = "value")]
    #[doc(alias = "present")]
    #[doc(alias = "defined")]
    #[doc(alias = "just")]
    Some(T),
}

/// The former name of [`IntoIter`].
#[deprecated(since = "0.3.0", note = "renamed to `IntoIter`")]
pub type Item<A> = IntoIter<A>;

/// Display implementation
impl<T: fmt::Display> fmt::Display for Presence<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Presence::Absent => write!(f, "(absent)"),
            Presence::Null => write!(f, "null"),
            Presence::Some(val) => write!(f, "{val}"),
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
    /// use presence_rs::Presence;
    ///
    /// let x: Presence<i32> = Default::default();
    /// assert_eq!(x, Presence::Absent);
    /// ```
    fn default() -> Presence<T> {
        Presence::Absent
    }
}
