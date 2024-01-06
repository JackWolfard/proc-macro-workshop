use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let builder = format_ident!("{}Builder", name);
    let builder_fields_decl = fields(&input.data).map(builder_field_decl);
    let builder_fields_none = fields(&input.data).map(builder_field_none);

    let expanded = quote! {
        impl #name {
            pub fn builder() -> #builder {
                #builder {
                    #(#builder_fields_none),*
                }
            }
        }

        pub struct #builder {
            #(#builder_fields_decl),*
        }
    };

    expanded.into()
}

fn builder_field_decl(field: &Field) -> TokenStream {
    let ty = &field.ty;
    if let Some(name) = &field.ident {
        return quote! {
            #name: Option<#ty>
        };
    }
    unimplemented!();
}

fn builder_field_none(field: &Field) -> TokenStream {
    if let Some(name) = &field.ident {
        return quote! {
            #name: None
        };
    }
    unimplemented!();
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
