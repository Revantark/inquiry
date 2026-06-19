use inquiry_sqlx::{Queryable, QueryOrderingOperator};

#[derive(sqlx::FromRow, Queryable, Debug)]
struct Player {
    #[query(primary_key)]
    id: String,
    name: String,
}

fn assert_text_operator_safety<T: sqlx::Database>(pool: sqlx::Pool<T>) {
    let _ = Player::query(pool).where_name(QueryOrderingOperator::Gte, String::from("Tom"));
}

fn main() {}
