use inquiry_sqlx::Queryable;

#[derive(Queryable)]
#[query(table = "players; drop table players")]
struct InvalidTableIdentifier {
    id: String,
}

fn main() {}
