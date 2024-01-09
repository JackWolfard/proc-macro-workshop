use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{braced, parse_macro_input, token, Ident, LitInt, Result, Token};

#[derive(Debug)]
struct Sequence {
    ident: Ident,
    in_token: Token![in],
    start: LitInt,
    dotdot_token: Token![..],
    end: LitInt,
    brace_token: token::Brace,
    content: TokenStream,
}

impl Parse for Sequence {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Sequence {
            ident: input.parse()?,
            in_token: input.parse()?,
            start: input.parse()?,
            dotdot_token: input.parse()?,
            end: input.parse()?,
            brace_token: braced!(content in input),
            content: content.parse::<TokenStream>()?,
        })
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let sequence = parse_macro_input!(input as Sequence);

    quote! {}.into()
}
