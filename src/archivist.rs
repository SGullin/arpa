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
use table::TableItem;

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
    pub async fn exists<T: TableItem>(&self, id: i32) -> Result<bool> {
        let exists = T::exists(&self.pool, id).await?;
        Ok(exists)
    }

    /// Same as `exists`, but returns a result instead of an option.
    /// # Errors
    /// Fails if the id does not exist. Forwards errors from `sqlx`.
    pub async fn assert_exists<T: TableItem>(&self, id: i32) -> Result<()> {
        if self.exists::<T>(id).await? {
            Ok(())
        } else {
            Err(ArchivistError::MissingID(T::TABLE, id))
        }
    }

    /// Returns an error if the provided item collides with anything.
    /// # Errors
    /// Fails if there is a collision. Forwards errors from `sqlx`.
    pub async fn assert_unique<T: TableItem>(&self, item: &T) -> Result<()> {
        item.check_unique(&self.pool).await?.map_or(Ok(()), |id| {
            Err(ArchivistError::EntryAlreadyExists(T::TABLE.to_string(), id))
        })
    }

    /// Gets an item whose id you know.
    ///
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn get<T: TableItem>(&self, id: i32) -> Result<T> {
        self.assert_exists::<T>(id).await?;
        T::select_by_id(&self.pool, id)
            .await
            .map_err(ArchivistError::Sqlx)
    }

    /// Gets all items from `T::TABLE`.
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn get_all<T: TableItem>(&self) -> Result<Vec<T>> {
        T::select_all(&self.pool)
            .await
            .map_err(ArchivistError::Sqlx)
    }

    /// Finds an item from `T::TABLE`, fulfilling a `where`-condition.
    ///
    /// This is essentially just wrapping a query like `select T from TABLE
    /// where CONDITION;`.
    ///
    /// Due to the flexibility in the `condition` parameter, this is not run
    /// via any macro, and as such cannot be compile-time tested. Use
    /// responsibly.
    ///
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn find<T: TableItem>(&self, condition: &str, ) -> Result<Option<T>> {
        let query = format!("select id from {} where {};", T::TABLE, condition);

        let opt_id: Option<(i32,)> =
            sqlx::query_as(&query).fetch_optional(&self.pool).await?;

        let id = match opt_id {
            None => return Ok(None),
            Some((i,)) => i,
        };

        T::select_by_id(&self.pool, id)
            .await
            .map_err(ArchivistError::Sqlx)
            .map(|i| Some(i))
    }

    /// Adds a new entry to `T::TABLE`, making sure no unique fields are
    /// duplicated.
    ///
    /// Returns the id of the newly inserted item.
    /// # Errors
    /// Fails if there are collisions in the table. Forwards errors from `sqlx`.
    pub async fn insert<T: TableItem>(&mut self, item: T) -> Result<i32> {
        self.assert_unique(&item).await?;
        let tx = self.get_transaction().await?;

        item.insert(tx).await.map_err(ArchivistError::Sqlx)
    }

    /// Update an entry with the given `id` in the given `table`. `value` in
    /// this case is a string like `number = 2`, i.e. both the column and the
    /// actual value.
    ///
    /// Remember that string values need to be incased in single quotes.
    ///
    /// Due to the flexibility in the `value` parameter, this is not run via
    /// any macro, and as such cannot be compile-time tested. Use responsibly.
    ///
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn update<T: TableItem>(&mut self, id: i32, value: &str, ) -> Result<()> {
        self.assert_exists::<T>(id).await?;

        let query = format!("update {} set {value} where id={id};", T::TABLE,);

        let tx = self.get_transaction().await?;
        sqlx::query(&query).execute(tx).await?;

        Ok(())
    }

    /// Updates all columns for a the row with the supplied `id`.
    ///
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn update_from_cache<T: TableItem>(&mut self, cache: &T, id: i32,) -> Result<()> {
        self.assert_exists::<T>(id).await?;
        let tx = self.get_transaction().await?;

        cache.update(tx, id).await.map_err(ArchivistError::Sqlx)
    }

    /// Deletes an item from a table. Make sure you are providing the correct
    /// type, as there is no way of checking your intentions!
    ///
    /// # Errors
    /// Fails if `id` does not exist. Forwards errors from `sqlx`.
    pub async fn delete<T: TableItem>(&mut self, id: i32) -> Result<()> {
        if !self.exists::<T>(id).await? {
            warn!(
                "Entry with id {id} does not exists and thus cannot be removed"
            );
            return Ok(());
        }

        let tx = self.get_transaction().await?;
        T::delete(tx, id).await.map_err(ArchivistError::Sqlx)
    }

    /// Gets the indicated values from `table`, for one row if it meets
    /// `condition`.
    ///
    /// This may be preferred if you want only a specific value instead of the
    /// whole item, or a value that is not present in the rust-end struct, but
    /// is stored in the table (e.g. a password hash).
    ///
    /// Due to the flexibility in the parameters, this is not run via any
    /// macro, and as such cannot be compile-time tested. Use responsibly.
    ///
    /// # Errors
    /// Forwards errors from `sqlx`.
    pub async fn get_special<T: TableItem, U>(&self, columns: &str, condition: &str, ) -> Result<Option<U>> where for<'r> U: FromRow<'r, PgRow> + Send + Unpin {
        let query = format!(
            "select {columns} from {} where {condition} limit 1;",
            T::TABLE,
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
