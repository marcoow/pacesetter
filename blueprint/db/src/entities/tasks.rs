#[cfg(feature = "test-helpers")]
use fake::{faker::lorem::en::*, Dummy};
use serde::Deserialize;
use serde::Serialize;
use sqlx::Postgres;
use uuid::Uuid;
use validator::Validate;

/// A task, i.e. TODO item.
#[derive(Serialize, Debug, Deserialize)]
pub struct Task {
    /// The id of the record.
    pub id: Uuid,
    /// The description, i.e. what to do.
    pub description: String,
}

/// A changeset representing the data that is intended to be used to either create a new task or update an existing task.
///
/// Changesets are validatated in the [`create`] and [`update`] functions which return an [Result::Err] if validation fails.
///
/// Changesets can also be used to generate fake data for tests when the `test-helpers` feature is enabled:
///
/// ```
/// let task_changeset: TaskChangeset = Faker.fake();
/// ```
#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct TaskChangeset {
    /// The description must be at least 1 character long.
    #[cfg_attr(feature = "test-helpers", dummy(faker = "Sentence(3..8)"))]
    #[validate(length(min = 1))]
    pub description: String,
}

/// Load all [`Task`]s from the database.
pub async fn load_all(
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Vec<Task>, anyhow::Error> {
    let tasks = sqlx::query_as!(Task, "SELECT id, description FROM tasks")
        .fetch_all(executor)
        .await?;
    Ok(tasks)
}

/// Load one [`Task`] from the database identified by its ID.
///
/// If no record can be found for the ID, a [`crate::Error::NoRecordFound`] will be returned.
pub async fn load(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Task, crate::Error> {
    match sqlx::query_as!(Task, "SELECT id, description FROM tasks WHERE id = $1", id)
        .fetch_optional(executor)
        .await
        .map_err(|e| crate::Error::DbError(e.into()))?
    {
        Some(task) => Ok(task),
        None => Err(crate::Error::NoRecordFound),
    }
}

/// Delete a [`Task`] from the database identified by its ID.
///
/// If no record can be found for the ID, a [`crate::Error::NoRecordFound`] will be returned.
pub async fn delete(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    match sqlx::query!("DELETE FROM tasks WHERE id = $1 RETURNING id", id)
        .fetch_optional(executor)
        .await
        .map_err(|e| crate::Error::DbError(e.into()))?
    {
        Some(_) => Ok(()),
        None => Err(crate::Error::NoRecordFound),
    }
}

/// Create a task in the database with the data in the passed [`TaskChangeset`].
///
/// If the data in the changeset isn't valid, a [`crate::Error::ValidationError`] will be returned, otherwise the created task is returned.
pub async fn create(
    task: TaskChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Task, crate::Error> {
    task.validate().map_err(crate::Error::ValidationError)?;

    let record = sqlx::query!(
        "INSERT INTO tasks (description) VALUES ($1) RETURNING id",
        task.description
    )
    .fetch_one(executor)
    .await
    .map_err(|e| crate::Error::DbError(e.into()))?;

    Ok(Task {
        id: record.id,
        description: task.description,
    })
}

/// Updates a task in the database with the data in the passed [`TaskChangeset`].
///
/// If the data in the changeset isn't valid, a [`crate::Error::ValidationError`] will be returned, otherwise the updated [`Task`] is returned. If no record can be found for the ID, a [`crate::Error::NoRecordFound`] will be returned.
pub async fn update(
    id: Uuid,
    task: TaskChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Task, crate::Error> {
    task.validate().map_err(crate::Error::ValidationError)?;

    match sqlx::query!(
        "UPDATE tasks SET description = $1 WHERE id = $2 RETURNING id, description",
        task.description,
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(|e| crate::Error::DbError(e.into()))?
    {
        Some(record) => Ok(Task {
            id: record.id,
            description: record.description,
        }),
        None => Err(crate::Error::NoRecordFound),
    }
}
