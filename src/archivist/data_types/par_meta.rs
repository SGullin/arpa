//! Metadata for ephemerides.

use crate::{archivist::TableItem, conveniences::compute_checksum};
use item_macro::TableItem;
use sqlx::types::uuid;

#[derive(Debug, Clone, sqlx::FromRow, TableItem)]
#[table(ParMetas)]
/// The metadata of an ephemeride
pub struct ParMeta {
    /// Mandatory id.
    #[derived]
    pub id: i32,
    /// Storing the name here is redundant; the id links to the name and
    /// all aliases.
    pub pulsar_id: i32,
    #[unique]
    /// The 128 bit checksum of the file.
    pub checksum: uuid::Uuid,
    #[unique]
    /// The path to the actual file.
    pub file_path: String,
}
impl ParMeta {
    /// Creates a new ephemeride meta object.
    /// # Errors
    /// Will only pass on errors from the io calls made.
    pub fn new(
        file_path: String,
        pulsar_id: i32,
        block_size: usize,
    ) -> std::io::Result<Self> {
        let u128 = compute_checksum(&file_path, block_size, true)?;
        let checksum = uuid::Uuid::from_u128(u128);

        Ok(Self {
            id: 0,
            pulsar_id,
            checksum,
            file_path,
        })
    }
}
