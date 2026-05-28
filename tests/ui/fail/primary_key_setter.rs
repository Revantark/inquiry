use inquiry::Queryable;

#[derive(sqlx::FromRow, Queryable, Debug)]
#[query(table = "players")]
struct Player {
    #[query(primary_key)]
    id: String,
    name: String,
}

fn main() {
    let _ = Player {
        id: String::from("player-1"),
        name: String::from("Alice"),
    }
    .set_id(String::from("player-2"));
}
