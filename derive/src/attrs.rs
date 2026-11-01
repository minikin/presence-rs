use quote::ToTokens;
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::{Attribute, Field, Ident, Meta, Path, Token};

/// The options of `#[patch(...)]` on the model struct.
pub(crate) struct ContainerOptions {
    pub(crate) name: Option<Ident>,
    pub(crate) derives: Vec<Path>,
    /// The path to presence-rs in generated code, `::presence_rs` unless
    /// `#[patch(crate = ...)]` names another.
    pub(crate) krate: Path,
}

impl Default for ContainerOptions {
    fn default() -> Self {
        ContainerOptions {
            name: None,
            derives: Vec::new(),
            krate: syn::parse_quote!(::presence_rs),
        }
    }
}

impl ContainerOptions {
    /// Whether the derive list asks for serde, matched on the last path segment so a
    /// re-export such as `my_prelude::Deserialize` counts. A false match, such as
    /// `rkyv::Deserialize`, fails to compile. A missed one would drop
    /// `#[serde(default)]` and turn every missing key into `Null` without a sound.
    pub(crate) fn uses_serde(&self) -> bool {
        self.derives_any(&["Serialize", "Deserialize"])
    }

    pub(crate) fn uses_json_schema(&self) -> bool {
        self.derives_any(&["JsonSchema"])
    }

    fn derives_any(&self, names: &[&str]) -> bool {
        self.derives.iter().any(|path| {
            path.segments
                .last()
                .is_some_and(|segment| names.iter().any(|name| segment.ident == name))
        })
    }

    pub(crate) fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut options = Self::default();
        for attr in patch_attrs(attrs) {
            attr.parse_nested_meta(|meta| options.parse_key(&meta))?;
        }
        Ok(options)
    }

    fn parse_key(&mut self, meta: &ParseNestedMeta) -> syn::Result<()> {
        if meta.path.is_ident("name") {
            self.name = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("crate") {
            self.krate = meta.value()?.call(Path::parse_mod_style)?;
            Ok(())
        } else if meta.path.is_ident("derive") {
            meta.parse_nested_meta(|derive| {
                if !is_default(&derive.path) {
                    self.derives.push(derive.path);
                }
                Ok(())
            })
        } else {
            Err(unknown_key(meta))
        }
    }
}

/// Whether a model field carries `#[patch(skip)]`.
pub(crate) fn is_skipped(field: &Field) -> syn::Result<bool> {
    let mut skipped = false;
    for attr in patch_attrs(&field.attrs) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                skipped = true;
                Ok(())
            } else {
                Err(unknown_key(&meta))
            }
        })?;
    }
    Ok(skipped)
}

/// `Default` is dropped from the derive list: the patch always has a
/// hand-written `Default` impl, and deriving it too would conflict.
fn is_default(path: &Path) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == "Default")
}

/// The model's `#[serde(...)]` container attributes that shape the wire format the
/// patch must share: `rename_all` and `deny_unknown_fields`.
pub(crate) fn container_serde(attrs: &[Attribute]) -> syn::Result<Vec<Meta>> {
    Ok(serde_metas(attrs)?
        .into_iter()
        .filter(|meta| has_key(meta, &["rename_all", "deny_unknown_fields"]))
        .collect())
}

/// What a model field's `#[serde(...)]` attributes mean for its patch field.
pub(crate) struct FieldSerde {
    /// `rename` and `alias`, copied to the patch field.
    pub(crate) kept: Vec<Meta>,
    /// `skip` or `skip_deserializing`: the field never comes off the wire.
    pub(crate) skipped: bool,
}

pub(crate) fn field_serde(field: &Field) -> syn::Result<FieldSerde> {
    let mut serde = FieldSerde {
        kept: Vec::new(),
        skipped: false,
    };
    for meta in serde_metas(&field.attrs)? {
        if has_key(
            &meta,
            &["with", "serialize_with", "deserialize_with", "flatten"],
        ) {
            return Err(codec_error(&meta));
        }
        serde.skipped |= has_key(&meta, &["skip", "skip_deserializing"]);
        if has_key(&meta, &["rename", "alias"]) {
            serde.kept.push(meta);
        }
    }
    Ok(serde)
}

