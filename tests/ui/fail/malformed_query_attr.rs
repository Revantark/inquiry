use inquiry_sqlx::Queryable;

#[derive(Queryable)]
#[query(table)]
struct MalformedAttr {
    #[query(primary_key)]
    id: String,
}

fn main() {}
