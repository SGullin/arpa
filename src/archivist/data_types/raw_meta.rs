use item_macro::TableItem;
use sqlx::{prelude::FromRow, types::uuid};
use crate::archivist::table::TableItem;

#[derive(FromRow, Clone, TableItem)]
#[table(RawMetas)]
pub struct RawMeta {
    #[derived]
    pub id: i32,

    #[unique]
    pub file_path: String,
    #[unique]
    pub checksum: uuid::Uuid,
    
    pub pulsar_id: i32,
    pub observer_id: i32,
}
