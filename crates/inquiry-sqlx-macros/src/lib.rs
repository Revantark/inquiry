extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod attrs;
mod expand;
mod model;
mod sql;

#[proc_macro_derive(Queryable, attributes(query))]
pub fn queryable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let model = match model::parse(input) {
        Ok(model) => model,
        Err(error) => return error.to_compile_error().into(),
    };
    expand::derive_query(model).into()
}
