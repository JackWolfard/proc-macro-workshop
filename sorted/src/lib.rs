use quote::quote;
use syn::{parse_macro_input, Error, Ident, Item, Result};

#[proc_macro_attribute]
pub fn sorted(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Item);

    let output = match sorted_impl(input) {
        Ok(output) => quote! {
            #output
        },
        Err(output) => output.into_compile_error(),
    };

    output.into()
}

fn sorted_impl(item: Item) -> Result<Item> {
    if let Item::Enum(item) = item {
        let variants: Vec<&Ident> = item.variants.iter().map(|v| &v.ident).collect();
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
        Ok(item.into())
    } else {
        Err(Error::new_spanned(
            item,
            "#[sorted] cannot be applied to things other than enum",
        ))
    }
}
