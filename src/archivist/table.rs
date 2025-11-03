use std::future::Future;

use sqlx::{PgConnection, PgPool};

/// This trait is what makes the `archivist` work. Any struct expected to go
/// into an `sql` table needs to implement this.
pub trait TableItem:
    for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow>
    + std::marker::Send
    + std::marker::Sync
    + std::marker::Unpin
{
    /// Which table we go into.
    const TABLE: &'static str;

    /// Our id.
    fn id(&self) -> i32;

    #[deprecated]
    /// The columns used for insertion.
    fn insert_columns() -> &'static str;

    #[deprecated]
    /// The values used for insertion.
    fn insert_values(&self) -> String;

    #[deprecated]
    /// The values used for checking conflicts.
    fn unique_values(&self) -> String;

    #[deprecated]
    /// The columns used for selection.
    fn select() -> &'static str;

    /// Checks if an id exists.
    fn exists(
        pool: &PgPool,
        id: i32,
    ) -> impl Future<Output = sqlx::Result<bool>> + Send;

    /// Gets all the items.
    fn select_all(
        pool: &PgPool,
    ) -> impl Future<Output = sqlx::Result<Vec<Self>>> + Send;

    /// Selects an element with the specified id.
    fn select_by_id(
        pool: &PgPool,
        id: i32,
    ) -> impl Future<Output = sqlx::Result<Self>> + Send;

    /// Returns the id of any row that matches any column of `self`.
    fn check_unique(
        &self,
        pool: &PgPool,
    ) -> impl Future<Output = sqlx::Result<Option<i32>>> + Send;

    /// Deletes entry of the specified id.
    fn delete(
        tx: &mut PgConnection,
        id: i32,
    ) -> impl Future<Output = sqlx::Result<()>> + Send;

    /// Inserts `self` and returns newly acquired id.
    fn insert(
        &self,
        tx: &mut PgConnection,
    ) -> impl Future<Output = sqlx::Result<i32>> + Send;

    /// Updates row of id `id` with `self`.
    fn update(
        &self,
        tx: &mut PgConnection,
        id: i32,
    ) -> impl Future<Output = sqlx::Result<()>> + Send;
}
