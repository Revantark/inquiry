use inquiry::Queryable;

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
