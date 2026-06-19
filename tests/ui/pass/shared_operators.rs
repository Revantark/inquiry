use inquiry_sqlx::{Queryable, QueryOperator, QueryOrderingOperator};

#[derive(sqlx::FromRow, Queryable, Debug)]
#[query(table = "players")]
struct Player {
    #[query(primary_key)]
    id: String,
    name: String,
    age: i64,
}

fn assert_shared_operator_api<T: sqlx::Database>(pool: sqlx::Pool<T>) {
    let _ = Player::query(pool)
        .where_name(QueryOperator::ILike, String::from("ali%"))
        .where_age(QueryOrderingOperator::Gte, 18);
}

fn main() {}
