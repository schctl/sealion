use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod de;

fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    de::impl_deserialize(input).into()
}
