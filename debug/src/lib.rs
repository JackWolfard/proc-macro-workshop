use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, LitStr};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let lit_name = LitStr::new(&name.to_string(), name.span());

    let fields_debug = fields(&input.data).map(field_debug);

    quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_struct(#lit_name)
                    #(#fields_debug)*
                    .finish()
            }
        }
    }
    .into()
}

fn fields(data: &Data) -> impl Iterator<Item = &Field> {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields.named.iter(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn debug_attr(field: &Field) -> Option<LitStr> {
    if field.attrs.len() != 1 {
        return None;
    }
    let attr = field.attrs.first().unwrap();
    if !attr.path().is_ident("debug") {
        return None;
    }
    if let syn::Meta::NameValue(meta) = &attr.meta {
        if let syn::Expr::Lit(expr) = &meta.value {
            if let syn::Lit::Str(lit_str) = &expr.lit {
                return Some(lit_str.clone());
            }
        }
    }
    None
}

fn field_debug(field: &Field) -> TokenStream {
    let debug = debug_attr(field);
    if let Some(name) = &field.ident {
        let lit_name = LitStr::new(&name.to_string(), name.span());
        if let Some(debug) = debug {
            return quote! {
                .field(#lit_name, &std::format_args!(#debug, &self.#name))
            };
        }
        return quote! {
            .field(#lit_name, &self.#name)
        };
    }
    unimplemented!();
}
