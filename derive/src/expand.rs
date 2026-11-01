use crate::attrs::{ContainerOptions, container_serde, field_serde, is_skipped};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DataStruct, DeriveInput, Field, Fields, GenericArgument, Meta, Path, PathArguments,
    PathSegment, Type, TypePath,
};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    syn::parse2::<DeriveInput>(input)
        .and_then(|input| derive(&input))
        .unwrap_or_else(syn::Error::into_compile_error)
}

fn derive(input: &DeriveInput) -> syn::Result<TokenStream> {
    let options = ContainerOptions::parse(&input.attrs);
    let serde = options.as_ref().is_ok_and(ContainerOptions::uses_serde);
    let container_serde = if serde {
        container_serde(&input.attrs)
    } else {
        Ok(Vec::new())
    };
    let ((options, fields), container_serde) =
        both(both(options, named_fields(input, serde)), container_serde)?;
    let generator = Generator {
        krate: &options.krate,
        serde,
        schema: options.uses_json_schema(),
    };
    let krate = generator.krate;
    let container_serde = serde_list(&container_serde);
    let vis = &input.vis;
    let model = &input.ident;
    let patch = options
        .name
        .clone()
        .unwrap_or_else(|| format_ident!("{model}Patch"));
    let derives = derive_attribute(&options.derives);
    let generics = &input.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    // Plain text: with `JsonSchema` derived, this becomes the schema's description.
    let doc = format!("A partial update of a {model}.");
    let declarations = fields.iter().map(|field| generator.declaration(field));
    let applications = fields.iter().map(|field| generator.application(field));
    let unset = fields.iter().map(|field| generator.unset(field));
    let merges = fields.iter().map(|field| generator.merge(field));
    let emptiness = fields.iter().map(|field| generator.emptiness(field));
    Ok(quote! {
        #[doc = #doc]
        #derives
        #container_serde
        #vis struct #patch #generics #where_clause {
            #(#declarations,)*
        }

        #[automatically_derived]
        impl #impl_generics ::core::default::Default for #patch #type_generics #where_clause {
            fn default() -> Self {
                Self { #(#unset,)* }
            }
        }

        #[automatically_derived]
        impl #impl_generics #krate::Patch for #patch #type_generics #where_clause {
            type Target = #model #type_generics;

            fn apply_to(self, target: &mut Self::Target) {
                #(#applications)*
            }

            fn merge(self, later: Self) -> Self {
                Self { #(#merges,)* }
            }

            fn is_empty(&self) -> bool {
                true #(&& #emptiness)*
            }
        }
    })
}
fn named_fields(input: &DeriveInput, serde: bool) -> syn::Result<Vec<PatchField<'_>>> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => kept_fields(fields.named.iter(), serde),
        _ => Err(syn::Error::new_spanned(
            &input.ident,
            "Patch can only be derived for structs with named fields",
        )),
    }
}

/// The fields without `#[patch(skip)]`, or every field's attribute error at once.
fn kept_fields<'a>(
    fields: impl Iterator<Item = &'a Field>,
    serde: bool,
) -> syn::Result<Vec<PatchField<'a>>> {
    let mut kept = Ok(Vec::new());
    for field in fields {
        kept = both(kept, patch_field(field, serde)).map(|(mut kept, field)| {
            kept.extend(field);
            kept
        });
    }
    kept
}

/// The patch field for a model field, or `None` when `#[patch(skip)]` leaves it
/// out. A patch that derives serde shares the model's wire format, so for it
/// `#[serde(skip)]` and `#[serde(skip_deserializing)]` leave the field out too.
/// A patch without serde ignores the model's serde attributes.
fn patch_field(field: &Field, serde: bool) -> syn::Result<Option<PatchField<'_>>> {
    if is_skipped(field)? {
        return Ok(None);
    }
    if !serde {
        return Ok(Some(PatchField::new(field, Vec::new())));
    }
    let serde = field_serde(field)?;
    Ok((!serde.skipped).then(|| PatchField::new(field, serde.kept)))
}

