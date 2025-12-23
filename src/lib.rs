//! Three-valued logic for Rust: distinguishing between absent, null, and present values.
//!
//! This crate provides the [`Presence<T>`] type, a three-valued alternative to [`Option<T>`]
//! that distinguishes between "field not present" and "field present but null".
//!
//! # The Three States
//!
//! - **`Absent`**: Field doesn't exist in the data structure (`{}` in JSON)
//! - **`Null`**: Field exists but is explicitly null (`{"field": null}` in JSON)
//! - **`Some(value)`**: Field exists with a concrete value (`{"field": 42}` in JSON)
//!
//! # When to Use This
//!
//! Use `Presence<T>` when you need to distinguish between absence and null:
//!
//! - JSON/API responses where `{}`, `{"field": null}`, and `{"field": value}` are semantically different
//! - IPLD schemas where absent and null fields have distinct meanings
//! - Database operations where NULL and missing columns differ
//! - Form data where unchecked differs from explicitly cleared
//!
//! # Quick Example
//!
//! ```
//! use presence_rs::Presence;
//!
//! let present = Presence::Some(42);
//! let null = Presence::<i32>::Null;
//! let absent = Presence::<i32>::Absent;
//!
//! assert!(present.is_present());
//! assert!(null.is_defined());      // Exists in structure (even though null)
//! assert!(!absent.is_defined());   // Doesn't exist in structure
//!
//! // Transformations preserve null vs absent
//! assert_eq!(null.map(|x| x * 2), Presence::Null);
//! ```
//!
//! See the [`mod@presence`] module for detailed documentation and examples.
//!
//! [`Presence<T>`]: presence::Presence

pub mod presence;
pub use presence::Presence;

#[cfg(feature = "serde")]
mod serde;

/// Convenience macro for creating [`Presence`] values.
///
/// This macro provides a concise syntax for constructing `Presence` values,
/// similar to how `vec![]` works for `Vec`.
///
/// [`Presence`]: presence::Presence
///
/// # Syntax
///
/// - `presence!()` - Creates `Presence::Absent`
/// - `presence!(null)` - Creates `Presence::Null`
/// - `presence!(value)` - Creates `Presence::Some(value)`
///
/// # Examples
///
/// ```
/// use presence_rs::presence;
///
/// let absent: presence::Presence<i32> = presence!();
/// assert_eq!(absent, presence::Presence::Absent);
///
/// let null: presence::Presence<i32> = presence!(null);
/// assert_eq!(null, presence::Presence::Null);
///
/// let some = presence!(42);
/// assert_eq!(some, presence::Presence::Some(42));
///
/// // Works with any expression
/// let computed = presence!(2 + 2);
/// assert_eq!(computed, presence::Presence::Some(4));
///
/// let owned = presence!("hello".to_string());
/// assert_eq!(owned, presence::Presence::Some("hello".to_string()));
/// ```
#[macro_export]
macro_rules! presence {
    () => {
        $crate::presence::Presence::Absent
    };
    (null) => {
        $crate::presence::Presence::Null
    };
    ($value:expr) => {
        $crate::presence::Presence::Some($value)
    };
}
