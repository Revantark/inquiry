use inquiry::Queryable;

#[derive(Queryable)]
struct BadType {
    id: Vec<String>,
}

fn main() {}
