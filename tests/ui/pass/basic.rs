use inquiry_sqlx::Queryable;

#[derive(sqlx::FromRow, Queryable, Debug)]
#[query(table = "players_v3")]
struct Player {
    #[query(column = "player_id", sql_type = "TEXT", primary_key)]
    id: String,
    name: String,
    age: i64,
}

fn assert_public_api<T: sqlx::Database>(pool: sqlx::Pool<T>) {
    let query = Player::query(pool);
    let _ = query
        .clone()
        .by_id(String::new())
        .by_name(String::new())
        .by_age(1);
}

async fn assert_update_one_api(pool: sqlx::PgPool) {
    let query = Player::query(pool);
    let _ = query
        .update_one(Player {
            id: String::from("player-1"),
            name: String::from("Alice"),
            age: 42,
        })
        .await;
}

async fn assert_delete_api(pool: sqlx::PgPool) {
    let query = Player::query(pool);

    let _ = query.clone().by_id(String::from("player-1")).delete_one().await;
    let _ = query.by_name(String::from("Alice")).delete_many().await;
}

async fn assert_count_exists_api(pool: sqlx::PgPool) {
    let query = Player::query(pool);

    let _: Result<i64, _> = query.clone().by_age(42).count().await;
    let _: Result<bool, _> = query.by_name(String::from("Alice")).exists().await;
}

fn assert_model_setter_api() {
    let player = Player {
        id: String::from("player-1"),
        name: String::from("Alice"),
        age: 41,
    }
    .set_name(String::from("Bob"))
    .set_age(42);

    let _ = player;
}

fn main() {}
