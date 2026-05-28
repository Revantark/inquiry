use inquiry::Queryable;

#[derive(sqlx::FromRow, Queryable, Debug)]
#[query(table = "inquiry_example_users_v2")]
pub struct User {
    #[query(primary_key)]
    pub id: String,
    pub name: String,
}

#[derive(sqlx::FromRow, Queryable, Debug, Clone)]
#[query(table = "inquiry_example_posts_v2")]
pub struct Post {
    #[query(primary_key)]
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub like_count: i64,
    pub enabled: bool,
}
