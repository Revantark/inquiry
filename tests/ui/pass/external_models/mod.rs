use inquiry_sqlx::Queryable;

#[derive(sqlx::FromRow, Queryable, Debug)]
#[query(table = "users")]
pub struct User {
    #[query(primary_key)]
    pub id: String,
    pub name: String,
}

#[derive(sqlx::FromRow, Queryable, Debug)]
#[query(table = "posts")]
pub struct Post {
    #[query(primary_key)]
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub likes: i64,
}
