mod models;

use inquiry::{QueryOperator, QueryOrderingOperator};
use models::{Post, User};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    let users = User::query(pool.clone());
    let posts = Post::query(pool.clone());

    users.create_table_if_not_exists().await?;
    posts.create_table_if_not_exists().await?;

    users
        .upsert_one(User {
            id: "user-1".to_string(),
            name: "Alice".to_string(),
        })
        .await?;

    users
        .upsert_one(User {
            id: "user-2".to_string(),
            name: "Bob".to_string(),
        })
        .await?;

    posts
        .upsert_one(Post {
            id: "post-1".to_string(),
            user_id: "user-1".to_string(),
            title: "Rust query builders".to_string(),
            like_count: 12,
            enabled: true,
        })
        .await?;

    posts
        .upsert_one(Post {
            id: "post-2".to_string(),
            user_id: "user-2".to_string(),
            title: "PostgreSQL notes".to_string(),
            like_count: 3,
            enabled: true,
        })
        .await?;

    let alice_posts = posts
        .clone()
        .by_user_id("user-1".to_string())
        .where_title(QueryOperator::ILike, "%rust%".to_string())
        .where_like_count(QueryOrderingOperator::Gte, 1)
        .fetch_many()
        .await?;

    for post in &alice_posts {
        println!(
            "{} by {}: {} (likes: {}, enabled: {})",
            post.id, post.user_id, post.title, post.like_count, post.enabled
        );
    }

    if let Some(post) = alice_posts.first() {
        posts.update_one(post.clone().set_enabled(false)).await?;
    }

    let bobs = users.by_name("Bob".to_string()).fetch_many().await?;

    for user in &bobs {
        println!("{}: {}", user.id, user.name);
    }

    Ok(())
}
