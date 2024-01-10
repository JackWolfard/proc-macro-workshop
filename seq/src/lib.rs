use proc_macro2::{Delimiter, Group, Literal, Spacing, TokenStream, TokenTree};
use quote::{format_ident, TokenStreamExt};
use std::ops::Range;
use syn::parse::{Parse, ParseStream};
use syn::{braced, parse_macro_input, token, Ident, LitInt, Result, Token};

#[derive(Debug)]
struct Sequence {
    ident: Ident,
    _in_token: Token![in],
    start: usize,
    _exclusive_range: Option<Token![..]>,
    _inclusive_range: Option<Token![..=]>,
    end: usize,
    _brace_token: token::Brace,
    content: TokenStream,
}

impl Parse for Sequence {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        let _in_token = input.parse()?;
        let start = input.parse::<LitInt>()?.base10_parse()?;
        let is_inclusive = input.peek(Token![..=]);
        let (_exclusive_range, _inclusive_range) = match is_inclusive {
            true => (None, Some(input.parse()?)),
            false => (Some(input.parse()?), None),
        };
        let end = input.parse::<LitInt>()?.base10_parse::<usize>()? + is_inclusive as usize;
        let content;
        Ok(Sequence {
            ident,
            _in_token,
            start,
            _exclusive_range,
            _inclusive_range,
            end,
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

    if has_repeat_annotation(content.clone()) {
        stream = repeat(content.clone(), &ident, start..end);
    } else {
        for i in start..end {
            let substitution = Literal::usize_unsuffixed(i);
            stream.append_all(substitute(content.clone(), &ident, &substitution));
        }
    }

    stream.into()
}

fn has_repeat_annotation(stream: TokenStream) -> bool {
    let tokens = Vec::from_iter(stream);

    let mut i = 0;
    while i < tokens.len() {
        if let TokenTree::Group(group) = &tokens[i] {
            if has_repeat_annotation(group.stream()) {
                return true;
            }
        } else if repeat_annotation(tokens.as_slice(), i).is_some() {
            return true;
        }
        i += 1;
    }
    false
}

fn repeat_annotation(tokens: &[TokenTree], i: usize) -> Option<TokenStream> {
    if let Some(slice) = tokens.get(i..i + 3) {
        if let Ok([TokenTree::Punct(pound), TokenTree::Group(group), TokenTree::Punct(star)]) =
            <&[TokenTree; 3]>::try_from(slice)
        {
            return (pound.as_char() == '#'
                && group.delimiter() == Delimiter::Parenthesis
                && star.as_char() == '*')
                .then_some(group.stream());
        }
    }
    None
}

fn repeat(stream: TokenStream, victim: &Ident, sequence: Range<usize>) -> TokenStream {
    let tokens = Vec::from_iter(stream);

    let mut output = Vec::<TokenTree>::with_capacity(tokens.len());

    let mut i = 0;
    while i < tokens.len() {
        if let TokenTree::Group(group) = &tokens[i] {
            let group_stream = repeat(group.stream(), victim, sequence.clone());
            output.push(Group::new(group.delimiter(), group_stream).into());
        } else if let Some(group_stream) = repeat_annotation(tokens.as_slice(), i) {
            for j in sequence.clone() {
                let substitution = Literal::usize_unsuffixed(j);
                output.extend(substitute(group_stream.clone(), victim, &substitution));
            }
            i += 2;
        } else {
            output.push(tokens[i].clone());
        }
        i += 1;
    }

    TokenStream::from_iter(output)
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
                    if let Some(slice) = tokens.get(i - 1..i + 2) {
                        if let Ok(
                            [Some(TokenTree::Ident(start)), .., Some(TokenTree::Ident(end))],
                        ) = <&[Option<TokenTree>; 3]>::try_from(slice)
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
            }
            _ => (),
        };
        i += 1;
    }

    TokenStream::from_iter(tokens.into_iter().flatten())
}
