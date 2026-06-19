mod external_models;

use external_models::{Post, User};
use inquiry_sqlx::{QueryOperator, QueryOrderingOperator};

fn assert_external_models_api<T: sqlx::Database>(pool: sqlx::Pool<T>) {
    let _ = User::query(pool.clone()).where_name(QueryOperator::ILike, String::from("ali%"));

    let _ = Post::query(pool)
        .by_user_id(String::from("user-1"))
        .where_title(QueryOperator::Like, String::from("intro%"))
        .where_likes(QueryOrderingOperator::Gte, 10);
}

fn main() {}
