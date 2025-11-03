//! Signal-to-noise ratio for fully scrunched data, using `psrchive::psrstat`.
use std::path::Path;

use super::DiagnosticOut;
use crate::{
    Result, config::Config, conveniences::parse, external_tools::psrchive,
};
use log::info;

pub fn run(config: &Config, path: impl AsRef<Path>) -> Result<DiagnosticOut> {
    info!("Calculating SNR for {}...", path.as_ref().display());
    let res = psrchive(
        config,
        "psrstat",
        &[
            "-Qq",
            "-j",
            "DTFp",
            "-c",
            "snr",
            &path.as_ref().display().to_string(),
        ],
    )?;

    Ok(DiagnosticOut::Value(parse(res.trim())?))
}
