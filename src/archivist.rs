//! A module to handle the postgres database connection.
//! All DB interactions should pass through Archivist.
//!
//! Every function modifying the DB (i.e. not ones that only _get_ data) will
//! automatically start a transaction if it there is not already one active.
//! No function should commit a transaction, except for `commit_transcation`.

use crate::{ARPAError, config::Config};
use log::{info, warn};
use std::{fmt::Debug, fs::read_to_string};

pub mod data_types;
mod error;
pub mod table;

pub use error::ArchivistError;
use sqlx::{
    FromRow, PgConnection, Pool, Postgres, Transaction,
    postgres::{PgPoolOptions, PgRow},
};
use table::{Table, TableItem};

type Result<T> = std::result::Result<T, ArchivistError>;

/// This keeps a live connection to the database and acts as your friend in
/// getting and posting data.
///
/// For any queries that modify the DB, a transaction
/// _will_ be used, and if the user has not explicitly started one, a warning
/// will be issued (though not an error). NB: If a transaction goes out of scope,
/// it is rolled back.
///
/// All tables are accessible _only_ through the `Table` enum.
pub struct Archivist {
    pool: Pool<Postgres>,
    config: Config,

    /// This is here so that potentially destructive app commands always go
    /// through transactions.
    current_transaction: Option<Transaction<'static, Postgres>>,
}

impl Archivist {
    /// Initializes a new connection to the database.
    ///
    /// # Errors
    /// Fails if setup data is missing. Forwards errors from `sqlx`.
    pub async fn new(
        config_path: impl AsRef<std::path::Path>,
        sql_setup_dir: impl AsRef<std::path::Path>,
    ) -> std::result::Result<Self, ARPAError> {
        info!("Reading config \"{}\"...", config_path.as_ref().display());
        let config = Config::load(config_path)?;

        let pool = PgPoolOptions::new()
            .max_connections(config.database.pool_connections)
            .acquire_timeout(std::time::Duration::from_millis(
                config.database.connection_timeout,
            ))
            .connect(&config.database.url)
            .await
            .map_err(ArchivistError::from)?;

        info!("Connected to database!");

        // Setup from sql directory
        info!(
            "Reading setup dir \"{}\"...",
            sql_setup_dir.as_ref().display()
        );
        let files = std::fs::read_dir(sql_setup_dir)?
            .flat_map(|entry| entry.map(|e| read_to_string(e.path())))
            .flatten()
            .collect::<Vec<_>>();

        for file in files {
            for sql in file.split(';') {
                sqlx::query(sql)
                    .execute(&pool)
                    .await
                    .map_err(ArchivistError::from)?;
            }
        }
        info!("Finished setup!");

        Ok(Self {
            pool,
            config,
            current_transaction: None,
        })
    }

    /// Starts a new transaction. Returns an error if there is a previous
    /// transaction still live.
    /// # Errors
    /// Fails if there is already a live transaction
    pub async fn start_transaction(&mut self) -> Result<()> {
        if self.current_transaction.is_some() {
            return Err(ArchivistError::TransactionAlreadyLive);
        }

        self.current_transaction = Some(self.pool.begin().await?);
        Ok(())
    }

    /// Commits a currently live transaction. Returns an error if there is none
    /// present.
    /// # Errors
    /// Fails if there is no live transaction. Forwards errors from `sqlx`.
    pub async fn commit_transaction(&mut self) -> Result<()> {
        self.current_transaction
            .take()
            .ok_or(ArchivistError::NoTransactionToCommit)?
            .commit()
            .await?;

        Ok(())
    }

    /// Undos a currently live transaction. Returns an error if there is none
    /// present.
    /// # Errors
    /// Fails if there is no live transaction. Forwards errors from `sqlx`.
    pub async fn rollback_transaction(&mut self) -> Result<()> {
        self.current_transaction
            .take()
            .ok_or(ArchivistError::NoTransactionToRollback)?
            .rollback()
            .await?;

        Ok(())
    }

    /// Checks whether a row with `id` exists in `table`.
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn id_exists(&self, table: Table, id: i32) -> Result<bool> {
        let table_name = table.to_string();

        let query = format!(
            "select exists (select 1 from {table_name} where id={id});",
        );
        let exists: (bool,) =
            sqlx::query_as(&query).fetch_one(&self.pool).await?;

        Ok(exists.0)
    }

    /// Same as `entry_exists`, but returns a result instead of an option.
    /// # Errors
    /// Fails if the id does not exist. Forwards errors from `sqlx`.
    pub async fn assert_id(&self, table: Table, id: i32) -> Result<()> {
        if self.id_exists(table, id).await? {
            Ok(())
        } else {
            Err(ArchivistError::MissingID(table, id))
        }
    }

    /// Returns an error if the provided item collides with anything.
    /// # Errors
    /// Fails if there is a collision. Forwards errors from `sqlx`.
    pub async fn assert_unique<T>(&self, item: &T) -> Result<()>
    where
        T: TableItem,
    {
        let uniques = item.unique_values();
        if uniques.is_empty() {
            return Ok(());
        }

        let query = format!("select id from {} where {};", T::TABLE, uniques,);
        let id: Option<(i32,)> =
            sqlx::query_as(&query).fetch_optional(&self.pool).await?;

        id.map_or(Ok(()), |(id,)| {
            Err(ArchivistError::EntryAlreadyExists(
                item.insert_values(),
                T::TABLE.to_string(),
                id,
            ))
        })
    }

