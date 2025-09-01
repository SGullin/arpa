use item_macro::TableItem;
use crate::TableItem;

#[derive(sqlx::FromRow, TableItem)]
#[table(DiagnosticFloats)]
pub struct DiagnosticFloat { 
    pub id: i32,
    pub process: i32,
    pub diagnostic: String,
    pub result: f32,
}
#[derive(sqlx::FromRow, TableItem)]
#[table(DiagnosticPlots)]
pub struct DiagnosticPlot { 
    pub id: i32,
    pub process: i32,
    pub diagnostic: String,
    pub filepath: String,
}
