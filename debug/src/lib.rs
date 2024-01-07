use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, LitStr};

struct Test {
    x: i32,
    y: i32,
}

impl std::fmt::Debug for Test {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Test")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}

#[proc_macro_derive(CustomDebug)]
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

fn field_debug(field: &Field) -> TokenStream {
    if let Some(name) = &field.ident {
        let lit_name = LitStr::new(&name.to_string(), name.span());
        return quote! {
            .field(#lit_name, &self.#name)
        };
    }
    unimplemented!();
}
