use inquiry::Queryable;

#[derive(Queryable)]
struct BadPrimaryKeys {
    #[query(primary_key)]
    id: String,
    #[query(primary_key)]
    external_id: String,
}

fn main() {}
