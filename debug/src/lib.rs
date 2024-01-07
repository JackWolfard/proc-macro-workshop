use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, AngleBracketedGenericArguments, Data, DeriveInput, Field,
    Fields, GenericArgument, GenericParam, Generics, Ident, LitStr, PathArguments, Type,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let lit_name = LitStr::new(&name.to_string(), name.span());

    let generics = add_trait_bounds(input.generics, &input.data);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let fields_debug = fields(&input.data).map(field_debug);

    quote! {
        impl #impl_generics std::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_struct(#lit_name)
                    #(#fields_debug)*
                    .finish()
            }
        }
    }
    .into()
}

fn add_trait_bounds(mut generics: Generics, data: &Data) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = param {
            if !fields(data)
                .map(|field| is_phantom_generic_ty(field, &type_param.ident))
                .fold(false, |a, b| a || b)
            {
                type_param.bounds.push(parse_quote!(std::fmt::Debug));
            }
        }
    }
    generics
}

fn is_phantom_generic_ty(field: &Field, generic_ty: &Ident) -> bool {
    if let Some(ty) = get_inner_ty(field, "PhantomData") {
        if let Type::Path(ty) = ty {
            return ty.path.get_ident().is_some_and(|ty| ty == generic_ty);
        }
    }
    false
}

fn get_inner_ty<'a>(field: &'a Field, outer: &str) -> Option<&'a Type> {
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
