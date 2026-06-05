# inquiry

`inquiry` is a small Rust crate for people who want simple SQLx-backed query
builders without writing the same CRUD glue for every struct.

You define a model, derive `Queryable`, and `inquiry` gives you a typed query
builder for that model. It can create the table, insert rows, upsert by primary
key, update by primary key, fetch rows, and delete rows with field-specific
filters.

The crate currently targets SQLx and PostgreSQL.

## What It Generates

For a model like `Player`, `inquiry` generates:

- `Player::query(pool)`
- a query builder named `PlayerQuery<T>`
- `PlayerQueryError`
- table creation helpers
- insert, upsert, update, fetch, count, exists, and delete methods
- `by_<field>` equality filters
- `where_<field>` operator filters

The operator types are shared by the library:

- `QueryOperator` for text fields
- `QueryOrderingOperator` for numeric fields
- `QueryEqualityOperator` for fields that only support equality checks

That keeps generated models usable across normal Rust modules without creating
duplicate operator enums for each model.

## A Small Example

```rust
use inquiry::{Queryable, QueryOperator, QueryOrderingOperator};
use sqlx::PgPool;

#[derive(sqlx::FromRow, Queryable, Debug)]
#[query(table = "players")]
struct Player {
    #[query(column = "player_id", sql_type = "TEXT", primary_key)]
    id: String,
    name: String,
    age: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    let players = Player::query(pool);

    players.create_table_if_not_exists().await?;

    players
        .upsert_one(Player {
            id: "one".to_string(),
            name: "Alice".to_string(),
            age: 21,
        })
        .await?;

    let rows = players
        .clone()
        .where_name(QueryOperator::ILike, "ali%".to_string())
        .where_age(QueryOrderingOperator::Gte, 18)
        .fetch_many()
        .await?;

    for row in rows {
        println!("{row:?}");
    }

    Ok(())
}
```

Derive `sqlx::FromRow` when you want to fetch rows back into the model.

## Creating a Query Builder

```rust
let query = Player::query(pool);
```

This stores the SQLx pool and starts with no filters.

## Creating Tables

```rust
query.create_table_if_not_exists().await?;
```

This creates the backing table from the model fields. A field marked with
`#[query(primary_key)]` becomes the table primary key.

## Inserting Rows

```rust
query.add_one(player).await?;
query.add_many(players).await?;
```

`add_one` inserts one row. `add_many` inserts every row in the vector. Passing an
empty vector is a no-op.

## Upserting Rows

```rust
query.upsert_one(player).await?;
```

`upsert_one` inserts the row, or updates the existing row with the same primary
key. The model needs exactly one `#[query(primary_key)]` field for this method.

## Updating Rows

```rust
query.update_one(player).await?;
query.update_many(players).await?;
```

Updates are matched by primary key. Each non-primary-key field is written back to
the row.

As with upserts, the model needs exactly one `#[query(primary_key)]` field.

## Filtering

Every field gets a `by_<field>` method for equality:

```rust
let query = Player::query(pool)
    .by_id("one".to_string())
    .by_age(21);
```

Every field also gets a `where_<field>` method for explicit operators:

```rust
let query = Player::query(pool)
    .where_name(QueryOperator::Like, "Ali%".to_string())
    .where_age(QueryOrderingOperator::Gte, 18);
```

Filters are joined with `AND`.

Use `any` when you need an `OR` group:

```rust
let query = Player::query(pool)
    .where_name(QueryOperator::ILike, "ali%".to_string())
    .any(|q| {
        q.where_age(QueryOrderingOperator::Lt, 10)
            .where_name(QueryOperator::ILike, "bo%".to_string())
    });
```

Filters outside `any` are still joined with `AND`. Conditions inside one `any`
group are joined with `OR`.

## Fetching Rows

```rust
let one = query.fetch_one().await?;
let many = query.fetch_many().await?;
```

`fetch_one` returns at most one matching row. `fetch_many` returns all matching
rows.

Both methods require at least one filter. If you call them without filters, they
return `PlayerQueryError::NoFilters`. This is intentional: the generated API is
biased away from accidental full-table reads.

