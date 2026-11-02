//! Acceptance tests for spec 09: one test per scenario, named after the scenario.

use presence_rs::{Patch, Presence};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use std::fmt::Debug;

/// Compiler diagnostics change between releases, so the checked-in `.stderr`
/// files are compared on stable only.
fn compile_fail(path: &str) {
    if rustversion::cfg!(stable) {
        trybuild::TestCases::new().compile_fail(path);
    }
}

fn persisted_config() -> ProptestConfig {
    ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "proptest-regressions",
        ))),
        ..ProptestConfig::default()
    }
}

#[derive(Patch, Debug, Clone, PartialEq)]
struct User {
    id: u64,
    name: String,
    nickname: Option<String>,
}

fn any_user() -> impl Strategy<Value = User> {
    (any::<u64>(), any::<String>(), any::<Option<String>>()).prop_map(|(id, name, nickname)| User {
        id,
        name,
        nickname,
    })
}

fn any_string_presence() -> impl Strategy<Value = Presence<String>> {
    prop_oneof![
        any::<String>().prop_map(Presence::Some),
        Just(Presence::Null),
        Just(Presence::Absent),
    ]
}

/// The fields of a `UserPatch`. Strategies yield these rather than the patch,
/// because a patch derives nothing beyond what the model asks for, and proptest
/// needs `Debug` and `Clone` on its values.
type UserPatchFields = (Option<u64>, Option<String>, Presence<String>);

fn any_user_patch() -> impl Strategy<Value = UserPatchFields> {
    (
        any::<Option<u64>>(),
        any::<Option<String>>(),
        any_string_presence(),
    )
}

fn user_patch((id, name, nickname): UserPatchFields) -> UserPatch {
    UserPatch { id, name, nickname }
}

fn fields(patch: UserPatch) -> UserPatchFields {
    (patch.id, patch.name, patch.nickname)
}

fn is_unset(patch: &UserPatchFields) -> bool {
    patch.0.is_none() && patch.1.is_none() && patch.2.is_absent()
}

fn apply(patch: UserPatch, user: &User) -> User {
    let mut patched = user.clone();
    patch.apply_to(&mut patched);
    patched
}

// Spec 09: applying fields

proptest! {
    #![proptest_config(persisted_config())]

    #[test]
    fn an_unset_nullable_field_leaves_the_models_field_unchanged(
        user in any_user(),
        patch in any_user_patch(),
    ) {
        // Given any User and a UserPatch whose nickname is Presence::Absent
        let patch = UserPatch { nickname: Presence::Absent, ..user_patch(patch) };
        // When the patch is applied to the user
        let patched = apply(patch, &user);
        // Then the user's nickname is unchanged
        prop_assert_eq!(patched.nickname, user.nickname);
    }

    #[test]
    fn a_null_nullable_field_clears_the_models_field(
        user in any_user(),
        patch in any_user_patch(),
    ) {
        // Given any User and a UserPatch whose nickname is Presence::Null
        let patch = UserPatch { nickname: Presence::Null, ..user_patch(patch) };
        // When the patch is applied to the user
        let patched = apply(patch, &user);
        // Then the user's nickname is None
        prop_assert_eq!(patched.nickname, None);
    }

    #[test]
    fn a_set_nullable_field_sets_the_models_field(
        user in any_user(),
        patch in any_user_patch(),
        s in any::<String>(),
    ) {
        // Given any User, any string s and a UserPatch whose nickname is Presence::Some(s)
        let patch = UserPatch { nickname: Presence::Some(s.clone()), ..user_patch(patch) };
        // When the patch is applied to the user
        let patched = apply(patch, &user);
        // Then the user's nickname is Some(s)
        prop_assert_eq!(patched.nickname, Some(s));
    }

    #[test]
    fn a_required_field_is_replaced_only_when_set(
        user in any_user(),
        patch in any_user_patch(),
        name in any::<Option<String>>(),
    ) {
        // Given any User and a UserPatch whose name is any Option<String>
        let patch = UserPatch { name: name.clone(), ..user_patch(patch) };
        // When the patch is applied to the user
        let patched = apply(patch, &user);
        // Then the user's name is the set value if name is Some, and otherwise unchanged
        prop_assert_eq!(patched.name, name.unwrap_or(user.name));
    }

    #[test]
    fn generic_code_accepts_any_derived_patch(
        user in any_user(),
        patch in any_user_patch(),
    ) {
        // Given a function generic over P: Patch that applies P to a P::Target
        fn apply_any<P: Patch>(patch: P, target: &mut P::Target) {
            patch.apply_to(target);
        }
        // When it is called with a UserPatch and a User
        let mut generic = user.clone();
        apply_any(user_patch(patch.clone()), &mut generic);
        // Then the user is patched as if UserPatch::apply_to had been called
        prop_assert_eq!(generic, apply(user_patch(patch), &user));
    }
}

