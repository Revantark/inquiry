use inquiry_sqlx::{Queryable, QueryEqualityOperator};

#[derive(sqlx::FromRow, Queryable, Debug)]
struct FeatureFlag {
    #[query(primary_key)]
    name: String,
    enabled: bool,
}

fn assert_bool_operator_api<T: sqlx::Database>(pool: sqlx::Pool<T>) {
    let _ = FeatureFlag::query(pool).where_enabled(QueryEqualityOperator::Eq, true);
}

fn main() {}
