//! Metadata for a template file.

use crate::archivist::table::TableItem;
use crate::conveniences::compute_checksum;
use item_macro::TableItem;
use sqlx::{prelude::FromRow, types::uuid};

#[derive(FromRow, Clone, TableItem)]
#[table(TemplateMetas)]
/// Metadata for a template file.
pub struct TemplateMeta {
    /// Mandatory id.
    #[derived]
    pub id: i32,

    /// ID of pulsar it belongs to.
    pub pulsar_id: i32,

    /// Path to file.
    #[unique]
    pub file_path: String,

    /// 128 bit checksum.
    #[unique]
    pub checksum: uuid::Uuid,
}
impl TemplateMeta {
    /// Creates a new template metafile.
    ///
    /// # Errors
    /// Fails if the file can't be read.
    pub fn new(
        file_path: String,
        pulsar_id: i32,
    ) -> std::io::Result<Self> {
        let u128 = compute_checksum(&file_path, true)?;
        let checksum = uuid::Uuid::from_u128(u128);

        Ok(Self {
            id: 0,
            pulsar_id,
            file_path,
            checksum,
        })
    }
}
