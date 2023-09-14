use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ConfigStorage)]
pub fn config_storage_derive(input: TokenStream) -> TokenStream {
    // Parse the input into a DeriveInput struct
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the struct name
    let struct_name = &input.ident;

    // Generate the expanded code
    let expanded = quote! {
        impl #struct_name {
            pub fn load(store: &dyn Storage) -> StdResult<Self> {
                CONFIG.load(store)
            }

            pub fn save(&self, store: &mut dyn Storage) -> StdResult<()> {
                CONFIG.save(store, self)
            }
        }
    };

    // Return the generated code as a TokenStream
    TokenStream::from(expanded)
}
