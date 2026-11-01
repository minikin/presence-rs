//! Derive macro for [`presence-rs`](https://docs.rs/presence-rs).
//!
//! Use it through `presence_rs::Patch` with presence-rs's `derive` feature, not as
//! a dependency of its own: the generated code names presence-rs items.

mod attrs;
mod expand;

use proc_macro::TokenStream;

/// Derives a patch struct and its `presence_rs::Patch` impl for a struct with named fields.
///
/// A model field of type `Option<T>` becomes a `Presence<T>` in the patch, and any other
/// field type `T` an `Option<T>`. The patch struct is named after the model with a
/// `Patch` suffix, keeps the model's generics and visibilities, copies each field's docs,
/// and implements `Default`.
///
/// Options, on the model:
///
/// - `#[patch(name = AccountUpdate)]` names the patch struct.
/// - `#[patch(derive(Debug, Clone, Serialize, Deserialize, JsonSchema))]` adds derives.
///   Serde derives add the serde attributes a patch needs and carry over the model's
///   `rename_all`, `deny_unknown_fields`, `rename` and `alias`. `JsonSchema` gives a
///   schema that matches what the patch accepts.
/// - `#[patch(crate = path)]` names the path to presence-rs.
///
/// On a field:
///
/// - `#[patch(skip)]` leaves the field out of the patch.
///
/// See the `presence_rs::Patch` trait for an example.
#[proc_macro_derive(Patch, attributes(patch))]
pub fn derive_patch(input: TokenStream) -> TokenStream {
    expand::expand(input.into()).into()
}
