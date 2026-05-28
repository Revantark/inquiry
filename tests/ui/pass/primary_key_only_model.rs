use inquiry::Queryable;

#[derive(sqlx::FromRow, Queryable, Debug)]
#[query(table = "locks")]
struct Lock {
    #[query(primary_key)]
    id: String,
}

fn assert_primary_key_only_api<T: sqlx::Database>(pool: sqlx::Pool<T>) {
    let query = Lock::query(pool);
    let _ = query.clone().by_id(String::from("lock-1"));
}

async fn assert_primary_key_only_update_one_api(pool: sqlx::PgPool) {
    let query = Lock::query(pool);
    let _ = query
        .update_one(Lock {
            id: String::from("lock-1"),
        })
        .await;
}

fn main() {}
