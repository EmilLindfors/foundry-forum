pub use sqlx;
pub use sqlx::sqlite::SqlitePool;
use std::path::Path;
pub mod error;
pub mod user;
pub mod article;

use sqlx::{
    migrate::Migrator,
    pool::PoolConnection,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow},
    Executor, Row, Sqlite, SqliteConnection,
};

pub type DbPool = SqlitePool;

pub async fn pool() -> anyhow::Result<DbPool> {
    let options = SqliteConnectOptions::new()
    .filename(&dotenvy::var("DATABASE_URL")?)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;
    Ok(pool)
}

pub async fn migrate(mut conn: PoolConnection<Sqlite>) -> Result<(), sqlx::Error> {
    let path = Path::new("db/migrations");
    let migrator = Migrator::new(path).await?;
    migrator.run(&mut conn).await?;
    Ok(())
}
