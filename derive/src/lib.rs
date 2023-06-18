use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Type};

#[proc_macro_derive(Resolver, attributes(resolver_target, to_string_resolver))]
pub fn derive_resolver(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input as DeriveInput);

    let (std_variant, to_string_variant) = if let Data::Enum(e) = data {
        let mut std_variants = Vec::new();
        let mut to_string_variants = Vec::new();
        for v in e.variants {
            let use_to_string = v
                .attrs
                .iter()
                .any(|a| a.path().is_ident("to_string_resolver"));
            if use_to_string {
                to_string_variants.push(v.ident);
            } else {
                std_variants.push(v.ident);
            }
        }
        (std_variants, to_string_variants)
    } else {
        panic!("expected request enum")
    };

    let attr = attrs
        .into_iter()
        .find(|attr| attr.path().is_ident("resolver_target"))
        .expect("did not find resolver_target attribute");

    let target: Type = attr
        .parse_args()
        .expect("should pass struct to implement resolve_request on, eg. AppState");

    quote! {
        #[async_trait::async_trait]
        impl resolver_api::Resolver<#ident> for #target {
            async fn resolve_request(&self, request: #ident) -> anyhow::Result<String> {
                match request {
                    #(#ident::#std_variant(req) => self.resolve_response(req).await,)*
                    #(#ident::#to_string_variant(req) => self.resolve_to_string(req).await,)*
                }
            }
        }
    }
    .into()
}

#[proc_macro_derive(Request, attributes(response))]
pub fn has_response(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let req = input.ident;

    let attr = input
        .attrs
        .into_iter()
        .find(|attr| attr.path().is_ident("response"))
        .expect("did not find response attribute");

    let res: Type = attr.parse_args().expect("should pass response type");

    quote! {
        impl resolver_api::HasResponse for #req {
            type Response = #res;
            fn req_type() -> &'static str {
                stringify!(#req)
            }
        }
    }
    .into()
}
