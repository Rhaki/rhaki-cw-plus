use {
    core::panic,
    proc_macro::TokenStream,
    proc_macro2::{Ident, TokenTree},
    quote::{quote, ToTokens},
    syn::{
        parse_macro_input, parse_quote, Attribute, DeriveInput, GenericArgument, Meta,
        PathArguments, Type, TypePath,
    },
};

/// Similar to `cosmwasm_schema::cw_serde` but without the `schemars::JsonSchema` implementation.
///
/// Usefull on struct that use `rhaki_cw_utils::Value` (or `serde_cw_value::Value`)
#[proc_macro_attribute]
pub fn cw_serde_value(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = cw_serde_impl(input).into_token_stream();

    proc_macro::TokenStream::from(expanded)
}

fn cw_serde_impl(input: DeriveInput) -> DeriveInput {
    match input.data {
        syn::Data::Struct(_) => parse_quote! {
            #[derive(
                ::rhaki_cw_plus::_serde::Serialize,
                ::rhaki_cw_plus::_serde::Deserialize,
                ::std::clone::Clone,
                ::std::fmt::Debug,
                ::std::cmp::PartialEq,
            )]
            #[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
            #[serde(deny_unknown_fields, crate = "::rhaki_cw_plus::_serde")]
            #input
        },
        syn::Data::Enum(_) => parse_quote! {
            #[derive(
                ::rhaki_cw_plus::_serde::Serialize,
                ::rhaki_cw_plus::_serde::Deserialize,
                ::std::clone::Clone,
                ::std::fmt::Debug,
                ::std::cmp::PartialEq,
            )]
            #[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
            #[serde(deny_unknown_fields, rename_all = "snake_case", crate = "::rhaki_cw_plus::_serde")]
            #input
        },
        syn::Data::Union(_) => panic!("unions are not supported"),
    }
}

