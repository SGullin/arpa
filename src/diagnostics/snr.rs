//! Signal-to-noise ratio for fully scrunched data, using `psrchive::psrstat`.
use log::info;
use crate::{
    config::VolatileConfig, 
    conveniences::parse, 
    external_tools::psrchive,
    Result,
};
use super::DiagnosticOut;

pub fn run(config: &VolatileConfig, path: &str) -> Result<DiagnosticOut> {
    info!("Calculating SNR for {path}...");
    let res = psrchive(
        config,
        "psrstat", 
        &[
            "-Qq",
            "-j",
            "DTFp",
            "-c",
            "snr",
            path,
        ]
    )?;

    Ok(DiagnosticOut::Value(parse(res.trim())?))
}
