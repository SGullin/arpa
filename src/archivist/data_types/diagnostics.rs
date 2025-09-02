//! Diagnostic entries.

use crate::TableItem;
use item_macro::TableItem;

#[derive(sqlx::FromRow, TableItem)]
#[table(DiagnosticFloats)]
/// An entry referring to a diagnostic wiht a float value.
pub struct DiagnosticFloat {
    /// Mandatory id.
    #[derived]
    pub id: i32,
    /// The process id that led to this.
    pub process: i32,
    /// The diagnostic name.
    pub diagnostic: String,
    /// The value of the result.
    pub result: f32,
}
#[derive(sqlx::FromRow, TableItem)]
#[table(DiagnosticPlots)]
/// An entry referring to a diagnostic plot.
pub struct DiagnosticPlot {
    /// Mandatory id.
    #[derived]
    pub id: i32,
    /// The process id that led to this.
    pub process: i32,
    /// The diagnostic name.
    pub diagnostic: String,
    /// The path to the plot.
    pub filepath: String,
}
