use inquiry_sqlx::Queryable;

#[derive(sqlx::FromRow, Queryable, Debug)]
struct Player {
    #[query(primary_key)]
    id: String,
    name: String,
}

fn assert_query_field_storage_is_removed<T: sqlx::Database>(pool: sqlx::Pool<T>) {
    let query = Player::query(pool).by_name(String::from("Alice"));
    let _ = query.name;
}

fn main() {}