## Counting and Checking Rows

```rust
let matching = query.count().await?;
let any_matching = query.exists().await?;
```

`count` returns the number of matching rows as an `i64`. `exists` returns
whether at least one matching row exists.

Both methods require at least one filter. If you call them without filters, they
return `PlayerQueryError::NoFilters`, matching the fetch safety behavior.

## Deleting Rows

```rust
query.by_id("one".to_string()).delete_one().await?;
query.where_name(QueryOperator::ILike, "bot-%".to_string()).delete_many().await?;
```

`delete_one` deletes at most one matching row. `delete_many` deletes all matching
rows.

Both methods require at least one filter. If you call them without filters, they
return `PlayerQueryError::NoFilters`, matching the fetch safety behavior.

## Operators

`QueryOperator` is for strings:

```rust
QueryOperator::Eq
QueryOperator::Ne
QueryOperator::Like
QueryOperator::ILike
```

These map to `=`, `!=`, `LIKE`, and `ILIKE`.

`QueryOrderingOperator` is for numeric fields:

```rust
QueryOrderingOperator::Eq
QueryOrderingOperator::Ne
QueryOrderingOperator::Gt
QueryOrderingOperator::Gte
QueryOrderingOperator::Lt
QueryOrderingOperator::Lte
```

These map to `=`, `!=`, `>`, `>=`, `<`, and `<=`.

`QueryEqualityOperator` is for fields that only support equality checks:

```rust
QueryEqualityOperator::Eq
QueryEqualityOperator::Ne
```

These map to `=` and `!=`.

## Multiple Models

Models can live in normal Rust modules.

```rust
// src/main.rs
mod models;

use inquiry::QueryOperator;
use models::{Post, User};

let users = User::query(pool.clone());
let posts = Post::query(pool.clone());

users.create_table_if_not_exists().await?;
posts.create_table_if_not_exists().await?;

let alice_posts = posts
    .where_user_id(QueryOperator::Eq, "user-1".to_string())
    .fetch_many()
    .await?;
```

```rust
// src/models/mod.rs
use inquiry::Queryable;

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
}
```

`Post` references `User` by storing `user_id`. For larger relationships, use the
same approach: keep the foreign key field on the model that points at another
model.

## Attributes

Use `#[query(table = "...")]` to set the table name:

```rust
#[query(table = "players")]
struct Player {
    // ...
}
```

If you omit it, the table name defaults to the lowercase struct name.

Use `#[query(column = "...")]` to set a column name:

```rust
#[query(column = "player_id")]
id: String,
```

If you omit it, the column name defaults to the field name.

Use `#[query(sql_type = "...")]` to set a SQL column type:

```rust
#[query(sql_type = "TEXT")]
id: String,
```

This is useful when you want to override the inferred type, or when the field
type is not one the macro knows how to infer.

Use `#[query(primary_key)]` to mark the primary key:

```rust
#[query(primary_key)]
id: String,
```

You can combine field attributes:

```rust
#[query(column = "player_id", sql_type = "TEXT", primary_key)]
id: String,
```

## Supported Field Types

The macro can infer PostgreSQL column types for these Rust types:

| Rust type | PostgreSQL type |
| --- | --- |
| `String` | `TEXT` |
| `i16` | `SMALLINT` |
| `i32` | `INTEGER` |
| `i64` | `BIGINT` |
| `bool` | `BOOLEAN` |
| `f32` | `REAL` |
| `f64` | `DOUBLE PRECISION` |

For other types, add `#[query(sql_type = "...")]` to the field.

## Notes and Limitations

- This crate currently targets SQLx and PostgreSQL.
- The derive macro only supports named struct fields.
- Only one primary key field is supported.
- `fetch_one`, `fetch_many`, `count`, `exists`, `delete_one`, and `delete_many`
  require at least one configured filter.
- `LIKE` and `ILIKE` are available through `QueryOperator`; pass the SQL pattern
  yourself, such as `"Ali%"` or `"%ice"`.
- PostgreSQL does not have native unsigned integer columns. Prefer signed Rust
  integer types such as `i64` for integer fields.
