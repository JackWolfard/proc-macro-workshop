use proc_macro2::{Literal, TokenStream, TokenTree};
use quote::TokenStreamExt;
use syn::parse::{Parse, ParseStream};
use syn::{braced, parse_macro_input, token, Ident, LitInt, Result, Token};

#[derive(Debug)]
struct Sequence {
    ident: Ident,
    _in_token: Token![in],
    start: usize,
    _dotdot_token: Token![..],
    end: usize,
    _brace_token: token::Brace,
    content: TokenStream,
}

impl Parse for Sequence {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Sequence {
            ident: input.parse()?,
            _in_token: input.parse()?,
            start: input.parse::<LitInt>()?.base10_parse()?,
            _dotdot_token: input.parse()?,
            end: input.parse::<LitInt>()?.base10_parse()?,
            _brace_token: braced!(content in input),
            content: content.parse::<TokenStream>()?,
        })
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let sequence = parse_macro_input!(input as Sequence);

    let Sequence {
        ident,
        start,
        end,
        content,
        ..
    } = sequence;

    let mut stream = TokenStream::new();

    for i in start..end {
        let substitution = Literal::usize_unsuffixed(i);
        stream.append_all(substitute(content.clone(), &ident, &substitution));
    }

    stream.into()
}

fn substitute(stream: TokenStream, victim: &Ident, substitution: &Literal) -> TokenStream {
    let mut tokens = Vec::from_iter(stream);

    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        let replacement: Option<TokenTree> = match token {
            TokenTree::Group(group) => {
                let stream = substitute(group.stream(), victim, substitution);
                let group = proc_macro2::Group::new(group.delimiter(), stream);
                Some(group.into())
            }
            TokenTree::Ident(ident) => ident.eq(victim).then_some(substitution.clone().into()),
            _ => None,
        };
        if let Some(replacement) = replacement {
            tokens[i] = replacement;
        }
        i += 1;
    }

    TokenStream::from_iter(tokens)
}
