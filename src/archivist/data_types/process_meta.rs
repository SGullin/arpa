//! Process information.

use crate::archivist::table::TableItem;
use item_macro::TableItem;
use sqlx::prelude::FromRow;

#[derive(FromRow, Clone, TableItem)]
#[table(ProcessMetas)]
/// The information of a process
pub struct ProcessInfo {
    /// Mandatory id.
    #[derived]
    pub id: i32,

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
