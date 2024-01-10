use proc_macro2::{Literal, Spacing, TokenStream, TokenTree};
use quote::{format_ident, TokenStreamExt};
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
    let mut tokens = Vec::from_iter(stream.into_iter().map(Some));

    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        match token {
            Some(TokenTree::Group(group)) => {
                let stream = substitute(group.stream(), victim, substitution);
                let group = proc_macro2::Group::new(group.delimiter(), stream);
                tokens[i] = Some(group.into())
            }
            Some(TokenTree::Ident(ident)) => {
                if ident == victim {
                    tokens[i] = Some(substitution.clone().into());
                }
            }
            Some(TokenTree::Punct(punct)) => {
                if punct.as_char() == '~' && punct.spacing() == Spacing::Alone {
                    if let Ok([Some(TokenTree::Ident(start)), .., Some(TokenTree::Ident(end))]) =
                        <&[Option<TokenTree>; 3]>::try_from(&tokens[i - 1..i + 2])
                    {
                        if end == victim {
                            let ident = format_ident!("{start}{substitution}");
                            tokens[i - 1] = Some(ident.into());
                            tokens[i] = None;
                            tokens[i + 1] = None;
                        }
                    }
                }
            }
            _ => (),
        };
        i += 1;
    }

    TokenStream::from_iter(tokens.into_iter().flatten())
}
