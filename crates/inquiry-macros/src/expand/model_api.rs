use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::context::{doc_lit, ExpansionContext};

pub(super) fn expand_model_impl(cx: &ExpansionContext) -> TokenStream {
    let struct_name = &cx.struct_name;
    let query_name = &cx.query_name;
    let setters = cx.fields.iter().filter(|field| !field.primary_key).map(|field| {
        let name = &field.ident;
        let ty = &field.ty;
        let method_name = Ident::new(&format!("set_{}", name), name.span());
        let doc = doc_lit(format!(
            "Sets the `{}` field and returns the updated model.",
            name
        ));

        quote! {
            #[doc = #doc]
            pub fn #method_name(mut self, value: #ty) -> Self {
                self.#name = value;
                self
            }
        }
    });

    quote! {
        impl #struct_name {
            /// Creates a query builder for this model using the given SQLx pool.
            pub fn query<T: ::sqlx::Database>(pool: ::sqlx::Pool<T>) -> #query_name<T> {
                #query_name::new(pool)
            }

            #( #setters )*
        }
    }
}
