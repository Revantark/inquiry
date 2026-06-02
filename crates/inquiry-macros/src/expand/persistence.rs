use proc_macro2::TokenStream;
use quote::quote;

use crate::sql;

use super::context::{field_bind_bounds, ExpansionContext};

pub(super) fn expand_insert_methods(cx: &ExpansionContext) -> TokenStream {
    let struct_name = &cx.struct_name;
    let table_name = &cx.table_name;
    let insert_fields = &cx.insert_fields;
    let query_error = &cx.query_error;
    let field_bind_bounds = field_bind_bounds(cx);
    let insert_value_binds = insert_value_binds(cx);

    quote! {
        /// Inserts one model row into the backing table.
        pub async fn add_one(&self, value: #struct_name) -> Result<(), #query_error>
        where
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::IntoArguments<'args, T>,
            for<'c> &'c mut T::Connection: ::sqlx::Executor<'c, Database = T>,
            #( #field_bind_bounds )*
        {
            let mut sql = ::std::string::String::from("INSERT INTO ");
            sql.push_str(#table_name);
            sql.push_str(" (");
            sql.push_str(#insert_fields);
            sql.push_str(") VALUES (");
            let mut args = <T as ::sqlx::Database>::Arguments::default();

            #( #insert_value_binds )*

            sql.push_str(")");

            let mut conn = self.pool.acquire().await.map_err(#query_error::Database)?;
            ::sqlx::query_with::<T, _>(&sql, args)
                .execute(&mut *conn)
                .await
                .map_err(#query_error::Database)?;

            Ok(())
        }

        /// Inserts many model rows into the backing table.
        pub async fn add_many(&self, values: ::std::vec::Vec<#struct_name>) -> Result<(), #query_error>
        where
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::IntoArguments<'args, T>,
            for<'c> &'c mut T::Connection: ::sqlx::Executor<'c, Database = T>,
            #( #field_bind_bounds )*
        {
            if values.is_empty() {
                return Ok(());
            }

            let mut sql = ::std::string::String::from("INSERT INTO ");
            sql.push_str(#table_name);
            sql.push_str(" (");
            sql.push_str(#insert_fields);
            sql.push_str(") VALUES ");
            let mut args = <T as ::sqlx::Database>::Arguments::default();

            for (row_index, value) in values.iter().enumerate() {
                if row_index > 0 {
                    sql.push_str(", ");
                }

                sql.push_str("(");
                #( #insert_value_binds )*
                sql.push_str(")");
            }

            let mut conn = self.pool.acquire().await.map_err(#query_error::Database)?;
            ::sqlx::query_with::<T, _>(&sql, args)
                .execute(&mut *conn)
                .await
                .map_err(#query_error::Database)?;

            Ok(())
        }
    }
}

pub(super) fn expand_upsert_method(cx: &ExpansionContext) -> TokenStream {
    let struct_name = &cx.struct_name;
    let table_name = &cx.table_name;
    let insert_fields = &cx.insert_fields;
    let query_error = &cx.query_error;
    let field_bind_bounds = field_bind_bounds(cx);
    let insert_value_binds = insert_value_binds(cx);

    if let Some(primary_key_field) = cx.primary_key() {
        let primary_key_column = &primary_key_field.column_name;
        let conflict_action = if cx.has_non_primary_key_fields() {
            let update_assignments = sql::excluded_update_assignments(&cx.fields);
            quote! {
                sql.push_str(") DO UPDATE SET ");
                sql.push_str(#update_assignments);
            }
        } else {
            quote! {
                sql.push_str(") DO NOTHING");
            }
        };

        quote! {
            fn build_upsert_one_sql<'args>(
                &'args self,
                value: &'args #struct_name,
            ) -> Result<
                (
                    ::std::string::String,
                    <T as ::sqlx::Database>::Arguments<'args>,
                ),
                #query_error,
            >
            where
                <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
                #( #field_bind_bounds )*
            {
                let mut sql = ::std::string::String::from("INSERT INTO ");
                sql.push_str(#table_name);
                sql.push_str(" (");
                sql.push_str(#insert_fields);
                sql.push_str(") VALUES (");
                let mut args = <T as ::sqlx::Database>::Arguments::default();

                #( #insert_value_binds )*

                sql.push_str(") ON CONFLICT (");
                sql.push_str(#primary_key_column);
                #conflict_action

                Ok((sql, args))
            }

            /// Inserts one row, or updates the existing row with the same primary key.
            pub async fn upsert_one(&self, value: #struct_name) -> Result<(), #query_error>
            where
                for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
                for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::IntoArguments<'args, T>,
                for<'c> &'c mut T::Connection: ::sqlx::Executor<'c, Database = T>,
                #( #field_bind_bounds )*
            {
                let (sql, args) = self.build_upsert_one_sql(&value)?;

                let mut conn = self.pool.acquire().await.map_err(#query_error::Database)?;
                ::sqlx::query_with::<T, _>(&sql, args)
                    .execute(&mut *conn)
                    .await
                    .map_err(#query_error::Database)?;

                Ok(())
            }
        }
    } else {
        quote! {}
    }
}

pub(super) fn expand_update_methods(cx: &ExpansionContext) -> TokenStream {
    let struct_name = &cx.struct_name;
    let table_name = &cx.table_name;
    let query_error = &cx.query_error;
    let field_bind_bounds = field_bind_bounds(cx);

    if let Some(primary_key_field) = cx.primary_key() {
        let primary_key_name = &primary_key_field.ident;
        let primary_key_column = &primary_key_field.column_name;
        let update_assignments = update_assignments(cx);

        if !cx.has_non_primary_key_fields() {
            return quote! {
                fn build_update_one_sql<'args>(
                    &'args self,
                    _value: &'args #struct_name,
                ) -> Result<
                    ::core::option::Option<(
                        ::std::string::String,
                        <T as ::sqlx::Database>::Arguments<'args>,
                    )>,
                    #query_error,
                >
                where
                    <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
                {
                    Ok(::core::option::Option::None)
                }

                /// Updates one row by matching the value on the primary key.
                pub async fn update_one(&self, _value: #struct_name) -> Result<(), #query_error>
                {
                    Ok(())
                }

                /// Updates many rows by matching each value on the primary key.
                pub async fn update_many(&self, _values: ::std::vec::Vec<#struct_name>) -> Result<(), #query_error>
                {
                    Ok(())
                }
            };
        }

        quote! {
            fn build_update_one_sql<'args>(
                &'args self,
                value: &'args #struct_name,
            ) -> Result<
                ::core::option::Option<(
                    ::std::string::String,
                    <T as ::sqlx::Database>::Arguments<'args>,
                )>,
                #query_error,
            >
            where
                <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
                #( #field_bind_bounds )*
            {
                let mut sql = ::std::string::String::from("UPDATE ");
                sql.push_str(#table_name);
                sql.push_str(" SET ");
                let mut args = <T as ::sqlx::Database>::Arguments::default();

                #( #update_assignments )*

                sql.push_str(" WHERE ");
                sql.push_str(#primary_key_column);
                sql.push_str(" = ");
                ::sqlx::Arguments::add(&mut args, &value.#primary_key_name)
                    .map_err(|error| #query_error::Database(::sqlx::Error::Encode(error)))?;
                ::sqlx::Arguments::format_placeholder(&args, &mut sql)
                    .expect("writing a SQL placeholder into a String should not fail");

                Ok(::core::option::Option::Some((sql, args)))
            }

            /// Updates one row by matching the value on the primary key.
            pub async fn update_one(&self, value: #struct_name) -> Result<(), #query_error>
            where
                for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
                for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::IntoArguments<'args, T>,
                for<'c> &'c mut T::Connection: ::sqlx::Executor<'c, Database = T>,
                #( #field_bind_bounds )*
            {
                let Some((sql, args)) = self.build_update_one_sql(&value)? else {
                    return Ok(());
                };

                let mut conn = self.pool.acquire().await.map_err(#query_error::Database)?;
                ::sqlx::query_with::<T, _>(&sql, args)
                    .execute(&mut *conn)
                    .await
                    .map_err(#query_error::Database)?;

                Ok(())
            }

            /// Updates many rows by matching each value on the primary key.
            pub async fn update_many(&self, values: ::std::vec::Vec<#struct_name>) -> Result<(), #query_error>
            where
                for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
                for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::IntoArguments<'args, T>,
                for<'c> &'c mut T::Connection: ::sqlx::Executor<'c, Database = T>,
                #( #field_bind_bounds )*
            {
                let mut conn = self.pool.acquire().await.map_err(#query_error::Database)?;

                for value in values {
                    let Some((sql, args)) = self.build_update_one_sql(&value)? else {
                        continue;
                    };

                    ::sqlx::query_with::<T, _>(&sql, args)
                        .execute(&mut *conn)
                        .await
                        .map_err(#query_error::Database)?;
                }

                Ok(())
            }
        }
    } else {
        quote! {}
    }
}

pub(super) fn expand_delete_methods(cx: &ExpansionContext) -> TokenStream {
    let table_name = &cx.table_name;
    let query_error = &cx.query_error;
    let field_bind_bounds = field_bind_bounds(cx);

    quote! {
        fn build_delete_sql<'args>(
            &'args self,
            limit: ::core::option::Option<u32>,
        ) -> Result<
            (
                ::std::string::String,
                <T as ::sqlx::Database>::Arguments<'args>,
            ),
            #query_error,
        >
        where
            <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
            #( #field_bind_bounds )*
        {
            let mut sql = ::std::string::String::from("DELETE FROM ");
            sql.push_str(#table_name);
            let mut args = <T as ::sqlx::Database>::Arguments::default();

            if let ::core::option::Option::Some(limit) = limit {
                sql.push_str(" WHERE ctid IN (SELECT ctid FROM ");
                sql.push_str(#table_name);

                let has_filters = self.append_conditions_sql(&mut sql, &mut args)?;

                if !has_filters {
                    return Err(#query_error::NoFilters);
                }

                sql.push_str(" LIMIT ");
                sql.push_str(&limit.to_string());
                sql.push_str(")");
            } else {
                let has_filters = self.append_conditions_sql(&mut sql, &mut args)?;

                if !has_filters {
                    return Err(#query_error::NoFilters);
                }
            }

            Ok((sql, args))
        }

        /// Deletes at most one row matching the configured filters and conditions.
        pub async fn delete_one(&self) -> Result<(), #query_error>
        where
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::IntoArguments<'args, T>,
            for<'c> &'c mut T::Connection: ::sqlx::Executor<'c, Database = T>,
            #( #field_bind_bounds )*
        {
            let (sql, args) = self.build_delete_sql(::core::option::Option::Some(1))?;
            let mut conn = self.pool.acquire().await.map_err(#query_error::Database)?;

            ::sqlx::query_with::<T, _>(&sql, args)
                .execute(&mut *conn)
                .await
                .map_err(#query_error::Database)?;

            Ok(())
        }

        /// Deletes all rows matching the configured filters and conditions.
        pub async fn delete_many(&self) -> Result<(), #query_error>
        where
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::IntoArguments<'args, T>,
            for<'c> &'c mut T::Connection: ::sqlx::Executor<'c, Database = T>,
            #( #field_bind_bounds )*
        {
            let (sql, args) = self.build_delete_sql(::core::option::Option::None)?;
            let mut conn = self.pool.acquire().await.map_err(#query_error::Database)?;

            ::sqlx::query_with::<T, _>(&sql, args)
                .execute(&mut *conn)
                .await
                .map_err(#query_error::Database)?;

            Ok(())
        }
    }
}

fn insert_value_binds(cx: &ExpansionContext) -> Vec<TokenStream> {
    let query_error = &cx.query_error;

    cx.fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let name = &field.ident;
            let separator = if index == 0 { "" } else { ", " };

            quote! {
                sql.push_str(#separator);
                ::sqlx::Arguments::add(&mut args, &value.#name)
                    .map_err(|error| #query_error::Database(::sqlx::Error::Encode(error)))?;
                ::sqlx::Arguments::format_placeholder(&args, &mut sql)
                    .expect("writing a SQL placeholder into a String should not fail");
            }
        })
        .collect()
}

fn update_assignments(cx: &ExpansionContext) -> Vec<TokenStream> {
    let query_error = &cx.query_error;

    cx.fields
        .iter()
        .filter(|field| !field.primary_key)
        .enumerate()
        .map(|(index, field)| {
            let name = &field.ident;
            let column_name = &field.column_name;
            let separator = if index == 0 { "" } else { ", " };

            quote! {
                sql.push_str(#separator);
                sql.push_str(#column_name);
                sql.push_str(" = ");
                ::sqlx::Arguments::add(&mut args, &value.#name)
                    .map_err(|error| #query_error::Database(::sqlx::Error::Encode(error)))?;
                ::sqlx::Arguments::format_placeholder(&args, &mut sql)
                    .expect("writing a SQL placeholder into a String should not fail");
            }
        })
        .collect()
}
