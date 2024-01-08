use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, AngleBracketedGenericArguments, Attribute, Data, DeriveInput,
    Field, Fields, GenericArgument, GenericParam, Generics, Ident, LitStr, PathArguments, Type,
    TypePath, WherePredicate,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let bound: Option<DebugBound> = match input
        .attrs
        .iter()
        .map(outer_debug_attr)
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(predicates) => predicates.into_iter().fold(None, |a, b| a.or(b)),
        Err(err) => {
            let err = err.to_compile_error();
            return quote! {
                #err
            }
            .into();
        }
    };

    let name = input.ident;
    let lit_name = LitStr::new(&name.to_string(), name.span());

    let generics = add_trait_bounds(input.generics, &input.data, bound);
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

struct DebugBound {
    generic: Option<Ident>,
    predicate: WherePredicate,
}

fn outer_debug_attr(attr: &Attribute) -> syn::Result<Option<DebugBound>> {
    if attr.path().is_ident("debug") {
        let mut bound: Option<DebugBound> = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("bound") {
                let value = meta.value()?;
                let lit = value.parse::<LitStr>()?;
                let predicate = syn::parse_str::<WherePredicate>(&lit.value())?;
                let generic: Option<Ident> = match predicate {
                    WherePredicate::Type(ref p_ty) => match &p_ty.bounded_ty {
                        Type::Path(TypePath { path, .. }) => {
                            if path.segments.len() > 1 {
                                let segment = path.segments.first().unwrap();
                                Some(segment.ident.clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    },
                    _ => None,
                };
                bound = Some(DebugBound { generic, predicate });
            }
            Ok(())
        })?;
        return Ok(bound);
    }
    Ok(None)
}

fn add_trait_bounds(mut generics: Generics, data: &Data, bound: Option<DebugBound>) -> Generics {
    generics.make_where_clause();
    if let Some(where_clause) = generics.where_clause.as_mut() {
        if let Some(DebugBound { predicate, .. }) = bound.as_ref() {
            where_clause.predicates.push(predicate.clone());
        }
        for param in &mut generics.params {
            if let GenericParam::Type(ref mut type_param) = param {
                let phantom_data = fields(data)
                    .map(|field| is_phantom_generic_ty(field, &type_param.ident))
                    .any(|a| a);
                let associated_types: Vec<&Type> = fields(data)
                    .filter_map(|field| get_associated_ty(field, &type_param.ident))
                    .collect();
                let bound_attr = bound.as_ref().is_some_and(|DebugBound { generic, .. }| {
                    generic.as_ref().is_some_and(|g| *g == type_param.ident)
                });
                if !phantom_data && associated_types.is_empty() && !bound_attr {
                    type_param.bounds.push(parse_quote!(std::fmt::Debug));
                } else {
                    associated_types.iter().for_each(|ty| {
                        where_clause
                            .predicates
                            .push(parse_quote!(#ty: std::fmt::Debug))
                    });
                }
            }
        }
    }
    generics
}

fn is_phantom_generic_ty(field: &Field, generic_ty: &Ident) -> bool {
    if let Some(Type::Path(ty)) = get_inner_ty(field, "PhantomData") {
        return ty.path.get_ident().is_some_and(|ty| ty == generic_ty);
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

fn get_associated_ty<'a>(field: &'a Field, generic_ty: &Ident) -> Option<&'a Type> {
    if let Type::Path(ty) = &field.ty {
        if ty.path.segments.len() != 1 {
            return None;
        }
        let segment = ty.path.segments.first().unwrap();
        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
            &segment.arguments
        {
            if args.len() != 1 {
                return None;
            }
            let generic_arg = args.first().unwrap();
            if let GenericArgument::Type(ty) = generic_arg {
                if let Type::Path(TypePath { path, .. }) = ty {
                    let segment = path.segments.first()?;
                    if segment.ident == *generic_ty && path.segments.len() > 1 {
                        return Some(ty);
                    }
                }
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
