//! Data for generated TOAs.

use crate::archivist::table::TableItem;
use item_macro::TableItem;

#[derive(Debug, sqlx::FromRow, TableItem)]
#[table(Toas)]
/// TOA information. This comes from `psrchive`.
pub struct TOAInfo {
    #[derived]
    id: i32,

    // Toaster has these ----------------
    /// The ID of the process that generated this info.
    pub process_id: i32,
    /// The ID of the template used.
    pub template_id: i32,
    /// The ID of the raw file used.
    pub rawfile_id: i32,

    // The data -------------------------
    /// The ID of the pulsar this belongs to.
    pub pulsar_id: i32,
    /// The ID of the observer that made the raw data.
    pub observer_id: i32,
    /// The integer part of the arrival time.
    pub toa_int: i32,
    /// The fractional part of the arrival time.
    pub toa_frac: f64,
    /// The error in the arrival time.
    pub toa_err: f32,
    /// The frequency of this observation.
    pub frequency: f32,
}

impl TOAInfo {
    #![allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
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
