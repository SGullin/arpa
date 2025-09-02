use super::Table;

#[derive(Debug)]
pub enum ArchivistError {
    Sqlx(sqlx::Error),

    EntryAlreadyExists(String, String, i32),

    NoTransactionToCommit,
    NoTransactionToRollback,
    TransactionAlreadyLive,

    MissingID(Table, i32),
}

impl std::fmt::Display for ArchivistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlx(error) => write!(f, "[sqlx] {error}",),

            Self::EntryAlreadyExists(key, table, id) => write!(
                f,
                "({key}) conflicts with preexisting entry (id = {id}) in {table}",
            ),

            Self::NoTransactionToCommit => write!(
                f,
                "Archivist was asked to commit a transaction, but none had \
                begun."
            ),
            Self::NoTransactionToRollback => write!(
                f,
                "Archivist was asked to rollback a transaction, but none had \
                begun."
            ),
            Self::TransactionAlreadyLive => write!(
                f,
                "Archivist was asked to start a transaction, but one is \
                already live."
            ),

            Self::MissingID(table, id) => write!(
                f,
                "There is no entry with id {id} in table \"{table}\".",
            ),
        }
    }
}

impl From<sqlx::Error> for ArchivistError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
    }
}
