use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Error, Expr, ExprLit, ExprRange, Lit, LitInt,
    RangeLimits, Result,
};

#[proc_macro_attribute]
pub fn bitfield(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let _ = input;

    unimplemented!()
}

#[proc_macro]
pub fn bit_specifier(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ExprRange);

    match bit_specifier_impl(input) {
        Ok(output) => output,
        Err(error) => error.into_compile_error(),
    }
    .into()
}

fn bit_specifier_impl(range: ExprRange) -> Result<TokenStream> {
    let error_span = range.span();
    let ExprRange {
        start, limits, end, ..
    } = range;
    match (start.as_deref(), end.as_deref()) {
        (
            Some(Expr::Lit(ExprLit {
                lit: Lit::Int(start),
                ..
            })),
            Some(Expr::Lit(ExprLit {
                lit: Lit::Int(end), ..
            })),
        ) => {
            let start = start.base10_parse::<usize>()?;
            let end = end.base10_parse::<usize>()?;
            let end = match limits {
                RangeLimits::HalfOpen(_) => end,
                RangeLimits::Closed(_) => end + 1,
            };
            let bits = (start..end).map(|b| {
                let ident = format_ident!("B{b}");
                let lit: LitInt = parse_quote!(#b);
                quote! {
                    pub enum #ident {}

                    impl Specifier for #ident {
                        const BITS: usize = #lit;
                    }
                }
            });
            Ok(quote! {
                #(#bits)*
            })
        }
        _ => Err(Error::new(error_span, "expected literal range of ints")),
    }
}
