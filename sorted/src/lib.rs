use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input,
    visit_mut::{self, VisitMut},
    Attribute, Error, ExprMatch, Ident, Item, ItemFn, Meta, Pat, Path, Result,
};

#[proc_macro_attribute]
pub fn sorted(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Item);

    let output = sorted_impl(Sort::Item(input.clone()));
    match output {
        Ok(output) => quote! {
            #output
        },
        Err(output) => {
            let output = output.into_compile_error();
            quote! {
                    #output
                    #input
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn check(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as ItemFn);

    let errors = check_impl(&mut input);
    let errors = errors.map(|e| e.into_compile_error());
    quote! {
        #(#errors)*
        #input
    }
    .into()
}

#[derive(Debug)]
enum Sort {
    Item(Item),
    Match(ExprMatch),
}

impl ToTokens for Sort {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Sort::Item(item) => item.to_tokens(tokens),
            Sort::Match(expr_match) => expr_match.to_tokens(tokens),
        }
    }
}

#[derive(Debug)]
enum SortElement {
    Ident(Ident),
    Path(Path),
}

impl PartialEq for SortElement {
    fn eq(&self, other: &Self) -> bool {
        self.to_string().eq(&other.to_string())
    }
}

impl PartialOrd for SortElement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.to_string().partial_cmp(&other.to_string())
    }
}

impl ToTokens for SortElement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SortElement::Ident(ident) => ident.to_tokens(tokens),
            SortElement::Path(path) => path.to_tokens(tokens),
        }
    }
}

impl Display for SortElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortElement::Ident(ident) => write!(f, "{ident}"),
            SortElement::Path(path) => {
                if path.leading_colon.is_some() {
                    write!(f, "::")?;
                }
                if let Some(segment) = path.segments.first() {
                    write!(f, "{}", segment.ident)?;
                }
                for segment in path.segments.iter().skip(1) {
                    write!(f, "::{}", segment.ident)?;
                }
                Ok(())
            }
        }
    }
}

fn sorted_impl(sort: Sort) -> Result<Sort> {
    let mut variants: Vec<SortElement> = Vec::new();
    match sort {
        Sort::Item(ref item) => {
            if let Item::Enum(item) = item {
                variants.extend(
                    item.variants
                        .iter()
                        .map(|v| SortElement::Ident(v.ident.clone())),
                );
            } else {
                return Err(Error::new_spanned(
                    item,
                    "#[sorted] cannot be applied to things other than enum or match expressions",
                ));
            }
        }
        Sort::Match(ref m) => {
            variants.extend(
                m.arms
                    .iter()
                    .map(|a| match a.pat {
                        Pat::Path(ref pat) => Ok(SortElement::Path(pat.path.clone())),
                        Pat::Struct(ref pat) => Ok(SortElement::Path(pat.path.clone())),
                        Pat::TupleStruct(ref pat) => Ok(SortElement::Path(pat.path.clone())),
                        _ => Err(Error::new_spanned(
                            a,
                            "#[sorted] cannot handle this pattern",
                        )),
                    })
                    .collect::<Result<Vec<_>>>()?,
            );
        }
    }
    if let Some(out_of_order) = variants
        .iter()
        .zip(variants.iter().skip(1))
        .fold(None, |acc, (a, b)| acc.or(a.gt(b).then_some(b)))
    {
        if let Some(order_before) = variants
            .iter()
            .fold(None, |acc, a| acc.or(a.gt(out_of_order).then_some(a)))
        {
            return Err(Error::new_spanned(
                out_of_order,
                format!("{out_of_order} should sort before {order_before}"),
            ));
        }
    }
    Ok(sort)
}

#[derive(Debug)]
struct SortMatchVisitor {
    errors: Vec<Error>,
}

impl SortMatchVisitor {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }
}

impl VisitMut for SortMatchVisitor {
    fn visit_expr_match_mut(&mut self, node: &mut ExprMatch) {
        let mut i = 0;
        let mut sorted_attr_pos = None;
        while i < node.attrs.len() {
            let attr = &node.attrs[i];
            if is_sorted_attr(attr) {
                if let Err(err) = sorted_impl(Sort::Match(node.clone())) {
                    self.errors.push(err);
                }
                sorted_attr_pos = Some(i);
            }
            i += 1;
        }
        if let Some(i) = sorted_attr_pos {
            node.attrs.swap_remove(i);
        }
        visit_mut::visit_expr_match_mut(self, node);
    }
}

fn is_sorted_attr(attr: &Attribute) -> bool {
    if let Meta::Path(ref path) = attr.meta {
        return path.is_ident("sorted");
    }
    false
}

fn check_impl(item: &mut ItemFn) -> impl Iterator<Item = Error> {
    let mut visitor = SortMatchVisitor::new();
    visitor.visit_item_fn_mut(item);
    visitor.errors.into_iter()
}