// Spec 09: the empty patch and merging

proptest! {
    #![proptest_config(persisted_config())]

    #[test]
    fn the_default_patch_is_empty_and_changes_nothing(user in any_user()) {
        // Given any User
        // When UserPatch::default() is applied to it
        let patched = apply(UserPatch::default(), &user);
        // Then the user is unchanged
        prop_assert_eq!(patched, user);
        // And UserPatch::default().is_empty() is true
        prop_assert!(UserPatch::default().is_empty());
    }

    #[test]
    fn is_empty_is_true_exactly_when_every_field_is_unset(
        patch in any_user_patch(),
        user in any_user(),
    ) {
        // Given any UserPatch
        let expected = is_unset(&patch);
        // When is_empty is called
        let empty = user_patch(patch.clone()).is_empty();
        // Then it is true if name is None and nickname is Absent (and id is None), false otherwise
        prop_assert_eq!(empty, expected);
        // And an empty patch changes no user
        if empty {
            prop_assert_eq!(apply(user_patch(patch), &user), user);
        }
    }

    #[test]
    fn applying_a_merged_patch_equals_applying_the_patches_in_order(
        user in any_user(),
        a in any_user_patch(),
        b in any_user_patch(),
    ) {
        // Given any User and any UserPatches a and b
        // When a.merge(b) is applied to one copy of the user
        let merged = apply(user_patch(a.clone()).merge(user_patch(b.clone())), &user);
        // And a and then b are applied to another copy
        let in_order = apply(user_patch(b), &apply(user_patch(a), &user));
        // Then both copies are equal
        prop_assert_eq!(merged, in_order);
    }

    #[test]
    fn merging_patches_is_associative_and_the_empty_patch_changes_nothing(
        a in any_user_patch(),
        b in any_user_patch(),
        c in any_user_patch(),
    ) {
        // Given any UserPatches a, b and c
        // When they are composed with merge
        let left = user_patch(a.clone()).merge(user_patch(b.clone())).merge(user_patch(c.clone()));
        let right = user_patch(a.clone()).merge(user_patch(b).merge(user_patch(c)));
        // Then a.merge(b).merge(c) equals a.merge(b.merge(c))
        prop_assert_eq!(fields(left), fields(right));
        // And UserPatch::default().merge(a) and a.merge(UserPatch::default()) both equal a
        prop_assert_eq!(fields(UserPatch::default().merge(user_patch(a.clone()))), a.clone());
        prop_assert_eq!(fields(user_patch(a.clone()).merge(UserPatch::default())), a);
    }
}

// Spec 09: naming and visibility

mod model {
    use presence_rs::{Patch, Presence};

    #[derive(Patch)]
    pub(crate) struct Account {
        pub owner: String,
        note: Option<String>,
    }

    pub(crate) fn owner_patch(owner: &str) -> AccountPatch {
        AccountPatch {
            owner: Some(owner.to_owned()),
            note: Presence::Absent,
        }
    }

    pub(crate) fn account(owner: &str) -> Account {
        Account {
            owner: owner.to_owned(),
            note: Some("kept".to_owned()),
        }
    }

    pub(crate) fn note(account: &Account) -> Option<&str> {
        account.note.as_deref()
    }
}

