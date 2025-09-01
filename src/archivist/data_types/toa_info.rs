use item_macro::TableItem;
use crate::archivist::table::TableItem;

#[derive(sqlx::FromRow, TableItem)]
#[table(Toas)]
pub struct TOAInfo {
    #[derived]
    id: i32,

    // Toaster has these ----------------
    process_id: i32,
    template_id: i32,
    rawfile_id: i32,
    
    // The data -------------------------
    pulsar_id: i32,
    observer_id: i32,
    toa_int: i32,
    toa_frac: f64,
    toa_err: f32,
    frequency: f32,
}

impl TOAInfo {
    #![allow(
        clippy::cast_possible_wrap,
        clippy::cast_possible_truncation)]
    /// Extracts `arpa`-relevant information from a `psrutils` struct.
    pub fn extract(
        toa: &psrutils::timfile::TOAInfo,
        pulsar_id: i32,
        observer_id: i32,
        process_id: i32,
        template_id: i32,
        rawfile_id: i32,
    ) -> Self {
        let toa_int = toa.mjd.int() as i32;
        let toa_frac = toa.mjd.frac();

        Self {
            id: 0,
            process_id,
            template_id,
            rawfile_id,

            pulsar_id,
            observer_id,
            toa_int,
            toa_frac,
            toa_err: toa.mjd_error as f32,
            frequency: toa.frequency as f32,
        }
    }
}