    /// Adds a new entry to `T::TABLE`, making sure no unique fields are
    /// duplicated.
    ///
    /// Returns the id of the newly inserted item.
    /// # Errors
    /// Fails if there are collisions in the table. Forwards errors from `sqlx`.
    pub async fn insert<T>(&mut self, item: T) -> Result<i32>
    where
        T: TableItem,
    {
        self.assert_unique(&item).await?;

        // Enter the item
        let query = format!(
            "insert into {}({}) values ({}) returning id;",
            T::TABLE,
            T::insert_columns(),
            item.insert_values(),
        );

        let tx = self.get_transaction().await?;
        let (id,) = sqlx::query_as(&query).fetch_one(&mut *tx).await?;

        Ok(id)
    }

    /// Gets all items from `T::TABLE`.
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn get_all<T>(&self) -> Result<Vec<T>>
    where
        T: TableItem,
    {
        let query = format!("select {} from {};", T::select(), T::TABLE,);

        let items = sqlx::query_as(&query).fetch_all(&self.pool).await?;

        Ok(items)
    }

    /// Finds an item from `T::TABLE`, fulfilling a `where`-condition.
    ///
    /// This is essentially just wrapping a query like `select T from TABLE
    /// where CONDITION;`.
    ///
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn find<T>(&self, condition: &str) -> Result<Option<T>>
    where
        T: TableItem,
    {
        let query = format!(
            "select {} from {} where {};",
            T::select(),
            T::TABLE,
            condition
        );

        let item = sqlx::query_as(&query).fetch_optional(&self.pool).await?;

        Ok(item)
    }

    /// Update an entry with the given `id` in the given `table`. `value` in
    /// this case is a string like `number = 2`, i.e. both the column and the
    /// actual value.
    ///
    /// Remember that string values need to be incased in single quotes.
    ///
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn update(
        &mut self,
        table: Table,
        id: i32,
        value: &str,
    ) -> Result<()> {
        self.assert_id(table, id).await?;

        let query = format!("update {table} set {value} where id={id};");

        let tx = self.get_transaction().await?;
        sqlx::query(&query).execute(tx).await?;

        Ok(())
    }

    /// Updates all columns for a the row with the supplied `id`.
    ///
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn update_from_cache<T>(
        &mut self,
        item: &T,
        id: i32,
    ) -> Result<()>
    where
        T: TableItem,
    {
        self.assert_id(T::TABLE, id).await?;

        let values = T::insert_columns()
            .split(',')
            .zip(item.insert_values().split(','))
            .map(|(col, val)| format!("{col}={val}"))
            .collect::<Vec<_>>()
            .join(",");

        let query =
            format!("update {} set {} where id={}", T::TABLE, values, id,);

        info!("q {query}");

        let tx = self.get_transaction().await?;
        sqlx::query(&query).execute(tx).await?;

        Ok(())
    }

    /// Gets an item whose id you know.
    ///
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn get<T>(&self, id: i32) -> Result<T>
    where
        T: TableItem,
    {
        self.assert_id(T::TABLE, id).await?;

        let query = format!(
            "select {} from {} where id={};",
            T::select(),
            T::TABLE,
            id,
        );
        let item = sqlx::query_as(&query).fetch_one(&self.pool).await?;

        Ok(item)
    }

    /// Deletes an item from a table. Make sure you are providing the correct
    /// type, as there is no way of checking your intentions!
    ///
    /// # Errors
    /// Fails if `id` does not exist. Forwards errors from `sqlx`.
    pub async fn delete<T>(&mut self, id: i32) -> Result<()>
    where
        T: TableItem,
    {
        if !self.id_exists(T::TABLE, id).await? {
            warn!(
                "Entry with id {id} does not exists and thus cannot be removed"
            );
            return Ok(());
        }

        let query = format!("delete from {} where id={};", T::TABLE, id,);

        let tx = self.get_transaction().await?;
        sqlx::query(&query).execute(tx).await?;

        Ok(())
    }

    /// Gets the indicated values from `table`, for one row if it meets
    /// `condition`.
    ///
    /// This may be preferred if you want only a specific value instead of the
    /// whole item, or a value that is not present in the rust-end struct, but
    /// is stored in the table (e.g. a password hash).
    ///
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn get_special<U>(
        &self,
        table: Table,
        columns: &str,
        condition: &str,
    ) -> Result<Option<U>>
    where
        for<'r> U: FromRow<'r, PgRow> + Send + Unpin,
    {
        let query = format!(
            "select {columns} from {table} where {condition} limit 1;",
        );

        let item = sqlx::query_as(&query).fetch_optional(&self.pool).await?;

        Ok(item)
    }

    /// Returns the currently live transaction. If there is none present, it
    /// first creates one.
    async fn get_transaction(&mut self) -> Result<&mut PgConnection> {
        if self.current_transaction.is_none() {
            warn!("Started implicit transaction.");
            self.current_transaction = Some(self.pool.begin().await?);
        }

        Ok(self.current_transaction.as_mut().unwrap())
    }

    /// The current configuration.
    pub const fn config(&self) -> &Config {
        &self.config
    }
}

impl Debug for Archivist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Archivist")
            .field("live:", &self.current_transaction.is_some())
            .finish_non_exhaustive()
    }
}
