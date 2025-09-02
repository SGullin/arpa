//! Diagnostic tools for the pipeline.

use crate::config::Config;
use crate::data_types::{archive_file, DiagnosticFloat, DiagnosticPlot};
use crate::{ARPAError, Archivist, Result};

mod composite;
mod snr;

/// The value of a diagnostic tool's output, either a plot or a float for now.
pub enum DiagnosticOut {
    /// A plot, with the inner argument being the path.
    Plot(String),
    /// A float value.
    Value(f32),
}

/// Runs an indicated diagnostic function and stores the result.
/// # Errors
/// Fails if the diagnositc tool fails, or the `archivist` can't do its thing.
pub async fn run_diagnostic(
    config: &Config,
    archivist: &mut Archivist,
    diagnostic: &str,
    process: i32,
    file: &str,
    directory: &str,
) -> Result<()> {
    let out = match diagnostic {
        "snr" => snr::run(config, file),
        "composite" => composite::run(config, file),

        other => Err(ARPAError::UnknownDiagnostic(other.to_string())),
    }?;

    match out {
        DiagnosticOut::Plot(mut path) => {
            _ = archive_file(
                config,
                &mut path,
                directory,
                &format!("{diagnostic}.png"),
            )?;

            let meta = DiagnosticPlot {
                id: 0,
                process,
                diagnostic: diagnostic.to_string(),
                filepath: path,
            };

            archivist.insert(meta).await?;
        }
        DiagnosticOut::Value(result) => {
            let meta = DiagnosticFloat {
                id: 0,
                process,
                diagnostic: diagnostic.to_string(),
                result,
            };

            archivist.insert(meta).await?;
        }
    }

    Ok(())
}
