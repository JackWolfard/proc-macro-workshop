use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Data, DeriveInput, Field, Fields,
    GenericArgument, LitStr, PathArguments, Type,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let builder = format_ident!("{}Builder", name);
    let fields_decl = fields(&input.data).map(field_decl);
    let fields_none = fields(&input.data).map(field_none);
    let fields_setter = fields(&input.data).map(field_setter);
    let build_fields = fields(&input.data).map(build_field);

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
            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields),*
                })
            }

            #(#fields_setter)*
        }
    };

    expanded.into()
}

fn get_option_ty(field: &Field) -> Option<&syn::Type> {
    if let Type::Path(ty) = &field.ty {
        if ty.path.segments.len() != 1 {
            return None;
        }
        let segment = ty.path.segments.first().unwrap();
        if segment.ident.to_string() != "Option" {
            return None;
        }
        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
            &segment.arguments
        {
            if args.len() != 1 {
                return None;
            }
            let generic_arg = args.first().unwrap();
            if let GenericArgument::Type(ty) = generic_arg {
                return Some(ty);
            }
        }
    }
    None
}

fn build_field(field: &Field) -> TokenStream {
    let is_optional = get_option_ty(field).is_some();
    if let Some(name) = &field.ident {
        if is_optional {
            return quote! {
                #name: self.#name.clone()
            };
        }
        let err_msg = LitStr::new(
            &format!("expected '{}' to have been set", name),
            name.span(),
        );
        return quote! {
            #name: self.#name.clone().ok_or(#err_msg)?
        };
    }
    unimplemented!();
}

fn field_setter(field: &Field) -> TokenStream {
    let ty = get_option_ty(field).unwrap_or(&field.ty);
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
    let ty = get_option_ty(field).unwrap_or(&field.ty);
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
