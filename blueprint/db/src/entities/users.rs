use serde::Serialize;
use sqlx::Postgres;
use uuid::Uuid;

/// A user record.
#[derive(Serialize, Debug, Clone)]
pub struct User {
    /// The id of the record.
    pub id: Uuid,
    /// The user's name.
    pub name: String,
}

/// Loads a user based on the passed token.
///
/// If no user exists for the token, [`Option::None`] is returned, otherwise [`Option::Some(User)`] is returned.
pub async fn load_with_token(
    token: &str,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Option<User>, anyhow::Error> {
    match sqlx::query_as!(User, "SELECT id, name FROM users WHERE token = $1", token)
        .fetch_one(executor)
        .await
    {
        Ok(user) => Ok(Some(user)),
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(None),
            _ => Err(error.into()),
        },
    }
}
