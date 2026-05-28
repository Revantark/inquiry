use inquiry::Queryable;

#[derive(Queryable)]
struct UnknownFieldAttr {
    #[query(colum = "player_id")]
    id: String,
}

fn main() {}
