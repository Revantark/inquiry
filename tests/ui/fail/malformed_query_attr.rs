use inquiry::Queryable;

#[derive(Queryable)]
#[query(table)]
struct MalformedAttr {
    #[query(primary_key)]
    id: String,
}

fn main() {}
