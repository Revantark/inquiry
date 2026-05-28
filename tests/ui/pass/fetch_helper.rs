use inquiry::{Queryable, QueryOrderingOperator};

#[derive(sqlx::FromRow, Queryable, Debug)]
struct Player {
    #[query(primary_key)]
    id: String,
    name: String,
    age: i64,
}

fn assert_fetch_helper_api<T: sqlx::Database>(pool: sqlx::Pool<T>)
where
    for<'args> <T as sqlx::Database>::Arguments<'args>:
        Default + sqlx::Arguments<'args, Database = T> + sqlx::IntoArguments<'args, T>,
    for<'q> &'q String: sqlx::Encode<'q, T>,
    String: sqlx::Type<T>,
    for<'q> &'q i64: sqlx::Encode<'q, T>,
    i64: sqlx::Type<T>,
{
    let query = Player::query(pool)
        .by_name(String::from("Alice"))
        .where_age(QueryOrderingOperator::Gte, 18);

    let _ = query.build_select_sql(Some(1));
    let _ = query.build_select_sql(None);
}

fn main() {}
