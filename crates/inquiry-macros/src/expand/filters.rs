use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::model::FieldInfo;

use super::context::{doc_lit, field_bind_bounds, ExpansionContext};

pub(super) fn expand_condition_type(cx: &ExpansionContext) -> TokenStream {
    let query_condition = &cx.query_condition;
    let query_error = &cx.query_error;
    let field_bind_bounds = field_bind_bounds(cx);
    let condition_variants = cx.fields.iter().map(|field| {
        let variant = condition_variant_ident(field);
        let ty = &field.ty;
        let operator = operator_type_for_field(cx, field);
        quote! {
            #variant(#operator, #ty)
        }
    });
    let condition_match_arms = cx.fields.iter().map(|field| {
        let variant = condition_variant_ident(field);
        let column_name = &field.column_name;
        quote! {
            Self::#variant(operator, value) => {
                sql.push_str(#column_name);
                sql.push_str(" ");
                sql.push_str(operator.as_sql());
                sql.push_str(" ");
                ::sqlx::Arguments::add(args, value)
                    .map_err(|error| #query_error::Database(::sqlx::Error::Encode(error)))?;
                ::sqlx::Arguments::format_placeholder(args, sql)
                    .expect("writing a SQL placeholder into a String should not fail");
            }
        }
    });

    quote! {
        #[derive(Clone, Debug)]
        enum #query_condition {
            #( #condition_variants, )*
            Any(::std::vec::Vec<#query_condition>),
        }

        impl #query_condition {
            fn is_empty(&self) -> bool {
                match self {
                    Self::Any(conditions) => conditions.is_empty(),
                    _ => false,
                }
            }

            fn append_sql<'args, T: ::sqlx::Database>(
                &'args self,
                sql: &mut ::std::string::String,
                args: &mut <T as ::sqlx::Database>::Arguments<'args>,
            ) -> Result<(), #query_error>
            where
                <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::Arguments<'args, Database = T>,
                #( #field_bind_bounds )*
            {
                match self {
                    #( #condition_match_arms )*
                    Self::Any(conditions) => {
                        sql.push_str("(");
                        let mut has_any_filters = false;

                        for condition in conditions {
                            if condition.is_empty() {
                                continue;
                            }

                            if has_any_filters {
                                sql.push_str(" OR ");
                            } else {
                                has_any_filters = true;
                            }

                            condition.append_sql::<T>(sql, args)?;
                        }

                        sql.push_str(")");
                    }
                }

                Ok(())
            }
        }
    }
}

pub(super) fn expand_group_type(cx: &ExpansionContext) -> TokenStream {
    let query_group = &cx.query_group;
    let query_condition = &cx.query_condition;
    let group_methods = cx.fields.iter().flat_map(|field| {
        let name = &field.ident;
        let ty = &field.ty;
        let variant = condition_variant_ident(field);
        let operator = operator_type_for_field(cx, field);
        let where_method = Ident::new(&format!("where_{}", name), name.span());
        let by_method = Ident::new(&format!("by_{}", name), name.span());
        let where_doc = doc_lit(format!(
            "Adds a condition for the `{}` field to this `OR` group.",
            name
        ));
        let by_doc = doc_lit(format!(
            "Adds an equality condition for the `{}` field to this `OR` group.",
            name
        ));

        [
            quote! {
                #[doc = #where_doc]
                pub fn #where_method(mut self, operator: #operator, value: #ty) -> Self {
                    self.conditions.push(#query_condition::#variant(operator, value));
                    self
                }
            },
            quote! {
                #[doc = #by_doc]
                pub fn #by_method(self, value: #ty) -> Self {
                    self.#where_method(#operator::Eq, value)
                }
            },
        ]
    });

    quote! {
        /// Builds grouped `OR` conditions for a query.
        pub struct #query_group {
            conditions: ::std::vec::Vec<#query_condition>,
        }

        impl #query_group {
            fn new() -> Self {
                Self {
                    conditions: ::std::vec::Vec::new(),
                }
            }

            #( #group_methods )*
        }
    }
}

pub(super) fn expand_filter_methods(cx: &ExpansionContext) -> TokenStream {
    let query_condition = &cx.query_condition;
    let query_group = &cx.query_group;
    let by_methods = cx.fields.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;
        let operator = operator_type_for_field(cx, field);
        let method_name = Ident::new(&format!("by_{}", name), name.span());
        let where_method = Ident::new(&format!("where_{}", name), name.span());
        let doc = doc_lit(format!(
            "Filters rows where the `{}` field equals the given value.",
            name
        ));

        quote! {
            #[doc = #doc]
            pub fn #method_name(self, value: #ty) -> Self {
                self.#where_method(#operator::Eq, value)
            }
        }
    });
    let where_methods = cx.fields.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;
        let variant = condition_variant_ident(field);
        let operator = operator_type_for_field(cx, field);
        let method_name = Ident::new(&format!("where_{}", name), name.span());
        let doc = doc_lit(format!(
            "Adds a condition for the `{}` field using the given operator and value.",
            name
        ));

        quote! {
            #[doc = #doc]
            pub fn #method_name(mut self, operator: #operator, value: #ty) -> Self {
                self.conditions.push(#query_condition::#variant(operator, value));
                self
            }
        }
    });

    quote! {
        #( #by_methods )*
        #( #where_methods )*

        /// Adds grouped `OR` conditions to this query.
        pub fn any(mut self, build: impl FnOnce(#query_group) -> #query_group) -> Self {
            let group = build(#query_group::new());

            if !group.conditions.is_empty() {
                self.conditions.push(#query_condition::Any(group.conditions));
            }

            self
        }
    }
}

fn condition_variant_ident(field: &FieldInfo) -> Ident {
    let field_name = field.ident.to_string();
    let mut variant_name = String::new();

    for part in field_name.split('_').filter(|part| !part.is_empty()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            variant_name.extend(first.to_uppercase());
            variant_name.push_str(chars.as_str());
        }
    }

    if variant_name.is_empty() {
        variant_name = field_name;
    }

    Ident::new(&variant_name, field.ident.span())
}

fn operator_type_for_field(cx: &ExpansionContext, field: &FieldInfo) -> TokenStream {
    let inquiry_path = &cx.inquiry_path;
    if field_supports_ordering(field) {
        quote! { #inquiry_path::QueryOrderingOperator }
    } else if field_supports_text_patterns(field) {
        quote! { #inquiry_path::QueryOperator }
    } else {
        quote! { #inquiry_path::QueryEqualityOperator }
    }
}

fn field_supports_text_patterns(field: &FieldInfo) -> bool {
    type_path_ident(field).is_some_and(|ident| ident == "String")
}

fn field_supports_ordering(field: &FieldInfo) -> bool {
    let Some(ident) = type_path_ident(field) else {
        return false;
    };

    matches!(ident.as_str(), "i16" | "i32" | "i64" | "f32" | "f64")
}

fn type_path_ident(field: &FieldInfo) -> Option<String> {
    let syn::Type::Path(type_path) = &field.ty else {
        return None;
    };

    Some(type_path.path.get_ident()?.to_string())
}
