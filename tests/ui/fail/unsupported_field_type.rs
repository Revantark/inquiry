use inquiry_sqlx::Queryable;

#[derive(Queryable)]
struct BadType {
    id: Vec<String>,
}

fn main() {}
