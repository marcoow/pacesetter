use crate::entities::users::User;
use fake::{faker::name::en::*, Dummy};
use sqlx::postgres::PgPool;

/// A changeset representing the data that is intended to be used to either create a new user or update an existing user.
///
/// Changesets are validated in the [`create`] function which return an [Result::Err] if validation fails.
///
/// Changesets can also be used to generate fake data for tests when the `test-helpers` feature is enabled:
///
/// ```
/// let user_changeset: UserChangeset = Faker.fake();
/// ```
#[derive(Debug, Clone, Dummy)]
pub struct UserChangeset {
    /// The user's name
    #[dummy(faker = "Name()")]
    pub name: String,
    /// The user's auth token, fake data will be a 100 characters long number
    #[dummy(faker = "100..101")]
    pub token: String,
}

/// Creates a user in the database with the data in the passed [`UserChangeset`].
///
/// If the data in the changeset isn't valid, a [`crate::Error::ValidationError`] will be returned, otherwise the created user is returned.
pub async fn create(user: UserChangeset, db: &PgPool) -> Result<User, anyhow::Error> {
    let record = sqlx::query!(
        "INSERT INTO users (name, token) VALUES ($1, $2) RETURNING id",
        user.name,
        user.token,
    )
    .fetch_one(db)
    .await?;

    Ok(User {
        id: record.id,
        name: user.name,
    })
}
