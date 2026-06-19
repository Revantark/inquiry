use inquiry_sqlx::Queryable;

#[derive(Queryable)]
struct UnknownFieldAttr {
    #[query(colum = "player_id")]
    id: String,
}

fn main() {}
