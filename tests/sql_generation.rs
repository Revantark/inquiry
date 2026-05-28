use inquiry::{QueryEqualityOperator, QueryOperator, Queryable};

#[derive(sqlx::FromRow, Queryable, Debug)]
#[query(table = "players")]
struct Player {
    #[query(column = "player_id", sql_type = "TEXT", primary_key)]
    id: String,
    name: String,
    active: bool,
}

#[derive(sqlx::FromRow, Queryable, Debug)]
#[query(table = "locks")]
struct Lock {
    #[query(primary_key)]
    id: String,
}

fn pg_pool() -> sqlx::PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://postgres:postgres@localhost/postgres")
        .expect("test database URL should parse")
}

#[tokio::test]
async fn select_sql_uses_aliases_filters_and_placeholders() {
    let query = Player::query(pg_pool())
        .where_name(QueryOperator::ILike, String::from("ali%"))
        .where_active(QueryEqualityOperator::Eq, true);

    let (sql, _) = query.build_select_sql(Some(1)).unwrap();

    assert_eq!(
        sql,
        "SELECT player_id AS id, name, active FROM players WHERE name ILIKE $1 AND active = $2 LIMIT 1"
    );
}

#[tokio::test]
async fn create_table_sql_marks_primary_key() {
    let query = Player::query(pg_pool());

    assert_eq!(
        query.create_table_sql(),
        "CREATE TABLE IF NOT EXISTS players (player_id TEXT PRIMARY KEY, name TEXT, active BOOLEAN)"
    );
}

#[tokio::test]
async fn primary_key_only_upsert_uses_do_nothing() {
    let query = Lock::query(pg_pool());
    let (sql, _) = query
        .build_upsert_one_sql(&Lock {
            id: "lock-1".to_string(),
        })
        .unwrap();

    assert_eq!(
        sql,
        "INSERT INTO locks (id) VALUES ($1) ON CONFLICT (id) DO NOTHING"
    );
}

#[tokio::test]
async fn primary_key_only_update_is_noop() {
    let query = Lock::query(pg_pool());
    let sql = query
        .build_update_one_sql(&Lock {
            id: "lock-1".to_string(),
        })
        .unwrap();

    assert!(sql.is_none());
}
