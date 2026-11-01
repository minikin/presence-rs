//! The [`Patch`] trait, implemented by `#[derive(Patch)]`.

/// A partial update of a model of type [`Target`](Patch::Target).
///
/// `#[derive(Patch)]`, behind the `derive` feature, generates a patch struct for a
/// model struct and implements this trait for it. A model field of type `Option<T>`
/// becomes a [`Presence<T>`](crate::Presence) in the patch, and any other field type
/// `T` becomes an `Option<T>`.
///
/// Patches form a monoid under [`merge`](Patch::merge) with [`Default`] as the
/// identity: the default patch sets nothing, and applying `a.merge(b)` equals applying
/// `a` and then `b`.
///
/// ```
/// # #[cfg(feature = "derive")] {
/// use presence_rs::{Patch, Presence};
///
/// #[derive(Debug, PartialEq, Patch)]
/// struct User {
///     name: String,
///     nickname: Option<String>,
/// }
///
/// let rename = UserPatch {
///     name: Some("Alicia".to_string()),
///     nickname: Presence::Absent,
/// };
/// let clear_nickname = UserPatch {
///     name: None,
///     nickname: Presence::Null,
/// };
/// assert!(UserPatch::default().is_empty());
///
/// let mut user = User {
///     name: "Alice".to_string(),
///     nickname: Some("al".to_string()),
/// };
/// rename.merge(clear_nickname).apply_to(&mut user);
/// assert_eq!(
///     user,
///     User {
///         name: "Alicia".to_string(),
///         nickname: None,
///     }
/// );
/// # }
/// ```
///
/// The README's "Deriving a patch" section lists the derive's options and limits.
pub trait Patch: Default {
    /// The model this patch updates.
    type Target;

    /// Applies the patch to `target`.
    ///
    /// A `Presence<T>` field follows [`Presence::apply_to`](crate::Presence::apply_to):
    /// `Absent` leaves the model's field alone, `Null` clears it and `Some` sets it. An
    /// `Option<T>` field replaces the model's field when it is `Some` and leaves it
    /// alone when it is `None`.
    fn apply_to(self, target: &mut Self::Target);

    /// Composes two patches into one, field by field. The field of `later` wins unless
    /// it is unset, as in [`Presence::merge`](crate::Presence::merge).
    #[must_use]
    fn merge(self, later: Self) -> Self;

    /// Returns `true` when no field is set, so applying the patch changes nothing.
    fn is_empty(&self) -> bool;
}
