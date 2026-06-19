use proc_macro2::TokenStream;
use quote::quote;

use super::context::ExpansionContext;

pub(super) fn expand_append_conditions_method(cx: &ExpansionContext) -> TokenStream {
    let query_error = &cx.query_error;
    let field_bind_bounds = &cx.field_bind_bounds;

    quote! {
        fn append_conditions_sql<'args>(
            &'args self,
            sql: &mut ::std::string::String,
            args: &mut <T as ::sqlx::Database>::Arguments<'args>,
        ) -> Result<bool, #query_error>
        where
            <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::Arguments<'args, Database = T>,
            #( #field_bind_bounds )*
        {
            let mut has_filters = false;

            for condition in &self.conditions {
                if condition.is_empty() {
                    continue;
                }

                if has_filters {
                    sql.push_str(" AND ");
                } else {
                    sql.push_str(" WHERE ");
                    has_filters = true;
                }

                condition.append_sql::<T>(sql, args)?;
            }

            Ok(has_filters)
        }
    }
}

pub(super) fn expand_fetch_methods(cx: &ExpansionContext) -> TokenStream {
    let struct_name = &cx.struct_name;
    let query_error = &cx.query_error;
    let field_bind_bounds = &cx.field_bind_bounds;

    quote! {
        /// Fetches at most one row matching the configured filters and conditions.
        pub async fn fetch_one(&self) -> Result<::core::option::Option<#struct_name>, #query_error>
        where
            for<'r> #struct_name: ::sqlx::FromRow<'r, T::Row> + ::core::marker::Send + ::core::marker::Unpin,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::IntoArguments<'args, T>,
            for<'c> &'c mut T::Connection: ::sqlx::Executor<'c, Database = T>,
            #( #field_bind_bounds )*
        {
            let (sql, args) = self.build_select_sql(::core::option::Option::Some(1))?;
            let mut conn = self.pool.acquire().await.map_err(#query_error::Database)?;
            let query = ::sqlx::query_as_with::<T, #struct_name, _>(&sql, args);

            query
                .fetch_optional(&mut *conn)
                .await
                .map_err(#query_error::Database)
        }

        /// Fetches all rows matching the configured filters and conditions.
        pub async fn fetch_many(&self) -> Result<::std::vec::Vec<#struct_name>, #query_error>
        where
            for<'r> #struct_name: ::sqlx::FromRow<'r, T::Row> + ::core::marker::Send + ::core::marker::Unpin,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::IntoArguments<'args, T>,
            for<'c> &'c mut T::Connection: ::sqlx::Executor<'c, Database = T>,
            #( #field_bind_bounds )*
        {
            let (sql, args) = self.build_select_sql(::core::option::Option::None)?;
            let mut conn = self.pool.acquire().await.map_err(#query_error::Database)?;
            let query = ::sqlx::query_as_with::<T, #struct_name, _>(&sql, args);

            query
                .fetch_all(&mut *conn)
                .await
                .map_err(#query_error::Database)
        }
    }
}

pub(super) fn expand_count_exists_methods(cx: &ExpansionContext) -> TokenStream {
    let table_name = &cx.table_name;
    let query_error = &cx.query_error;
    let field_bind_bounds = &cx.field_bind_bounds;

    quote! {
        fn build_count_sql<'args>(
            &'args self,
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
            let mut sql = ::std::string::String::from("SELECT COUNT(*) FROM ");
            sql.push_str(#table_name);
            let mut args = <T as ::sqlx::Database>::Arguments::default();
            let has_filters = self.append_conditions_sql(&mut sql, &mut args)?;

            if !has_filters {
                return Err(#query_error::NoFilters);
            }

            Ok((sql, args))
        }

        fn build_exists_sql<'args>(
            &'args self,
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
            let mut sql = ::std::string::String::from("SELECT EXISTS(SELECT 1 FROM ");
            sql.push_str(#table_name);
            let mut args = <T as ::sqlx::Database>::Arguments::default();
            let has_filters = self.append_conditions_sql(&mut sql, &mut args)?;

            if !has_filters {
                return Err(#query_error::NoFilters);
            }

            sql.push_str(" LIMIT 1)");

            Ok((sql, args))
        }

        /// Counts rows matching the configured filters and conditions.
        pub async fn count(&self) -> Result<i64, #query_error>
        where
            for<'r> (i64,): ::sqlx::FromRow<'r, T::Row>,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::IntoArguments<'args, T>,
            for<'c> &'c mut T::Connection: ::sqlx::Executor<'c, Database = T>,
            #( #field_bind_bounds )*
        {
            let (sql, args) = self.build_count_sql()?;
            let mut conn = self.pool.acquire().await.map_err(#query_error::Database)?;

            ::sqlx::query_scalar_with::<T, i64, _>(&sql, args)
                .fetch_one(&mut *conn)
                .await
                .map_err(#query_error::Database)
        }

        /// Returns whether any row matches the configured filters and conditions.
        pub async fn exists(&self) -> Result<bool, #query_error>
        where
            for<'r> (bool,): ::sqlx::FromRow<'r, T::Row>,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::core::default::Default + ::sqlx::Arguments<'args, Database = T>,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::IntoArguments<'args, T>,
            for<'c> &'c mut T::Connection: ::sqlx::Executor<'c, Database = T>,
            #( #field_bind_bounds )*
        {
            let (sql, args) = self.build_exists_sql()?;
            let mut conn = self.pool.acquire().await.map_err(#query_error::Database)?;

            ::sqlx::query_scalar_with::<T, bool, _>(&sql, args)
                .fetch_one(&mut *conn)
                .await
                .map_err(#query_error::Database)
        }
    }
}

pub(super) fn expand_build_select_sql_method(cx: &ExpansionContext) -> TokenStream {
    let table_name = &cx.table_name;
    let select_fields = &cx.select_fields;
    let query_error = &cx.query_error;
    let field_bind_bounds = &cx.field_bind_bounds;

    quote! {
        fn build_select_sql<'args>(
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
            let mut sql = ::std::string::String::from("SELECT ");
            sql.push_str(#select_fields);
            sql.push_str(" FROM ");
            sql.push_str(#table_name);
            let mut args = <T as ::sqlx::Database>::Arguments::default();
            let has_filters = self.append_conditions_sql(&mut sql, &mut args)?;

            if !has_filters {
                return Err(#query_error::NoFilters);
            }

            if let ::core::option::Option::Some(limit) = limit {
                sql.push_str(" LIMIT ");
                sql.push_str(&limit.to_string());
            }

            Ok((sql, args))
        }
    }
}
