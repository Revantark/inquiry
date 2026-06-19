use inquiry_sqlx::{Queryable, QueryOperator, QueryOrderingOperator};

#[derive(sqlx::FromRow, Queryable, Debug)]
#[query(table = "players_v3")]
struct Player {
    #[query(column = "player_id", sql_type = "TEXT", primary_key)]
    id: String,
    name: String,
    age: i64,
}

fn assert_where_api<T: sqlx::Database>(pool: sqlx::Pool<T>) {
    let _ = Player::query(pool)
        .where_age(QueryOrderingOperator::Gt, 18)
        .where_name(QueryOperator::Like, String::from("Ali%"))
        .any(|q| {
            q.where_age(QueryOrderingOperator::Lt, 10)
                .where_name(QueryOperator::ILike, String::from("bo%"))
                .where_id(QueryOperator::Eq, String::from("player-1"))
        })
        .by_age(21);
}

fn main() {}
