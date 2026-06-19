use inquiry_sqlx::Queryable;

#[derive(Queryable)]
struct InvalidColumnIdentifier {
    #[query(column = "player-id")]
    id: String,
}

fn main() {}
