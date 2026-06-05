use proc_macro2::TokenStream;
use quote::quote;

use crate::model::ModelInfo;

mod context;
mod filters;
mod model_api;
mod persistence;
mod retrieval;
mod schema;

use context::ExpansionContext;

pub(crate) fn derive_query(model: ModelInfo) -> TokenStream {
    expand_query(ExpansionContext::new(model))
}

fn expand_query(cx: ExpansionContext) -> TokenStream {
    let condition_type = filters::expand_condition_type(&cx);
    let group_type = filters::expand_group_type(&cx);
    let error_type = expand_error_type(&cx);
    let query_struct = expand_query_struct(&cx);
    let clone_impl = expand_clone_impl(&cx);
    let query_impl = expand_query_impl(&cx);
    let model_impl = model_api::expand_model_impl(&cx);

    quote! {
        #condition_type
        #group_type
        #error_type
        #query_struct
        #clone_impl
        #query_impl
        #model_impl
    }
}

fn expand_error_type(cx: &ExpansionContext) -> TokenStream {
    let query_error = &cx.query_error;

    quote! {
        /// Errors returned by generated query methods.
        #[derive(Debug)]
        pub enum #query_error {
            /// The query requires at least one filter condition.
            NoFilters,
            /// The database returned an error while executing the query.
            Database(::sqlx::Error),
        }

        impl ::std::error::Error for #query_error {
            fn source(&self) -> ::core::option::Option<&(dyn ::std::error::Error + 'static)> {
                match self {
                    Self::Database(error) => ::core::option::Option::Some(error),
                    Self::NoFilters => ::core::option::Option::None,
                }
            }
        }

        impl ::core::fmt::Display for #query_error {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    Self::NoFilters => write!(f, "at least one query filter is required"),
                    Self::Database(error) => write!(f, "database error: {}", error),
                }
            }
        }
    }
}

fn expand_query_struct(cx: &ExpansionContext) -> TokenStream {
    let query_name = &cx.query_name;
    let query_condition = &cx.query_condition;

    quote! {
        /// Query builder generated for the model.
        pub struct #query_name<T: ::sqlx::Database> {
            conditions: ::std::vec::Vec<#query_condition>,
            pool: ::sqlx::Pool<T>,
        }
    }
}

fn expand_clone_impl(cx: &ExpansionContext) -> TokenStream {
    let query_name = &cx.query_name;
    let clone_bounds = cx.fields.iter().map(|field| {
        let ty = &field.ty;
        quote! {
            #ty: ::core::clone::Clone,
        }
    });

    quote! {
        impl<T: ::sqlx::Database> ::core::clone::Clone for #query_name<T>
        where
            #( #clone_bounds )*
        {
            fn clone(&self) -> Self {
                Self {
                    conditions: self.conditions.clone(),
                    pool: self.pool.clone(),
                }
            }
        }
    }
}

fn expand_query_impl(cx: &ExpansionContext) -> TokenStream {
    let query_name = &cx.query_name;
    let constructor = expand_constructor();
    let append_conditions = retrieval::expand_append_conditions_method(cx);
    let filters = filters::expand_filter_methods(cx);
    let create_table = schema::expand_create_table_method(cx);
    let inserts = persistence::expand_insert_methods(cx);
    let upsert = persistence::expand_upsert_method(cx);
    let update_many = persistence::expand_update_methods(cx);
    let deletes = persistence::expand_delete_methods(cx);
    let build_select_sql = retrieval::expand_build_select_sql_method(cx);
    let fetch = retrieval::expand_fetch_methods(cx);
    let count_exists = retrieval::expand_count_exists_methods(cx);

    quote! {
        impl<T: ::sqlx::Database> #query_name<T> {
            #constructor
            #append_conditions
            #filters
            #create_table
            #inserts
            #upsert
            #update_many
            #deletes
            #build_select_sql
            #fetch
            #count_exists
        }
    }
}

fn expand_constructor() -> TokenStream {
    quote! {
        /// Creates a new query builder backed by the given SQLx pool.
        pub fn new(pool: ::sqlx::Pool<T>) -> Self {
            Self {
                conditions: ::std::vec::Vec::new(),
                pool,
            }
        }
    }
}