#[proc_macro_derive(Stargate, attributes(stargate))]
pub fn derive_stargate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let attributes = get_attr("stargate", &input.attrs).expect("stargate attribute not found");

    let mut query = None;

    let mut msgs = None;

    let mut name = None;

    if let Meta::List(list) = &attributes.meta {
        for (index, token) in list.tokens.clone().into_iter().enumerate() {
            if let TokenTree::Ident(ident) = token {
                if ident == "name" {
                    let a: Vec<TokenTree> = list.tokens.clone().into_iter().collect();
                    let a = a[index + 2].clone();

                    name = Some(quote! {#a})
                }
                if ident == "query_urls" {
                    let a: Vec<TokenTree> = list.tokens.clone().into_iter().collect();
                    let a = a[index + 2].clone();

                    query = Some(quote! {#a})
                }

                if ident == "msgs_urls" {
                    let a: Vec<TokenTree> = list.tokens.clone().into_iter().collect();
                    let a = a[index + 2].clone();
                    msgs = Some(quote! {#a})
                }
            }
        }
    }

    let query = query.expect("query_urls attribute not found");
    let msgs = msgs.expect("msgs_urls attribute not found");
    let name = name.expect("name attribute not found");

    let expanded = quote! {
        use strum::IntoEnumIterator;
        use cw_storage_plus::Item;

        impl #struct_name {
            pub const STARGATE_NAME: &'static str = #name;
        }

        impl StargateUrls for #struct_name {
            fn stargate_name(&self) -> String {
                #name.to_string()
            }

            fn is_query_type_url(&self, type_url: String) -> bool {
                #query::from_str(&type_url).is_ok()
            }

            fn is_msg_type_url(&self, type_url: String) -> bool {
                #msgs::from_str(&type_url).is_ok()
            }

            fn type_urls(&self) -> Vec<String> {
                let mut urls = Vec::new();
                urls.extend(#query::iter().map(|url| url.to_string()));
                urls.extend(#msgs::iter().map(|url| url.to_string()));
                urls
            }
        }

        impl Itemable for #struct_name {
            fn load(&self, storage: &dyn Storage) -> AnyResult<Box<dyn StargateApplication>> {
                Ok(Box::new(
                    Item::<Self>::new(&self.stargate_name())
                        .load(storage).unwrap_or_default()
                ))
            }

            fn save(&self, storage: &mut dyn Storage) -> AnyResult<()> {
                Item::<Self>::new(&self.stargate_name()).save(storage, &self)?;
                Ok(())
            }
        }

        impl ItemInterface for #struct_name {
            const NAMESPACE: &'static str = #name;
            const CONTRACT_NAME: &'static str = #name;
        }

    };

    TokenStream::from(expanded)
}

/// Implements following derive:
///
/// ```ignore
/// // Example
/// #[derive(strum_macros::EnumString, strum_macros::EnumIter, strum_macros::Display)]
/// pub enum Ics20MsgUrls {
///     #[strum(serialize = "/ibc.applications.transfer.v1.MsgTransfer")]
///     MsgTransfer,
///     ... // Others enum fields
/// }
#[proc_macro_attribute]
pub fn urls(_attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = quote! {
        #[derive(strum_macros::EnumString, strum_macros::EnumIter, strum_macros::Display)]
        #input
    };
    TokenStream::from(expanded)
}

fn get_attr<'a>(attr_ident: &str, attrs: &'a [syn::Attribute]) -> Option<&'a syn::Attribute> {
    attrs.iter().find(|&attr| {
        attr.path().segments.len() == 1 && attr.path().segments[0].ident == attr_ident
    })
}

// --- Optionalbale ---

/// Create another `Struct` with all fields as `Option<T>` where `T` is the original field type.
///
/// `optionable` attribute has to be provided before the `Struct` definition, with the following attributes:
/// - *(required)*`name`: the name of the `Struct` to be created;
/// - *(optional)*`derive`: the derives to be added to the `Struct`: ex: `derive(Clone, Debug)`;
/// - *(optional)*`attributes`: the attributes to be added to the `Struct`: ex: `attribute(cw_serde)`.
///
/// On each fiels of the `Struct`, is possible to use the `optionable` attribute as:
/// - `#[optionable(skip)]`: skip the field from the `Struct`.
///
/// If a field is alredy an `Option<T>`, the field will be transformed to `rhaki_cw_plus::utils::UpdateOption<T>`.
///
/// ## **Example**:
///
/// ```
/// use crate::Optionable;
/// use cosmwasm_schema::cw_serde;
///
/// #[derive(Optionable)]
/// #[optionable(name = UpdateConfig, attributes(cw_serde))]
/// pub struct Config {
///     pub foo: String,
///     #[optionable(skip)]
///     pub bar: u64,
/// }
///
/// let update_config = UpdateConfig {
///    foo: Some("foo".to_string())
/// };
/// ```
#[proc_macro_derive(Optionable, attributes(optionable))]
pub fn derive_option(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let attribute = get_attr("optionable", &input.attrs).expect("optionable attribute not found");
    let name_ident = get_attr_by_ident(&attribute, "name", 2).expect("name attribute not found");

    let name = if let TokenTree::Ident(name_ident) = name_ident {
        name_ident
    } else {
        panic!("name attribute is not ident: {name_ident:#?}")
    };

    let (derives, attributes) = get_derives_and_attributes(&attribute);

    let fields = if let syn::Data::Struct(data) = input.data.clone() {
        data.fields
    } else {
        panic!("not a struct");
    };

    let mut opt_fields = vec![];

    for field in &fields {
        let mut found = true;

        for attr in &field.attrs {
            if attr.path().is_ident("optionable") {
                if let Meta::List(meta_list) = &attr.meta {
                    let tokens = meta_list
                        .tokens
                        .clone()
                        .into_token_stream()
                        .into_iter()
                        .collect::<Vec<TokenTree>>();
                    if tokens.len() != 1 {
                        panic!("optionable attribute has to only one argument")
                    }

                    if let TokenTree::Ident(ident) = &tokens[0] {
                        let ident = ident.to_string();
                        match ident.as_str() {
                            "skip" => {
                                found = false;
                            }
                            _ => panic!("invalid optionable attribute: {ident}"),
                        }
                    } else {
                        panic!(
                            "optionable attribute has to be an ident, found {}",
                            tokens[0]
                        )
                    }
                }
            }
        }

        if found {
            let ident = &field.ident;
            let ty = &field.ty;

            let is_ty_option = get_inner_type_if_option(ty);

            if let Some(ty) = is_ty_option {
                opt_fields.push(quote! {
                    pub #ident: ::rhaki_cw_plus::utils::UpdateOption<#ty>
                })
            } else {
                opt_fields.push(quote! {
                    pub #ident: Option<#ty>
                })
            };
        }
    }

    let expanded = quote! {
        #[derive(#(#derives),*)]
        #(#[#attributes]),*
        pub struct #name {
            #(#opt_fields),*
        }
    };

    TokenStream::from(expanded)
}

// --- Smaller Twin ---