#[test]
fn the_patch_is_named_after_the_model_and_keeps_its_visibility() {
    // Given a pub(crate) model Account with a pub field and a private field
    // When Patch is derived
    // Then a pub(crate) struct AccountPatch exists, nameable outside its module
    let patch: model::AccountPatch = model::owner_patch("ada");
    // And its pub field is readable outside the module, as on the model
    assert_eq!(patch.owner.as_deref(), Some("ada"));
    let mut account = model::account("bob");
    patch.apply_to(&mut account);
    assert_eq!(account.owner, "ada");
    assert_eq!(model::note(&account), Some("kept"));
}

// Spec 09: container and field options

#[derive(Patch, Debug, Clone, PartialEq)]
struct Record {
    #[patch(skip)]
    id: u64,
    title: String,
}

#[test]
fn a_skipped_field_is_not_part_of_the_patch() {
    // Given a model whose id field has #[patch(skip)]
    // When code builds the patch with an id field
    // Then it fails to compile
    compile_fail("tests/ui/skipped_field.rs");
}

proptest! {
    #![proptest_config(persisted_config())]

    #[test]
    fn a_skipped_field_is_not_part_of_the_patch_and_is_never_changed(
        id in any::<u64>(),
        title in any::<String>(),
        patched_title in any::<Option<String>>(),
    ) {
        // Given a model whose id field has #[patch(skip)]
        let mut record = Record { id, title };
        // When any patch is applied to any model
        RecordPatch { title: patched_title }.apply_to(&mut record);
        // Then id is unchanged
        prop_assert_eq!(record.id, id);
    }
}

#[derive(Patch)]
#[patch(name = AccountUpdate)]
struct Profile {
    handle: String,
}

#[test]
fn the_patch_name_can_be_overridden() {
    // Given a model with #[patch(name = AccountUpdate)]
    // When Patch is derived
    // Then the patch struct is named AccountUpdate
    let update = AccountUpdate {
        handle: Some("ada".to_owned()),
    };
    let mut profile = Profile {
        handle: "bob".to_owned(),
    };
    update.apply_to(&mut profile);
    assert_eq!(profile.handle, "ada");
}

#[derive(Patch, Debug, PartialEq)]
#[patch(derive(Debug, Clone, PartialEq))]
struct Labelled<'a, T>
where
    T: Clone,
{
    label: &'a str,
    value: Option<T>,
}

#[test]
fn generics_lifetimes_and_where_clauses_carry_over() {
    // Given a model Labelled<'a, T> where T: Clone, with fields label: &'a str and value: Option<T>
    let mut labelled = Labelled {
        label: "before",
        value: Some(1_u8),
    };
    // When Patch is derived
    // Then LabelledPatch<'a, T> exists with the same bounds
    let patch: LabelledPatch<'_, u8> = LabelledPatch {
        label: Some("after"),
        value: Presence::Null,
    };
    // And applying it follows the field mapping rules
    patch.apply_to(&mut labelled);
    assert_eq!(
        labelled,
        Labelled {
            label: "after",
            value: None,
        }
    );
}

mod generic_wire {
    use presence_rs::Patch;
    use serde::{Deserialize, Serialize};

    #[derive(Patch, Debug, PartialEq)]
    #[patch(derive(Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema))]
    pub(crate) struct Wrapper<T> {
        pub(crate) value: T,
        pub(crate) extra: Option<T>,
    }

    /// No nullable field mentions `T`, so serde cannot infer `T: Deserialize`
    /// from one.
    #[derive(Patch)]
    #[patch(derive(Serialize, Deserialize))]
    pub(crate) struct Only<T> {
        pub(crate) value: T,
    }
}

