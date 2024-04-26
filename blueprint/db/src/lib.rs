use anyhow::{Context, Result};
use {{crate_name}}_config::DatabaseConfig;
use sqlx::{postgres::PgPoolOptions, Postgres, Transaction};
use thiserror::Error;

pub use sqlx::postgres::PgPool as DbPool;

pub mod entities;

#[doc(hidden)]
pub async fn connect_pool(config: DatabaseConfig) -> Result<DbPool, anyhow::Error> {
    let pool = PgPoolOptions::new()
        .connect(config.url.as_str())
        .await
        .context("Failed to connect to database")?;

    Ok(pool)
}

pub async fn transaction(
    db_pool: &DbPool,
) -> Result<Transaction<'static, Postgres>, anyhow::Error> {
    let tx = db_pool
        .begin()
        .await
        .context("Failed to begin transaction")?;

    Ok(tx)
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("database query failed")]
    DbError(anyhow::Error),
    #[error("no record found where one was expected")]
    NoRecordFound,
    #[error("validation failed")]
    ValidationError(validator::ValidationErrors),
}

#[cfg(feature = "test-helpers")]
pub mod test_helpers;
