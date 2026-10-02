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
//! The crate is `#![no_std]` and needs no allocator, with or without the `serde` feature.
//!
//! # Serde
//!
//! With the `serde` feature, `Presence<T>` implements `Serialize` and `Deserialize`.
//! `Some(value)` is written as the value; `Null` and `Absent` are both written as `null`.
//! When read back, a value becomes `Some` and `null` becomes `Null`.
//!
//! A struct field keeps all three states apart only when it has **both** attributes:
//!
//! - `#[serde(default)]` makes a missing field `Absent`. Without it, a missing `Presence`
//!   field is read exactly like `null` (the same rule serde applies to `Option`), so it
//!   silently becomes `Null`.
//! - `#[serde(skip_serializing_if = "Presence::is_absent")]` leaves `Absent` out of the
//!   output. Without it, `Absent` is written as `null` and reads back as `Null`.
//!
//! ```
//! # #[cfg(feature = "serde")] {
//! use presence_rs::Presence;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! struct Patch {
//!     #[serde(default, skip_serializing_if = "Presence::is_absent")]
//!     nickname: Presence<String>,
//! }
//!
//! // Deserializing tells the three states apart.
//! let set: Patch = serde_json::from_str(r#"{"nickname":"neo"}"#).unwrap();
//! let cleared: Patch = serde_json::from_str(r#"{"nickname":null}"#).unwrap();
//! let untouched: Patch = serde_json::from_str("{}").unwrap();
//! assert_eq!(set.nickname, Presence::Some("neo".to_string()));
//! assert_eq!(cleared.nickname, Presence::Null);
//! assert_eq!(untouched.nickname, Presence::Absent);
//!
//! // Serializing writes each one back the way it came in.
//! assert_eq!(serde_json::to_string(&set).unwrap(), r#"{"nickname":"neo"}"#);
//! assert_eq!(serde_json::to_string(&cleared).unwrap(), r#"{"nickname":null}"#);
//! assert_eq!(serde_json::to_string(&untouched).unwrap(), "{}");
//! # }
//! ```
//!
//! [`Presence<T>`]: presence::Presence

#![no_std]

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

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
