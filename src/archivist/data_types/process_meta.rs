//! Process information.

use crate::{archivist::table::TableItem, data_types::ParMeta};
use item_macro::TableItem;
use sqlx::prelude::FromRow;

#[derive(FromRow, Clone, TableItem)]
#[table(ProcessMetas)]
/// The information of a process
pub struct ProcessInfo {
    /// Mandatory id.
    #[derived]
    id: i32,

    /// ID of the raw file.
    pub raw_id: i32,
    /// ID of the ephemeride.
    pub par_id: Option<i32>,
    /// ID of the template.
    pub template_id: i32,
    /// Number of channels.
    pub n_channels: i16,
    /// Nubmer of subintervals.
    pub n_subints: i16,
    /// Which methoid to use when treating the file.
    pub method: String,
    /// Which user launched the process.
    pub user_id: i32,
}
impl ProcessInfo {
    pub(crate) fn new(
        user_id: i32, 
        raw: &super::RawMeta, 
        ephemeride: Option<&ParMeta>, 
        template: &super::TemplateMeta, 
        n_channels: i16, 
        n_subints: i16, 
        method: &str
    ) -> Self {
        Self { 
            id: 0, 
            raw_id: raw.id,
            par_id: ephemeride.map(|e| e.id), 
            template_id: template.id, 
            n_channels,
            n_subints, 
            method: method.to_string(), 
            user_id
        }
    }
}
