use inquiry::Queryable;

#[derive(Queryable)]
struct InvalidColumnIdentifier {
    #[query(column = "player-id")]
    id: String,
}

fn main() {}
