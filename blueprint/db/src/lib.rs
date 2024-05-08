use anyhow::{Context, Result};
use {{crate_name}}_config::DatabaseConfig;
use sqlx::{postgres::PgPoolOptions, Postgres, Transaction};
use thiserror::Error;

pub use sqlx::postgres::PgPool as DbPool;

/// Entity definitions and related functions
pub mod entities;

/// Starts a new database transaction.
///
/// Example:
/// ```
/// let tx = transaction(&app_state.db_pool).await?;
/// tasks::create(task_data, &mut *tx)?;
/// users::create(user_data, &mut *tx)?;
/// 
/// match tx.commit().await {
///     Ok(_) => Ok((StatusCode::CREATED, Json(results))),
///     Err(e) => Err((internal_error(e), "".into())),
/// }
/// ```
///
/// Transactions are rolled back automatically when they are dropped without having been committed.
pub async fn transaction(
    db_pool: &DbPool,
) -> Result<Transaction<'static, Postgres>, anyhow::Error> {
    let tx = db_pool
        .begin()
        .await
        .context("Failed to begin transaction")?;

    Ok(tx)
}

/// Errors that can occur as a result of a data layer operation.
#[derive(Error, Debug)]
pub enum Error {
    /// General database error, e.g. communicating with the database failed
    #[error("database query failed")]
    DbError(anyhow::Error),
    /// No record was found where one was expected, e.g. when loading a record by ID
    #[error("no record found where one was expected")]
    NoRecordFound,
    #[error("validation failed")]
    /// An invalid changeset was passed to a writing operation such as creating or updating a record.
    ValidationError(validator::ValidationErrors),
}

#[doc(hidden)]
pub async fn connect_pool(config: DatabaseConfig) -> Result<DbPool, anyhow::Error> {
    let pool = PgPoolOptions::new()
        .connect(config.url.as_str())
        .await
        .context("Failed to connect to database")?;

    Ok(pool)
}

#[cfg(any(feature = "test-helpers", doc))]
#[doc(cfg(feature = "test-helpers"))]
pub mod test_helpers;