fn serde_metas(attrs: &[Attribute]) -> syn::Result<Vec<Meta>> {
    let mut metas = Vec::new();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        metas.extend(attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?);
    }
    Ok(metas)
}

fn has_key(meta: &Meta, keys: &[&str]) -> bool {
    keys.iter().any(|key| meta.path().is_ident(key))
}

fn codec_error(meta: &Meta) -> syn::Error {
    let key = meta.path().to_token_stream().to_string();
    syn::Error::new_spanned(
        meta,
        format!(
            "`#[serde({key})]` cannot be carried over to a patch field: \
             add `#[patch(skip)]` to leave the field out of the patch"
        ),
    )
}

fn patch_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
    attrs.iter().filter(|attr| attr.path().is_ident("patch"))
}

fn unknown_key(meta: &ParseNestedMeta) -> syn::Error {
    let key = meta.path.to_token_stream().to_string().replace(' ', "");
    meta.error(format!("unknown patch attribute `{key}`"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream as TokenStream2;
    use syn::DeriveInput;
    use syn::parse_quote;

    fn container(input: &DeriveInput) -> syn::Result<ContainerOptions> {
        ContainerOptions::parse(&input.attrs)
    }

    fn keys(metas: &[syn::Meta]) -> Vec<String> {
        metas
            .iter()
            .map(|m| m.to_token_stream().to_string())
            .collect()
    }

    #[test]
    fn only_wire_shaping_container_attributes_are_kept() {
        let input: DeriveInput = parse_quote! {
            #[serde(rename_all = "camelCase", crate = "s", bound = "")]
            #[serde(deny_unknown_fields, default)]
            struct A { a: u8 }
        };
        let kept = container_serde(&input.attrs).expect("valid serde attributes");
        assert_eq!(
            keys(&kept),
            [r#"rename_all = "camelCase""#, "deny_unknown_fields"]
        );
    }

    #[test]
    fn rename_and_alias_are_kept_on_a_field_and_the_rest_dropped() {
        let field: Field = parse_quote! {
            #[serde(rename = "mail", default, skip_serializing_if = "x")]
            #[serde(alias = "e_mail", skip_serializing, rename(serialize = "m"))]
            email: String
        };
        let serde = field_serde(&field).expect("valid serde attributes");
        assert!(!serde.skipped);
        assert_eq!(
            keys(&serde.kept),
            [
                r#"rename = "mail""#,
                r#"alias = "e_mail""#,
                r#"rename (serialize = "m")"#,
            ]
        );
    }

    #[test]
    fn skip_and_skip_deserializing_skip_the_field() {
        for attr in [quote::quote!(skip), quote::quote!(skip_deserializing)] {
            let field: Field = parse_quote!(#[serde(#attr)] cache: u64);
            assert!(field_serde(&field).expect("valid").skipped, "{attr}");
        }
    }

    #[test]
    fn a_custom_codec_is_an_error_that_suggests_patch_skip() {
        for (attr, key) in [
            (quote::quote!(with = "m"), "with"),
            (quote::quote!(serialize_with = "f"), "serialize_with"),
            (quote::quote!(deserialize_with = "f"), "deserialize_with"),
            (quote::quote!(flatten), "flatten"),
        ] {
            let field: Field = parse_quote!(#[serde(#attr)] value: u8);
            let Err(error) = field_serde(&field) else {
                panic!("{key} is rejected");
            };
            assert_eq!(
                error.to_string(),
                format!(
                    "`#[serde({key})]` cannot be carried over to a patch field: \
                     add `#[patch(skip)]` to leave the field out of the patch"
                )
            );
        }
    }

    fn options_with(derives: &TokenStream2) -> ContainerOptions {
        let input: DeriveInput = parse_quote!(
            #[patch(derive(#derives))]
            struct A {
                a: u8,
            }
        );
        container(&input).expect("valid options")
    }

    #[test]
    fn serde_derives_count_under_any_path() {
        for derive in [
            quote::quote!(Serialize),
            quote::quote!(Deserialize),
            quote::quote!(serde::Serialize),
            quote::quote!(::serde::Deserialize),
            quote::quote!(serde_derive::Deserialize),
            quote::quote!(my_prelude::Serialize),
        ] {
            assert!(options_with(&derive).uses_serde(), "{derive}");
        }
    }

    #[test]
    fn other_derives_are_not_serde() {
        for derive in [quote::quote!(Debug), quote::quote!(Serializer)] {
            assert!(!options_with(&derive).uses_serde(), "{derive}");
        }
    }

    #[test]
    fn json_schema_counts_under_any_path() {
        assert!(options_with(&quote::quote!(JsonSchema)).uses_json_schema());
        assert!(options_with(&quote::quote!(schemars::JsonSchema)).uses_json_schema());
        assert!(!options_with(&quote::quote!(Debug)).uses_json_schema());
    }

    #[test]
    fn a_model_without_patch_attributes_has_default_options() {
        let input: DeriveInput = parse_quote!(
            #[derive(Clone)]
            struct A {
                a: u8,
            }
        );
        let options = container(&input).expect("valid options");
        assert!(options.name.is_none());
        assert!(options.derives.is_empty());
    }

    #[test]
    fn name_takes_an_identifier() {
        let input: DeriveInput = parse_quote!(
            #[patch(name = Update)]
            struct A {
                a: u8,
            }
        );
        let options = container(&input).expect("valid options");
        assert_eq!(options.name.expect("a name"), "Update");
    }

    #[test]
    fn derive_collects_paths_across_attributes() {
        let input: DeriveInput = parse_quote! {
            #[patch(derive(Debug, serde::Serialize))]
            #[patch(derive(Clone))]
            struct A { a: u8 }
        };
        let derives: Vec<String> = container(&input)
            .expect("valid options")
            .derives
            .iter()
            .map(|path| path.to_token_stream().to_string().replace(' ', ""))
            .collect();
        assert_eq!(derives, ["Debug", "serde::Serialize", "Clone"]);
    }

    #[test]
    fn default_in_the_derive_list_is_dropped_because_the_patch_always_implements_it() {
        let input: DeriveInput = parse_quote! {
            #[patch(derive(Debug, Default, core::default::Default))]
            struct A { a: u8 }
        };
        let derives: Vec<String> = container(&input)
            .expect("valid options")
            .derives
            .iter()
            .map(|path| path.to_token_stream().to_string())
            .collect();
        assert_eq!(derives, ["Debug"]);
    }

    #[test]
    fn crate_takes_a_path_and_defaults_to_presence_rs() {
        let input: DeriveInput = parse_quote!(
            #[patch(crate = ::my::presence)]
            struct A {
                a: u8,
            }
        );
        let path = container(&input).expect("valid options").krate;
        assert_eq!(path.to_token_stream().to_string(), ":: my :: presence");
        let input: DeriveInput = parse_quote!(
            struct A {
                a: u8,
            }
        );
        let path = container(&input).expect("valid options").krate;
        assert_eq!(path.to_token_stream().to_string(), ":: presence_rs");
    }

    #[test]
    fn an_unknown_container_key_is_an_error() {
        let input: DeriveInput = parse_quote!(
            #[patch(rename = X)]
            struct A {
                a: u8,
            }
        );
        let Err(error) = container(&input) else {
            panic!("an unknown key is rejected");
        };
        assert_eq!(error.to_string(), "unknown patch attribute `rename`");
    }

    #[test]
    fn a_field_with_skip_is_skipped() {
        let field: Field = parse_quote!(#[patch(skip)] id: u64);
        assert!(is_skipped(&field).expect("valid options"));
    }

    #[test]
    fn a_field_without_patch_attributes_is_kept() {
        let field: Field = parse_quote!(#[serde(skip)] id: u64);
        assert!(!is_skipped(&field).expect("valid options"));
    }

    #[test]
    fn an_unknown_field_key_is_an_error() {
        let field: Field = parse_quote!(#[patch(nullable)] id: u64);
        let error = is_skipped(&field).expect_err("an unknown key is rejected");
        assert_eq!(error.to_string(), "unknown patch attribute `nullable`");
    }
}
