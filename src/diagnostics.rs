use crate::config::VolatileConfig;
use crate::data_types::diagnostics::{DiagnosticFloat, DiagnosticPlot};
use crate::data_types::raw_file::archive_file;
use crate::{ARPAError, Archivist, Result};

mod snr;
mod composite;

pub enum DiagnosticOut {
    Plot(String),
    Value(f32),
}

/// Runs an indicated diagnostic function and stores the result.
/// # Errors
/// Fails if the diagnositc tool fails, or the `archivist` can't do its thing.
pub async fn run_diagnostic(
    config: &VolatileConfig, 
    archivist: &mut Archivist,
    diagnostic: &str, 
    process: i32,
    file: &str,
    directory: &str,
) -> Result<()> {
    let out = match diagnostic {
        "snr" => snr::run(config, file),
        "composite" => composite::run(config, file),

        other => Err(ARPAError::UnknownDiagnostic(other.to_string()))
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
        },
        DiagnosticOut::Value(result) => {
            let meta = DiagnosticFloat {
                id: 0,
                process,
                diagnostic: diagnostic.to_string(),
                result,
            };

            archivist.insert(meta).await?;
        },
    }

    Ok(())
}
