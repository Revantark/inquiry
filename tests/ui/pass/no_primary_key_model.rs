use inquiry_sqlx::Queryable;

#[derive(sqlx::FromRow, Queryable, Debug)]
#[query(table = "audit_events")]
struct AuditEvent {
    event_type: String,
    actor_id: String,
}

fn assert_no_primary_key_api<T: sqlx::Database>(pool: sqlx::Pool<T>) {
    let query = AuditEvent::query(pool);

    let _ = query
        .clone()
        .where_event_type(inquiry_sqlx::QueryOperator::Eq, String::from("login"))
        .by_actor_id(String::from("user-1"));
}

fn main() {}
