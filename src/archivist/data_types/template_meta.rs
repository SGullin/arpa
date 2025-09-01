use crate::{conveniences::compute_checksum};
use crate::archivist::table::TableItem;
use item_macro::TableItem;
use sqlx::{prelude::FromRow, types::uuid};

#[derive(FromRow, Clone, TableItem)]
#[table(TemplateMetas)]
pub struct TemplateMeta {
    #[derived]
    pub id: i32,
    pub pulsar_id: i32,
    #[unique]
    pub file_path: String,
    #[unique]
    pub checksum: uuid::Uuid,
}
impl TemplateMeta {
    /// Creates a new template metafile.
    /// 
    /// # Errors
    /// Fails if the file can't be read.
    pub fn new(file_path: String, pulsar_id: i32) -> std::io::Result<Self> {
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
