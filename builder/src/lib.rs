use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let builder = format_ident!("{}Builder", name);
    let fields_decl = fields(&input.data).map(field_decl);
    let fields_none = fields(&input.data).map(field_none);
    let fields_setter = fields(&input.data).map(field_setter);

    let expanded = quote! {
        impl #name {
            pub fn builder() -> #builder {
                #builder {
                    #(#fields_none),*
                }
            }
        }

        pub struct #builder {
            #(#fields_decl),*
        }

        impl #builder {
            #(#fields_setter)*
        }
    };

    expanded.into()
}

fn field_setter(field: &Field) -> TokenStream {
    let ty = &field.ty;
    if let Some(name) = &field.ident {
        return quote! {
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        };
    }
    unimplemented!();
}

fn field_decl(field: &Field) -> TokenStream {
    let ty = &field.ty;
    if let Some(name) = &field.ident {
        return quote! {
            #name: Option<#ty>
        };
    }
    unimplemented!();
}

fn field_none(field: &Field) -> TokenStream {
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
