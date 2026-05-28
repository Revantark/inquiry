use inquiry::{QueryOperator, Queryable};

#[derive(sqlx::FromRow, Queryable, Debug)]
struct FeatureFlag {
    #[query(primary_key)]
    name: String,
    enabled: bool,
}

fn assert_bool_rejects_like_operator<T: sqlx::Database>(pool: sqlx::Pool<T>) {
    let _ = FeatureFlag::query(pool).where_enabled(QueryOperator::Like, true);
}

fn main() {}