#[test]
fn generics_lifetimes_and_where_clauses_carry_over_with_serde_derives() {
    // Given a generic model Wrapper<T> whose patch derives Serialize and Deserialize
    // When Patch is derived
    let patch: generic_wire::WrapperPatch<u8> =
        serde_json::from_str(r#"{"value": 3, "extra": null}"#).unwrap();
    // Then the patch deserializes, rejects null for value, and applies as usual
    let only: generic_wire::OnlyPatch<u8> = serde_json::from_str(r#"{"value": 4}"#).unwrap();
    assert_eq!(only.value, Some(4));
    assert!(serde_json::from_str::<generic_wire::WrapperPatch<u8>>(r#"{"value": null}"#).is_err());
    let mut wrapper = generic_wire::Wrapper {
        value: 1_u8,
        extra: Some(2),
    };
    patch.apply_to(&mut wrapper);
    assert_eq!(
        wrapper,
        generic_wire::Wrapper {
            value: 3,
            extra: None,
        }
    );
}

#[test]
fn requested_derives_are_added_to_the_patch() {
    fn assert_derived<T: Debug + Clone + PartialEq + Default>() {}
    // Given a model with #[patch(derive(Debug, Clone, PartialEq))]
    // When Patch is derived
    // Then the patch implements Debug, Clone, PartialEq and Default
    assert_derived::<LabelledPatch<'static, u8>>();
    let patch = LabelledPatch::<u8> {
        label: Some("x"),
        value: Presence::Some(1),
    };
    assert_eq!(patch.clone(), patch);
    assert_eq!(
        format!("{patch:?}"),
        r#"LabelledPatch { label: Some("x"), value: Some(1) }"#
    );
}

// Spec 09: compile errors

#[test]
fn only_structs_with_named_fields_can_derive_patch() {
    // Given a tuple struct, a unit struct, an enum or a union
    // When Patch is derived on it
    // Then compilation fails with an error pointing at the item
    compile_fail("tests/ui/unsupported_input.rs");
}

#[test]
fn an_unknown_patch_attribute_is_a_compile_error() {
    // Given a model with #[patch(rename = X)] or a field with #[patch(nullable)]
    // When Patch is derived
    // Then compilation fails with an error pointing at each unknown key
    compile_fail("tests/ui/unknown_keys.rs");
}

// Spec 09: serde

mod wire {
    use presence_rs::Patch;
    use serde::{Deserialize, Serialize};

    #[derive(Patch)]
    #[patch(derive(Debug, Clone, PartialEq, Serialize, Deserialize))]
    pub(crate) struct User {
        pub(crate) name: String,
        pub(crate) nickname: Option<String>,
    }
}

use wire::UserPatch as WireUserPatch;

fn any_wire_patch() -> impl Strategy<Value = WireUserPatch> {
    (any::<Option<String>>(), any_string_presence())
        .prop_map(|(name, nickname)| WireUserPatch { name, nickname })
}

#[test]
fn a_patch_without_serde_derives_carries_no_serde_attributes() {
    // Given a model without Serialize or Deserialize in #[patch(derive(...))]
    // (User at the top of this file, whose module has no serde derive in scope)
    // When Patch is derived
    // Then the crate compiles: a #[serde] attribute would be unknown here
    assert!(UserPatch::default().is_empty());
}

#[test]
fn unset_fields_are_left_out_of_the_serialized_patch() {
    // Given a UserPatch that derives Serialize, with name None and nickname Absent
    let patch = WireUserPatch::default();
    // When it is serialized to JSON
    let json = serde_json::to_string(&patch).unwrap();
    // Then the JSON is {}
    assert_eq!(json, "{}");
}

#[test]
fn missing_keys_deserialize_to_unset_fields() {
    // Given a UserPatch that derives Deserialize
    // When the JSON {} is deserialized
    let patch: WireUserPatch = serde_json::from_str("{}").unwrap();
    // Then the patch is empty
    assert!(patch.is_empty());
}

#[test]
fn a_null_nullable_field_round_trips_as_null() {
    // Given a UserPatch that derives Serialize and Deserialize
    // When the JSON {"nickname": null} is deserialized
    let patch: WireUserPatch = serde_json::from_str(r#"{"nickname": null}"#).unwrap();
    // Then nickname is Presence::Null
    assert_eq!(patch.nickname, Presence::Null);
    // And serializing the patch gives {"nickname": null}
    assert_eq!(
        serde_json::to_string(&patch).unwrap(),
        r#"{"nickname":null}"#
    );
}

#[test]
fn a_null_required_field_is_a_deserialization_error() {
    // Given a UserPatch that derives Deserialize
    // When the JSON {"name": null} is deserialized
    let result = serde_json::from_str::<WireUserPatch>(r#"{"name": null}"#);
    // Then deserialization fails
    assert!(result.is_err());
}

proptest! {
    #![proptest_config(persisted_config())]

    #[test]
    fn a_serialized_patch_deserializes_to_the_same_patch(patch in any_wire_patch()) {
        // Given any UserPatch that derives Serialize and Deserialize
        // When it is serialized to JSON and deserialized back
        let json = serde_json::to_string(&patch).unwrap();
        let back: WireUserPatch = serde_json::from_str(&json).unwrap();
        // Then the result equals the original
        prop_assert_eq!(back, patch);
    }
}

// Spec 09: serde attributes from the model

mod renamed {
    use presence_rs::Patch;
    use serde::{Deserialize, Serialize};

    #[derive(Patch, Serialize, Deserialize)]
    #[patch(derive(Debug, PartialEq, Serialize, Deserialize))]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    pub(crate) struct Contact {
        pub(crate) display_name: String,
        #[serde(
            rename = "mail",
            alias = "e_mail",
            skip_serializing_if = "String::is_empty"
        )]
        pub(crate) email: String,
        #[serde(skip)]
        pub(crate) cache: u64,
        #[serde(skip_deserializing)]
        pub(crate) revision: u64,
    }
}

use renamed::ContactPatch;

#[test]
fn the_models_wire_names_are_kept() {
    // Given a model with #[serde(rename_all = "camelCase")], a field display_name
    // and a field email with #[serde(rename = "mail", alias = "e_mail")]
    // When Patch is derived with Serialize and Deserialize
    let patch = ContactPatch {
        display_name: Some("Ada".to_owned()),
        email: Some("ada@example.com".to_owned()),
    };
    // Then the patch reads and writes displayName and mail
    let json = serde_json::to_string(&patch).unwrap();
    assert_eq!(json, r#"{"displayName":"Ada","mail":"ada@example.com"}"#);
    assert_eq!(serde_json::from_str::<ContactPatch>(&json).unwrap(), patch);
    // And it also accepts e_mail when deserializing
    let aliased: ContactPatch = serde_json::from_str(r#"{"e_mail":"ada@example.com"}"#).unwrap();
    assert_eq!(aliased.email.as_deref(), Some("ada@example.com"));
}

#[test]
fn unknown_keys_are_rejected_when_the_model_rejects_them() {
    // Given a model with #[serde(deny_unknown_fields)]
    // When JSON with a key the model does not have is deserialized into its patch
    let result = serde_json::from_str::<ContactPatch>(r#"{"phone":"555"}"#);
    // Then deserialization fails
    let error = result.unwrap_err().to_string();
    assert!(error.contains("unknown field `phone`"), "{error}");
}

#[test]
fn fields_skipped_by_serde_are_left_out_of_the_patch() {
    // Given a model with a field marked #[serde(skip)] and another marked #[serde(skip_deserializing)]
    // When Patch is derived
    // Then the patch has neither field: this literal names every field it has
    let patch = ContactPatch {
        display_name: None,
        email: None,
    };
    assert!(patch.is_empty());
    let mut contact = renamed::Contact {
        display_name: String::new(),
        email: String::new(),
        cache: 7,
        revision: 3,
    };
    patch.apply_to(&mut contact);
    assert_eq!((contact.cache, contact.revision), (7, 3));
}

#[test]
fn a_field_with_a_custom_serde_codec_is_a_compile_error() {
    // Given a model with a field marked with, serialize_with, deserialize_with or flatten
    // When Patch is derived
    // Then compilation fails with an error on that field that suggests #[patch(skip)]
    compile_fail("tests/ui/serde_codec.rs");
}

// Spec 09: type aliases of Option

mod aliased {
    use presence_rs::Patch;
    use serde::{Deserialize, Serialize};

    pub(crate) type MaybeName = Option<String>;

    #[derive(Patch)]
    #[patch(derive(Debug, Serialize, Deserialize))]
    pub(crate) struct Tagged {
        pub(crate) field: MaybeName,
    }
}

#[test]
fn a_type_alias_of_option_is_treated_as_a_required_field() {
    // Given a model field of type MaybeName, where type MaybeName = Option<String>
    // When Patch is derived with Deserialize
    // Then the patch field has type Option<MaybeName>
    let patch = aliased::TaggedPatch {
        field: Some(None::<String> as aliased::MaybeName),
    };
    let _: &Option<aliased::MaybeName> = &patch.field;
    // And {"field": null} fails to deserialize
    let result = serde_json::from_str::<aliased::TaggedPatch>(r#"{"field": null}"#);
    assert!(result.is_err());
}

// Spec 09: JSON Schema

mod described {
    use presence_rs::Patch;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Patch)]
    #[patch(derive(JsonSchema, Serialize, Deserialize))]
    pub(crate) struct User {
        pub(crate) name: String,
        pub(crate) nickname: Option<String>,
    }
}

#[test]
fn the_schema_of_a_patch_matches_what_it_accepts() {
    // Given a UserPatch that derives JsonSchema, Serialize and Deserialize
    // When its JSON Schema is generated
    let schema = serde_json::to_value(schemars::schema_for!(described::UserPatch)).unwrap();
    // Then name has the schema of a string, not nullable
    assert_eq!(
        schema["properties"]["name"],
        serde_json::json!({"type": "string"})
    );
    // And nickname has the schema of a string or null
    assert_eq!(
        schema["properties"]["nickname"],
        serde_json::json!({"type": ["string", "null"]})
    );
    // And neither is listed in required
    assert!(schema.get("required").is_none(), "{schema}");
}

mod described_without_serde {
    use presence_rs::Patch;
    use schemars::JsonSchema;

    #[derive(Patch)]
    #[patch(derive(JsonSchema))]
    pub(crate) struct User {
        pub(crate) name: String,
        pub(crate) nickname: Option<String>,
    }
}

#[test]
fn the_schema_of_a_patch_matches_what_it_accepts_without_serde_derives() {
    // Given a UserPatch that derives JsonSchema but no serde trait
    // When its JSON Schema is generated
    let schema =
        serde_json::to_value(schemars::schema_for!(described_without_serde::UserPatch)).unwrap();
    // Then name is a string, nickname a string or null, and neither is listed in required
    assert_eq!(
        schema["properties"]["name"],
        serde_json::json!({"type": "string"})
    );
    assert_eq!(
        schema["properties"]["nickname"],
        serde_json::json!({"type": ["string", "null"]})
    );
    assert!(schema.get("required").is_none(), "{schema}");
}

// Spec 09: crate path

/// Stands in for a renamed dependency: everything the generated code needs,
/// reached through a path other than `::presence_rs`.
mod renamed_presence {
    pub use presence_rs::{__private, Patch, Presence};
}

mod with_crate_path {
    use super::renamed_presence::Patch;
    use serde::{Deserialize, Serialize};

    #[derive(Patch, Debug, PartialEq)]
    #[patch(crate = crate::renamed_presence, derive(Serialize, Deserialize))]
    pub(crate) struct Device {
        pub(crate) name: String,
        pub(crate) owner: Option<String>,
    }
}

#[test]
fn the_crate_path_can_be_overridden() {
    use renamed_presence::Patch as _;
    // Given a crate that reaches presence-rs under another path
    // And a model with #[patch(crate = crate::renamed_presence)]
    // When Patch is derived
    let patch: with_crate_path::DevicePatch = serde_json::from_str(r#"{"owner": null}"#).unwrap();
    let mut device = with_crate_path::Device {
        name: "pump".to_owned(),
        owner: Some("ada".to_owned()),
    };
    // Then the crate compiles and the patch applies as usual
    patch.apply_to(&mut device);
    assert_eq!(
        device,
        with_crate_path::Device {
            name: "pump".to_owned(),
            owner: None,
        }
    );
}
