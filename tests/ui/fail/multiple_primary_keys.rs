use inquiry_sqlx::Queryable;

#[derive(Queryable)]
struct BadPrimaryKeys {
    #[query(primary_key)]
    id: String,
    #[query(primary_key)]
    external_id: String,
}

fn main() {}
