use inquiry::Queryable;

#[derive(Queryable)]
#[query(tabel = "players")]
struct UnknownTableAttr {
    id: String,
}

fn main() {}
