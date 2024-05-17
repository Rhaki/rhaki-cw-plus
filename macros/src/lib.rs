use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_quote, DeriveInput, Meta};

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
