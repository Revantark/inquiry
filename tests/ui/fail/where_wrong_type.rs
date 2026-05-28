use inquiry::{Queryable, QueryOrderingOperator};

#[derive(sqlx::FromRow, Queryable, Debug)]
struct Player {
    #[query(primary_key)]
    id: String,
    age: i64,
}

fn assert_where_type_safety<T: sqlx::Database>(pool: sqlx::Pool<T>) {
    let _ = Player::query(pool).where_age(QueryOrderingOperator::Eq, String::from("old"));
}

fn main() {}
