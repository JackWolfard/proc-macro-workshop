use quote::quote;
use syn::{parse_macro_input, Error, Item, Result};

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
        Ok(item.into())
    } else {
        Err(Error::new_spanned(
            item,
            "#[sorted] cannot be applied to things other than enum",
        ))
    }
}