/// Create another `Struct` where some fields can be skipped.
///
/// `smaller_twin` attribute has to be provided before the `Struct` definition, with the following attributes:
/// - *(required)*`name`: the name of the `Struct` to be created;
/// - *(optional)*`derive`: the derives to be added to the `Struct`: ex: `derive(Clone, Debug)`;
/// - *(optional)*`attributes`: the attributes to be added to the `Struct`: ex: `attribute(cw_serde)`.
///
/// On each fiels of the `Struct`, is possible to use the `optionable` attribute as:
/// - `#[smaller_twin(skip)]`: skip the field from the `Struct`.
///
/// ## **Example**:
///
/// ```
/// use crate::SmallerTwin;
/// use cosmwasm_schema::cw_serde;
///
/// #[derive(SmallerTwin)]
/// #[smaller_twin(name = UpdateConfig, attributes(cw_serde))]
/// pub struct Config {
///     pub foo: String,
///     #[smaller_twin(skip)]
///     pub bar: u64,
/// }
///
/// let update_config = UpdateConfig {
///    foo: "foo".to_string()
/// };
/// ```
#[proc_macro_derive(SmallerTwin, attributes(smaller_twin))]
pub fn derive_smaller_twin(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let attribute =
        get_attr("smaller_twin", &input.attrs).expect("smaller_twin attribute not found");
    let name_ident = get_attr_by_ident(&attribute, "name", 2).expect("name attribute not found");

    let (derives, attributes) = get_derives_and_attributes(&attribute);

    let name = if let TokenTree::Ident(name_ident) = name_ident {
        name_ident
    } else {
        panic!("name attribute is not ident: {name_ident:#?}")
    };

    let fields = if let syn::Data::Struct(data) = input.data.clone() {
        data.fields
    } else {
        panic!("not a struct");
    };

    let mut smaller_fields = vec![];

    for field in &fields {
        let mut found = true;

        for attr in &field.attrs {
            if attr.path().is_ident("smaller_twin") {
                if let Meta::List(meta_list) = &attr.meta {
                    let tokens = meta_list
                        .tokens
                        .clone()
                        .into_token_stream()
                        .into_iter()
                        .collect::<Vec<TokenTree>>();
                    if tokens.len() != 1 {
                        panic!("smaller_twin attribute has to only one argument")
                    }

                    if let TokenTree::Ident(ident) = &tokens[0] {
                        let ident = ident.to_string();
                        match ident.as_str() {
                            "skip" => {
                                found = false;
                            }

                            _ => panic!("invalid smaller_twin attribute: {ident}"),
                        }
                    } else {
                        panic!(
                            "optionable attribute has to be an ident, found {}",
                            tokens[0]
                        )
                    }
                }
            }
        }

        if found {
            let ident = &field.ident;
            let ty = &field.ty;
            smaller_fields.push(quote! {
                pub #ident: #ty
            })
        }
    }

    let expanded = quote! {
        #[derive(#(#derives),*)]
        #(#[#attributes]),*
        pub struct #name {
            #(#smaller_fields),*
        }
    };

    TokenStream::from(expanded)
}

// --- Utils ---

fn get_attr_by_ident(attr: &Attribute, ident_name: &str, after: usize) -> Option<TokenTree> {
    if let Meta::List(list) = &attr.meta {
        let list_tokens = list.tokens.clone().into_iter().collect::<Vec<TokenTree>>();

        let idx = list_tokens.iter().position(|t| {
            if let TokenTree::Ident(ident) = t {
                if ident == ident_name {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        });

        if let Some(idx) = idx {
            Some(list_tokens[idx + after].clone())
        } else {
            None
        }
    } else {
        None
    }
}

fn get_inner_type_if_option(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.first() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(angle_bracketed_args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_type)) =
                        angle_bracketed_args.args.first()
                    {
                        return Some(inner_type);
                    }
                }
            }
        }
    }
    None
}

fn get_derives_and_attributes(attr: &Attribute) -> (Vec<Ident>, Vec<Ident>) {
    let mut derives = vec![];

    if let Some(derive_ident) = get_attr_by_ident(&attr, "derive", 1) {
        if let TokenTree::Group(group) = derive_ident {
            for dev in group.stream().into_iter() {
                if let TokenTree::Ident(ident) = dev {
                    derives.push(ident);
                }
            }
        }
    }

    let mut attributes = vec![];

    if let Some(attrs) = get_attr_by_ident(&attr, "attributes", 1) {
        if let TokenTree::Group(group) = attrs {
            for dev in group.stream().into_iter() {
                if let TokenTree::Ident(ident) = dev {
                    attributes.push(ident);
                }
            }
        }
    }

    (derives, attributes)
}
