use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Data, DeriveInput, Field, Fields,
    GenericArgument, Ident, LitStr, PathArguments, Type,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let builder = format_ident!("{}Builder", name);
    let fields_decl = fields(&input.data).map(field_decl);
    let fields_none = fields(&input.data).map(field_none);
    let fields_setter = fields(&input.data).filter_map(field_setter);
    let fields_each_setter = fields(&input.data).filter_map(field_each_setter);
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
            #(#fields_each_setter)*
        }
    };

    expanded.into()
}

fn get_inner_ty<'a>(field: &'a Field, outer: &str) -> Option<&'a syn::Type> {
    if let Type::Path(ty) = &field.ty {
        if ty.path.segments.len() != 1 {
            return None;
        }
        let segment = ty.path.segments.first().unwrap();
        if segment.ident != outer {
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

fn get_builder_attr_each(field: &Field) -> Option<LitStr> {
    get_inner_ty(field, "Vec")?;
    if field.attrs.len() != 1 {
        return None;
    }
    let attr = field.attrs.first().unwrap();
    if !attr.path().is_ident("builder") {
        return None;
    }

    let mut s: Option<LitStr> = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("each") {
            let value = meta.value()?;
            s = Some(value.parse::<LitStr>()?);
            Ok(())
        } else {
            Err(meta.error("unsupported attribute"))
        }
    })
    .unwrap();

    s
}

fn field_each_setter(field: &Field) -> Option<TokenStream> {
    let name = field.ident.as_ref()?;
    let ty = get_inner_ty(field, "Vec")?;
    let lit = get_builder_attr_each(field)?;
    let item_name = Ident::new(&lit.value(), lit.span());
    Some(quote! {
        fn #item_name(&mut self, item: #ty) -> &mut Self {
            if let Some(ref mut #name) = self.#name {
                #name.push(item);
            } else {
                self.#name = Some(vec![item]);
            }
            self
        }
    })
}

fn build_field(field: &Field) -> TokenStream {
    let is_optional = get_inner_ty(field, "Option").is_some();
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

fn field_setter(field: &Field) -> Option<TokenStream> {
    let name = field.ident.as_ref()?;
    if let Some(lit) = get_builder_attr_each(field) {
        if *name == lit.value() {
            return None;
        }
    }
    let ty = get_inner_ty(field, "Option").unwrap_or(&field.ty);
    Some(quote! {
        fn #name(&mut self, #name: #ty) -> &mut Self {
            self.#name = Some(#name);
            self
        }
    })
}

fn field_decl(field: &Field) -> TokenStream {
    let ty = get_inner_ty(field, "Option").unwrap_or(&field.ty);
    if let Some(name) = &field.ident {
        return quote! {
            #name: Option<#ty>
        };
    }
    unimplemented!();
}

fn field_none(field: &Field) -> TokenStream {
    if let Some(name) = &field.ident {
        if get_builder_attr_each(field).is_some() {
            return quote! {
                #name: Some(Vec::new())
            };
        }

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
