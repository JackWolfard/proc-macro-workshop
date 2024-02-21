use proc_macro2::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Expr, ExprLit, ExprRange, Field, Ident, Item, ItemStruct, Lit, LitInt, RangeLimits,
    Result, Token, Type,
};

#[proc_macro_attribute]
pub fn bitfield(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Item);

    match bitfield_impl(input) {
        Ok(output) => output,
        Err(error) => error.into_compile_error(),
    }
    .into()
}

fn bitfield_impl(item: Item) -> Result<TokenStream> {
    match item {
        Item::Struct(item) => {
            let ident = &item.ident;
            let mut size = TokenStream::new();
            let sizes = fields(&item)?.map(|f| {
                let ty = &f.ty;
                quote! {
                    #ty::BITS
                }
            });
            let plus: Token![+] = parse_quote!(+);
            size.append_separated(sizes, plus);
            let mut punct = Punctuated::<Type, Token![+]>::new();
            punct.push(parse_quote!(bitfield::Zero::BITS));
            let methods = fields(&item)?
                .scan(punct, |offset, f| {
                    if let Some(ref ident) = f.ident {
                        let ty = &f.ty;
                        let getter = format_ident!("get_{ident}");
                        let setter = format_ident!("set_{ident}");
                        let output = Some(Ok(quote! {
                            fn #getter(&self) -> u64 {
                                self.get::<#ty>(#offset)
                            }

                            fn #setter(&mut self, value: u64) {
                                self.set::<#ty>(#offset, value);
                            }
                        }));
                        offset.push(parse_quote!(#ty::BITS));
                        output
                    } else {
                        Some(Err(Error::new(
                            f.span(),
                            "expected field to have identifier",
                        )))
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(quote! {
                #[repr(C)]
                #[derive(Default)]
                pub struct #ident {
                    data: [u8; (#size) / 8],
                }

                impl #ident {
                    fn new() -> Self {
                        Self::default()
                    }

                    fn get<T: Specifier>(&self, offset: usize) -> u64 {
                        let start_byte = offset / 8;
                        let start_offset = offset % 8;
                        let mut data = 0;
                        let bits_to_read = T::BITS;
                        let mut bits_read = 0;
                        let mut i = 0;
                        while bits_read < bits_to_read {
                            let mut byte = self.data[start_byte + i] as u64;
                            let mut bits_reading = 8;
                            if i == 0 {
                                byte >>= start_offset;
                                bits_reading -= start_offset;
                            }
                            let bits_left = bits_to_read - bits_read;
                            if bits_left <= 8 && bits_left < bits_reading {
                                bits_reading = bits_left;
                            }
                            let mask = (1 << bits_reading) - 1;
                            data |= (byte & mask) << bits_read;
                            bits_read += bits_reading;
                            i += 1;
                        }
                        data
                    }

                    fn set<T: Specifier>(&mut self, offset: usize, mut value: u64) {
                        let start_byte = offset / 8;
                        let start_offset = offset % 8;
                        let bits_to_write = T::BITS;
                        let mut bits_written = 0;
                        let mut i = 0;
                        while bits_written < bits_to_write {
                            let mut bits_reading = 8;
                            let mut shift = 0;
                            if i == 0 {
                                bits_reading -= start_offset;
                                shift = start_offset;
                            }
                            let bits_left = bits_to_write - bits_written;
                            if bits_left <= 8 && bits_left < bits_reading {
                                bits_reading = bits_left;
                            }
                            let mask = (1 << bits_reading) - 1;
                            let byte = value & mask;
                            value >>= bits_reading;
                            self.data[start_byte + i] &= !((mask << shift) as u8);
                            self.data[start_byte + i] |= (byte << shift) as u8;
                            bits_written += bits_reading;
                            i += 1;
                        }
                    }

                    #(#methods)*
                }

                impl bitfield::checks::CheckTotalSizeIsMultipleOf8 for #ident {
                    type Size = bitfield::checks::TotalSize<[(); (#size) % 8]>;
                }
            })
        }
        _ => Err(Error::new(item.span(), "expected struct")),
    }
}

fn fields(item: &ItemStruct) -> Result<impl Iterator<Item = &Field>> {
    match &item.fields {
        syn::Fields::Named(fields) => Ok(fields.named.iter()),
        _ => Err(Error::new(
            item.fields.span(),
            "expected fields to be named",
        )),
    }
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

struct MultipleOf8 {
    lit: LitInt,
    _comma: Token![,],
    ident: Ident,
}

impl Parse for MultipleOf8 {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(MultipleOf8 {
            lit: input.parse()?,
            _comma: input.parse()?,
            ident: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn multiple_of_8(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let MultipleOf8 { lit, ident, .. } = parse_macro_input!(input as MultipleOf8);

    quote! {
        pub enum #ident {}

        impl KnownSize for TotalSize<[(); #lit]> {
            type Check = #ident;
        }
    }
    .into()
}
