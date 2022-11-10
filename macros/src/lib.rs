mod trait_enum;
use proc_macro::TokenStream;
use trait_enum::expand_trait_enum;

#[proc_macro]
pub fn trait_enum(input: TokenStream) -> TokenStream {
    expand_trait_enum(input)
}