/// `#[serde(...)]` holding `metas`. Only a patch that derives serde collects any.
fn serde_list(metas: &[Meta]) -> TokenStream {
    if metas.is_empty() {
        TokenStream::new()
    } else {
        quote!(#[serde(#(#metas),*)])
    }
}

/// Both results, or every error among them, so one build reports them all.
fn both<A, B>(a: syn::Result<A>, b: syn::Result<B>) -> syn::Result<(A, B)> {
    match (a, b) {
        (Ok(a), Ok(b)) => Ok((a, b)),
        (Err(mut first), Err(second)) => {
            first.combine(second);
            Err(first)
        }
        (Err(error), Ok(_)) | (Ok(_), Err(error)) => Err(error),
    }
}

fn derive_attribute(derives: &[Path]) -> TokenStream {
    if derives.is_empty() {
        TokenStream::new()
    } else {
        quote!(#[derive(#(#derives),*)])
    }
}

enum Kind<'a> {
    /// A model field of type `Option<T>`, holding `T`.
    Nullable(&'a Type),
    /// A model field of any other type.
    Required(&'a Type),
}

/// A model field, classified once so every generated item agrees on its kind.
struct PatchField<'a> {
    field: &'a Field,
    kind: Kind<'a>,
    /// The model field's `rename` and `alias`.
    serde: Vec<Meta>,
}

impl<'a> PatchField<'a> {
    fn new(field: &'a Field, serde: Vec<Meta>) -> Self {
        let kind = option_inner(&field.ty).map_or(Kind::Required(&field.ty), Kind::Nullable);
        PatchField { field, kind, serde }
    }
}

/// Generates the code for each patch field, with every path to presence-rs
/// going through the crate path of `#[patch(crate = ...)]`.
struct Generator<'a> {
    krate: &'a Path,
    serde: bool,
    schema: bool,
}

impl Generator<'_> {
    fn declaration(&self, patch_field: &PatchField<'_>) -> TokenStream {
        let krate = self.krate;
        let Field { vis, ident, .. } = patch_field.field;
        let ty = match patch_field.kind {
            Kind::Nullable(inner) => quote!(#krate::Presence<#inner>),
            Kind::Required(ty) => quote!(::core::option::Option<#ty>),
        };
        let docs = documentation(patch_field.field);
        let carried = serde_list(&patch_field.serde);
        let serde = self.serde.then(|| self.serde_attribute(&patch_field.kind));
        let schema = self
            .schema
            .then(|| self.schema_attribute(&patch_field.kind));
        quote! {
            #docs
            #serde
            #carried
            #schema
            #vis #ident: #ty
        }
    }

    /// A required field accepts a `T` but never `null`, so its schema is `T`'s,
    /// not the `T` or `null` that its `Option<T>` type would give. Without serde's
    /// `default`, schemars would list every field as required, so a patch without
    /// serde derives says `default` to schemars directly.
    fn schema_attribute(&self, kind: &Kind<'_>) -> TokenStream {
        let with = match kind {
            Kind::Nullable(_) => TokenStream::new(),
            Kind::Required(ty) => {
                let ty = quote!(#ty).to_string();
                quote!(#[schemars(with = #ty)])
            }
        };
        let default = (!self.serde).then(|| {
            let skip_if = self.skip_if(kind);
            quote!(#[schemars(default, skip_serializing_if = #skip_if)])
        });
        quote!(#with #default)
    }

    fn serde_attribute(&self, kind: &Kind<'_>) -> TokenStream {
        let skip_if = self.skip_if(kind);
        match kind {
            Kind::Nullable(_) => quote! {
                #[serde(default, skip_serializing_if = #skip_if)]
            },
            Kind::Required(ty) => {
                let some = self.path_string("__private::some");
                // `deserialize_with` stops serde from inferring `T: Deserialize` for a
                // generic field, so the field states its bound itself.
                let bound = format!(
                    "{}: {}<'de>",
                    quote!(#ty),
                    self.path_string("__private::Deserialize")
                );
                quote! {
                    #[serde(
                        default,
                        skip_serializing_if = #skip_if,
                        deserialize_with = #some,
                        bound(deserialize = #bound)
                    )]
                }
            }
        }
    }

    /// When a field is unset, so serde leaves it out and schemars omits its default.
    fn skip_if(&self, kind: &Kind<'_>) -> String {
        match kind {
            Kind::Nullable(_) => self.path_string("Presence::is_absent"),
            Kind::Required(_) => "::core::option::Option::is_none".to_owned(),
        }
    }

    /// `item` under the crate path, as the string a serde attribute takes.
    fn path_string(&self, item: &str) -> String {
        let krate = self.krate;
        format!("{}::{item}", quote!(#krate)).replace(' ', "")
    }

    fn application(&self, patch_field: &PatchField<'_>) -> TokenStream {
        let krate = self.krate;
        let ident = &patch_field.field.ident;
        match patch_field.kind {
            Kind::Nullable(_) => {
                quote!(#krate::Presence::apply_to(self.#ident, &mut target.#ident);)
            }
            Kind::Required(_) => quote! {
                if let ::core::option::Option::Some(value) = self.#ident {
                    target.#ident = value;
                }
            },
        }
    }

    fn unset(&self, patch_field: &PatchField<'_>) -> TokenStream {
        let krate = self.krate;
        let ident = &patch_field.field.ident;
        match patch_field.kind {
            Kind::Nullable(_) => quote!(#ident: #krate::Presence::Absent),
            Kind::Required(_) => quote!(#ident: ::core::option::Option::None),
        }
    }

    fn merge(&self, patch_field: &PatchField<'_>) -> TokenStream {
        let krate = self.krate;
        let ident = &patch_field.field.ident;
        match patch_field.kind {
            Kind::Nullable(_) => quote!(#ident: #krate::Presence::merge(self.#ident, later.#ident)),
            Kind::Required(_) => {
                quote!(#ident: ::core::option::Option::or(later.#ident, self.#ident))
            }
        }
    }

    fn emptiness(&self, patch_field: &PatchField<'_>) -> TokenStream {
        let krate = self.krate;
        let ident = &patch_field.field.ident;
        match patch_field.kind {
            Kind::Nullable(_) => quote!(#krate::Presence::is_absent(&self.#ident)),
            Kind::Required(_) => quote!(::core::option::Option::is_none(&self.#ident)),
        }
    }
}

/// The model field's own docs. A field without docs gets none: under
/// `missing_docs` the model field would need them first, and a generic line
/// would become every such field's description in a JSON Schema.
fn documentation(field: &Field) -> TokenStream {
    let docs = field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"));
    quote!(#(#docs)*)
}

/// The `T` of a field spelled `Option<T>`, `std::option::Option<T>` or
/// `core::option::Option<T>`. A macro sees only tokens, so a type alias of
/// `Option` is not recognized and counts as a required field.
fn option_inner(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(TypePath {
            qself: None, path, ..
        }) if is_option_path(path) => single_type_argument(path.segments.last()?),
        _ => None,
    }
}

fn is_option_path(path: &Path) -> bool {
    let names: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    matches!(
        (path.leading_colon.is_some(), names.join("::").as_str()),
        (false, "Option") | (_, "std::option::Option" | "core::option::Option")
    )
}

fn single_type_argument(segment: &PathSegment) -> Option<&Type> {
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    let mut arguments = arguments.args.iter();
    match (arguments.next(), arguments.next()) {
        (Some(GenericArgument::Type(ty)), None) => Some(ty),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::{ToTokens, quote};
    use syn::{File, ImplItem, Item, ItemImpl, ItemStruct, Type};

    fn expand_file(input: TokenStream) -> File {
        syn::parse2(expand(input)).expect("the expansion parses as Rust items")
    }

    fn patch_struct(file: &File) -> &ItemStruct {
        file.items
            .iter()
            .find_map(|item| match item {
                Item::Struct(s) => Some(s),
                _ => None,
            })
            .expect("the expansion contains a struct")
    }

    fn impls(file: &File) -> impl Iterator<Item = &ItemImpl> {
        file.items.iter().filter_map(|item| match item {
            Item::Impl(i) => Some(i),
            _ => None,
        })
    }

    fn trait_impl<'f>(file: &'f File, trait_path: &TokenStream) -> &'f ItemImpl {
        impls(file)
            .find(|i| {
                i.trait_
                    .as_ref()
                    .is_some_and(|(path, _)| tokens(path) == tokens(trait_path))
            })
            .unwrap_or_else(|| panic!("an impl of {trait_path}"))
    }

    fn method_body(file: &File, name: &str) -> String {
        impls(file)
            .flat_map(|i| &i.items)
            .find_map(|item| match item {
                ImplItem::Fn(f) if f.sig.ident == name => Some(tokens(&f.block)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("a method named {name}"))
    }

    fn tokens(t: &impl ToTokens) -> String {
        t.to_token_stream().to_string()
    }

    fn user() -> TokenStream {
        quote! {
            pub(crate) struct User {
                pub id: u64,
                name: String,
                pub(super) nickname: Option<String>,
            }
        }
    }

    #[test]
    fn the_patch_struct_is_named_after_the_model_with_its_visibility() {
        let file = expand_file(user());
        let patch = patch_struct(&file);
        assert_eq!(patch.ident, "UserPatch");
        assert_eq!(tokens(&patch.vis), "pub (crate)");
    }

    #[test]
    fn each_field_keeps_its_name_and_visibility() {
        let file = expand_file(user());
        let fields: Vec<(String, String)> = patch_struct(&file)
            .fields
            .iter()
            .map(|f| (tokens(&f.ident), tokens(&f.vis)))
            .collect();
        assert_eq!(
            fields,
            [
                ("id".to_owned(), "pub".to_owned()),
                ("name".to_owned(), String::new()),
                ("nickname".to_owned(), "pub (super)".to_owned()),
            ]
        );
    }

    #[test]
    fn option_fields_become_presence_and_other_fields_become_option() {
        let file = expand_file(user());
        let types: Vec<String> = patch_struct(&file)
            .fields
            .iter()
            .map(|f| tokens(&f.ty))
            .collect();
        assert_eq!(
            types,
            [
                tokens(&quote!(::core::option::Option<u64>)),
                tokens(&quote!(::core::option::Option<String>)),
                tokens(&quote!(::presence_rs::Presence<String>)),
            ]
        );
    }

    #[test]
    fn the_patch_implements_the_trait_for_the_model() {
        let file = expand_file(user());
        let imp = trait_impl(&file, &quote!(::presence_rs::Patch));
        assert_eq!(tokens(&imp.self_ty), "UserPatch");
        let target = imp.items.iter().find_map(|item| match item {
            ImplItem::Type(t) => Some(tokens(&t.ty)),
            _ => None,
        });
        assert_eq!(target.as_deref(), Some("User"));
    }

    #[test]
    fn apply_to_replaces_required_fields_and_patches_nullable_ones() {
        let file = expand_file(user());
        let body = method_body(&file, "apply_to");
        let expected = quote! {{
            if let ::core::option::Option::Some(value) = self.id {
                target.id = value;
            }
            if let ::core::option::Option::Some(value) = self.name {
                target.name = value;
            }
            ::presence_rs::Presence::apply_to(self.nickname, &mut target.nickname);
        }};
        assert_eq!(body, tokens(&expected));
    }

    #[test]
    fn the_default_patch_leaves_every_field_unset() {
        let file = expand_file(user());
        let default_impl = trait_impl(&file, &quote!(::core::default::Default));
        assert_eq!(tokens(&default_impl.self_ty), "UserPatch");
        let expected = quote! {{
            Self {
                id: ::core::option::Option::None,
                name: ::core::option::Option::None,
                nickname: ::presence_rs::Presence::Absent,
            }
        }};
        assert_eq!(method_body(&file, "default"), tokens(&expected));
    }

    #[test]
    fn merge_keeps_the_later_field_unless_it_is_unset() {
        let file = expand_file(user());
        let expected = quote! {{
            Self {
                id: ::core::option::Option::or(later.id, self.id),
                name: ::core::option::Option::or(later.name, self.name),
                nickname: ::presence_rs::Presence::merge(self.nickname, later.nickname),
            }
        }};
        assert_eq!(method_body(&file, "merge"), tokens(&expected));
    }

    #[test]
    fn is_empty_checks_that_every_field_is_unset() {
        let file = expand_file(user());
        let expected = quote! {{
            true && ::core::option::Option::is_none(&self.id)
                && ::core::option::Option::is_none(&self.name)
                && ::presence_rs::Presence::is_absent(&self.nickname)
        }};
        assert_eq!(method_body(&file, "is_empty"), tokens(&expected));
    }

    #[test]
    fn the_patch_struct_doc_is_plain_text() {
        let file = expand_file(user());
        let docs: Vec<String> = patch_struct(&file)
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("doc"))
            .map(tokens)
            .collect();
        assert_eq!(
            docs,
            [tokens(&quote!(#[doc = "A partial update of a User."]))]
        );
    }

    #[test]
    fn the_name_option_renames_the_patch() {
        let file = expand_file(quote! {
            #[patch(name = AccountUpdate)]
            struct Account { owner: String }
        });
        assert_eq!(patch_struct(&file).ident, "AccountUpdate");
        let imp = trait_impl(&file, &quote!(::presence_rs::Patch));
        assert_eq!(tokens(&imp.self_ty), "AccountUpdate");
    }

    #[test]
    fn requested_derives_are_put_on_the_patch_struct() {
        let file = expand_file(quote! {
            #[patch(derive(Debug, serde::Serialize))]
            struct Account { owner: String }
        });
        let attrs: Vec<String> = patch_struct(&file).attrs.iter().map(tokens).collect();
        assert!(
            attrs.contains(&tokens(&quote!(#[derive(Debug, serde::Serialize)]))),
            "{attrs:?}"
        );
    }

    #[test]
    fn without_requested_derives_the_struct_derives_nothing() {
        let file = expand_file(user());
        let derives = patch_struct(&file)
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("derive"))
            .count();
        assert_eq!(derives, 0);
    }

    #[test]
    fn a_skipped_field_is_left_out_of_every_generated_item() {
        let file = expand_file(quote! {
            struct Account {
                #[patch(skip)]
                id: u64,
                owner: String,
            }
        });
        let fields: Vec<String> = patch_struct(&file)
            .fields
            .iter()
            .map(|f| tokens(&f.ident))
            .collect();
        assert_eq!(fields, ["owner"]);
        for method in ["apply_to", "merge", "is_empty", "default"] {
            assert!(!method_body(&file, method).contains("id"), "{method}");
        }
    }

    #[test]
    fn generics_and_where_clauses_carry_over_to_the_patch() {
        let file = expand_file(quote! {
            struct Labelled<'a, T: Copy> where T: Clone {
                label: &'a str,
                value: Option<T>,
            }
        });
        let patch = patch_struct(&file);
        assert_eq!(tokens(&patch.generics), tokens(&quote!(<'a, T: Copy>)));
        assert_eq!(
            tokens(&patch.generics.where_clause),
            tokens(&quote!(where T: Clone))
        );
        for imp in impls(&file) {
            assert_eq!(tokens(&imp.generics), tokens(&quote!(<'a, T: Copy>)));
            assert_eq!(tokens(&imp.self_ty), tokens(&quote!(LabelledPatch<'a, T>)));
            assert_eq!(
                tokens(&imp.generics.where_clause),
                tokens(&quote!(where T: Clone))
            );
        }
        let imp = trait_impl(&file, &quote!(::presence_rs::Patch));
        let target = imp.items.iter().find_map(|item| match item {
            ImplItem::Type(t) => Some(tokens(&t.ty)),
            _ => None,
        });
        assert_eq!(target, Some(tokens(&quote!(Labelled<'a, T>))));
    }

    #[test]
    fn a_field_keeps_its_documentation() {
        let file = expand_file(quote! {
            struct Account {
                /// The owner's handle.
                owner: String,
                note: Option<String>,
            }
        });
        let docs: Vec<Vec<String>> = patch_struct(&file)
            .fields
            .iter()
            .map(|f| f.attrs.iter().map(tokens).collect())
            .collect();
        assert_eq!(
            docs,
            [
                vec![tokens(&quote!(#[doc = r" The owner's handle."]))],
                vec![],
            ]
        );
    }

    #[test]
    fn a_non_doc_field_attribute_is_not_copied() {
        let file = expand_file(quote! {
            struct Account {
                #[allow(dead_code)]
                owner: String,
            }
        });
        let field = patch_struct(&file).fields.iter().next().expect("one field");
        assert!(!field.attrs.iter().any(|a| a.path().is_ident("allow")));
    }

    fn field_attrs(file: &File) -> Vec<Vec<String>> {
        patch_struct(file)
            .fields
            .iter()
            .map(|f| {
                f.attrs
                    .iter()
                    .filter(|a| !a.path().is_ident("doc"))
                    .map(tokens)
                    .collect()
            })
            .collect()
    }

    #[test]
    fn serde_derives_give_each_field_its_serde_attributes() {
        for derive in [quote!(Deserialize), quote!(serde::Serialize)] {
            let file = expand_file(quote! {
                #[patch(derive(#derive))]
                struct User {
                    name: String,
                    nickname: Option<String>,
                }
            });
            assert_eq!(
                field_attrs(&file),
                [
                    vec![tokens(&quote!(#[serde(
                        default,
                        skip_serializing_if = "::core::option::Option::is_none",
                        deserialize_with = "::presence_rs::__private::some",
                        bound(deserialize = "String: ::presence_rs::__private::Deserialize<'de>")
                    )]))],
                    vec![tokens(&quote!(#[serde(
                        default,
                        skip_serializing_if = "::presence_rs::Presence::is_absent"
                    )]))],
                ],
                "{derive}"
            );
        }
    }

    #[test]
    fn the_models_serde_attributes_are_carried_over_when_the_patch_uses_serde() {
        let file = expand_file(quote! {
            #[patch(derive(Deserialize))]
            #[serde(rename_all = "camelCase")]
            struct User {
                #[serde(rename = "n")]
                name: String,
            }
        });
        let struct_attrs: Vec<String> = patch_struct(&file).attrs.iter().map(tokens).collect();
        assert!(
            struct_attrs.contains(&tokens(&quote!(#[serde(rename_all = "camelCase")]))),
            "{struct_attrs:?}"
        );
        let attrs = field_attrs(&file);
        assert_eq!(attrs[0].len(), 2);
        assert_eq!(attrs[0][1], tokens(&quote!(#[serde(rename = "n")])));
    }

    #[test]
    fn the_models_serde_attributes_are_not_carried_over_without_serde() {
        let file = expand_file(quote! {
            #[serde(rename_all = "camelCase")]
            struct User {
                #[serde(rename = "n")]
                name: String,
                #[serde(skip)]
                cache: u8,
                #[serde(with = "timestamp", flatten)]
                created: i64,
            }
        });
        assert!(
            patch_struct(&file)
                .attrs
                .iter()
                .all(|a| !a.path().is_ident("serde"))
        );
        let fields: Vec<String> = patch_struct(&file)
            .fields
            .iter()
            .map(|f| tokens(&f.ident))
            .collect();
        assert_eq!(fields, ["name", "cache", "created"]);
        assert_eq!(field_attrs(&file), [Vec::<String>::new(), vec![], vec![]]);
    }

    #[test]
    fn a_patch_without_serde_never_parses_the_models_serde_attributes() {
        let file = expand_file(quote! {
            #[serde(rename_all = )]
            struct User {
                #[serde(rename = )]
                name: String,
            }
        });
        assert_eq!(patch_struct(&file).ident, "UserPatch");
    }

    #[test]
    fn a_schema_without_serde_still_marks_every_field_optional() {
        let file = expand_file(quote! {
            #[patch(derive(JsonSchema))]
            struct User {
                name: String,
                nickname: Option<String>,
            }
        });
        assert_eq!(
            field_attrs(&file),
            [
                vec![
                    tokens(&quote!(#[schemars(with = "String")])),
                    tokens(
                        &quote!(#[schemars(default, skip_serializing_if = "::core::option::Option::is_none")])
                    ),
                ],
                vec![tokens(
                    &quote!(#[schemars(default, skip_serializing_if = "::presence_rs::Presence::is_absent")])
                )],
            ]
        );
    }

    #[test]
    fn a_json_schema_derive_describes_a_required_field_by_its_model_type() {
        let file = expand_file(quote! {
            #[patch(derive(schemars::JsonSchema))]
            struct User<'a> {
                name: &'a str,
                nickname: Option<String>,
            }
        });
        assert_eq!(
            field_attrs(&file),
            [
                vec![
                    tokens(&quote!(#[schemars(with = "& 'a str")])),
                    tokens(
                        &quote!(#[schemars(default, skip_serializing_if = "::core::option::Option::is_none")])
                    ),
                ],
                vec![tokens(
                    &quote!(#[schemars(default, skip_serializing_if = "::presence_rs::Presence::is_absent")])
                )],
            ]
        );
    }

    #[test]
    fn without_serde_derives_fields_get_no_serde_attributes() {
        let file = expand_file(quote! {
            #[patch(derive(Debug, my::Serializer))]
            struct User {
                name: String,
                nickname: Option<String>,
            }
        });
        assert_eq!(field_attrs(&file), [Vec::<String>::new(), Vec::new()]);
    }

    #[test]
    fn the_crate_option_replaces_every_presence_rs_path() {
        let output = expand(quote! {
            #[patch(crate = presence, derive(Serialize, JsonSchema))]
            struct User {
                name: String,
                nickname: Option<String>,
            }
        })
        .to_string();
        assert!(!output.contains("presence_rs"), "{output}");
        let file: File = syn::parse2(output.parse().unwrap()).unwrap();
        let imp = trait_impl(&file, &quote!(presence::Patch));
        assert_eq!(tokens(&imp.self_ty), "UserPatch");
        assert_eq!(
            field_attrs(&file)[0][0],
            tokens(&quote!(#[serde(
                default,
                skip_serializing_if = "::core::option::Option::is_none",
                deserialize_with = "presence::__private::some",
                bound(deserialize = "String: presence::__private::Deserialize<'de>")
            )]))
        );
        assert_eq!(
            field_attrs(&file)[1][0],
            tokens(
                &quote!(#[serde(default, skip_serializing_if = "presence::Presence::is_absent")])
            )
        );
    }

    fn compile_errors(input: TokenStream) -> Vec<String> {
        let output = expand(input);
        let file: File = syn::parse2(output).expect("the errors parse as items");
        file.items
            .iter()
            .map(|item| match item {
                Item::Macro(m) if m.mac.path.segments.last().unwrap().ident == "compile_error" => {
                    let message: syn::LitStr = m.mac.parse_body().expect("a message");
                    message.value()
                }
                other => panic!("expected only compile_error!, got {}", tokens(other)),
            })
            .collect()
    }

    #[test]
    fn items_without_named_fields_are_rejected() {
        for item in [
            quote!(
                struct Pair(u8, u8);
            ),
            quote!(
                struct Unit;
            ),
            quote!(
                enum Choice {
                    A,
                }
            ),
            quote!(union Bits { whole: u32 }),
        ] {
            assert_eq!(
                compile_errors(item.clone()),
                ["Patch can only be derived for structs with named fields"],
                "{item}"
            );
        }
    }

    #[test]
    fn every_unknown_key_is_reported_in_one_expansion() {
        let errors = compile_errors(quote! {
            #[patch(rename = X)]
            struct Account {
                #[patch(nullable)]
                owner: String,
                #[patch(flatten)]
                note: Option<String>,
            }
        });
        assert_eq!(
            errors,
            [
                "unknown patch attribute `rename`",
                "unknown patch attribute `nullable`",
                "unknown patch attribute `flatten`",
            ]
        );
    }

    #[test]
    fn a_container_error_and_a_shape_error_are_both_reported() {
        let errors = compile_errors(quote! {
            #[patch(rename = X)]
            enum Choice { A }
        });
        assert_eq!(
            errors,
            [
                "unknown patch attribute `rename`",
                "Patch can only be derived for structs with named fields",
            ]
        );
    }

    fn nullable(ty: &str) -> Option<String> {
        let ty: Type = syn::parse_str(ty).expect("a type");
        super::option_inner(&ty).map(tokens)
    }

    #[test]
    fn the_three_spellings_of_option_are_nullable() {
        for spelling in [
            "Option<u8>",
            "std::option::Option<u8>",
            "core::option::Option<u8>",
            "::std::option::Option<u8>",
            "::core::option::Option<u8>",
        ] {
            assert_eq!(nullable(spelling).as_deref(), Some("u8"), "{spelling}");
        }
    }

    #[test]
    fn other_types_are_required() {
        for ty in [
            "u8",
            "Vec<u8>",
            "my::Option<u8>",
            "std::Option<u8>",
            "alloc::option::Option<u8>",
            "std::option::Maybe<u8>",
            "Option",
            "Option<u8, u8>",
            "Option<'a>",
            "::Option<u8>",
            "<T as Trait>::Option<u8>",
            "&Option<u8>",
            "MaybeName",
            "option::Option<u8>",
        ] {
            assert_eq!(nullable(ty), None, "{ty}");
        }
    }
}
