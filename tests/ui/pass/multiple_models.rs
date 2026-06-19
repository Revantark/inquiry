mod users {
    use inquiry_sqlx::Queryable;

    #[derive(sqlx::FromRow, Queryable, Debug)]
    #[query(table = "users")]
    pub struct User {
        #[query(primary_key)]
        pub id: String,
        pub name: String,
    }
}

mod posts {
    use inquiry_sqlx::Queryable;

    #[derive(sqlx::FromRow, Queryable, Debug)]
    #[query(table = "posts")]
    pub struct Post {
        #[query(primary_key)]
        pub id: String,
        pub user_id: String,
        pub title: String,
    }
}

use inquiry_sqlx::QueryOperator;

fn assert_multiple_model_api<T: sqlx::Database>(pool: sqlx::Pool<T>) {
    let _ = users::User::query(pool.clone())
        .where_name(QueryOperator::ILike, String::from("ali%"));

    let _ = posts::Post::query(pool)
        .by_user_id(String::from("user-1"))
        .where_title(QueryOperator::Like, String::from("hello%"));
}

fn main() {}
