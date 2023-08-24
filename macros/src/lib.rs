use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput, parse_quote};

/// Similar to `cosmwasm_schema::cw_serde` but without the `schemars::JsonSchema` implementation.
/// 
/// Usefull on struct that use `rhaki_cw_utils::Value` (or `serde_cw_value::Value`)
#[proc_macro_attribute]
pub fn cw_serde_value(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
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
