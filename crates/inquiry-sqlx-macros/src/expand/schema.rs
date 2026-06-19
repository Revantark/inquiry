use proc_macro2::TokenStream;
use quote::quote;

use super::context::ExpansionContext;

pub(super) fn expand_create_table_method(cx: &ExpansionContext) -> TokenStream {
    let table_name = &cx.table_name;
    let create_table_columns = &cx.create_table_columns;
    let query_error = &cx.query_error;

    quote! {
        fn create_table_sql(&self) -> ::std::string::String {
            let mut sql = ::std::string::String::from("CREATE TABLE IF NOT EXISTS ");
            sql.push_str(#table_name);
            sql.push_str(" (");
            sql.push_str(#create_table_columns);
            sql.push_str(")");
            sql
        }

        /// Creates the backing table when it does not already exist.
        pub async fn create_table_if_not_exists(&self) -> Result<(), #query_error>
        where
            for<'c> &'c mut T::Connection: ::sqlx::Executor<'c, Database = T>,
            for<'args> <T as ::sqlx::Database>::Arguments<'args>: ::sqlx::IntoArguments<'args, T>,
        {
            let sql = self.create_table_sql();

            let mut conn = self.pool.acquire().await.map_err(#query_error::Database)?;
            ::sqlx::query::<T>(&sql)
                .execute(&mut *conn)
                .await
                .map_err(#query_error::Database)?;

            Ok(())
        }
    }
}
