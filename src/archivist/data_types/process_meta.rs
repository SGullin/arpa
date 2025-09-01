use item_macro::TableItem;
use sqlx::prelude::FromRow;
use crate::archivist::table::TableItem;

#[derive(FromRow, Clone, TableItem)]
#[table(ProcessMetas)]
pub struct ProcessMeta {
    #[derived]
    pub id: i32,
    
    pub raw_id: i32,
    pub par_id: Option<i32>,
    pub template_id: i32,
    pub n_channels: i16,
    pub n_subints: i16,
    pub method: String,
    pub user_id: i32,
}
