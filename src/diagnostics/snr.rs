//! Signal-to-noise ratio for fully scrunched data, using `psrchive::psrstat`.
use super::DiagnosticOut;
use crate::{
    Result, config::Config, conveniences::parse, external_tools::psrchive,
};
use log::info;

pub fn run(config: &Config, path: &str) -> Result<DiagnosticOut> {
    info!("Calculating SNR for {path}...");
    let res =
        psrchive(config, "psrstat", &["-Qq", "-j", "DTFp", "-c", "snr", path])?;

    Ok(DiagnosticOut::Value(parse(res.trim())?))
}
