//! Functions to call external tools.

use std::{ffi::OsStr, process::Command};

use crate::{Result, config::Config};
use log::{debug, info, warn};

/// Runs a psrchive tool `tool`, and returns its result.
/// # Errors
/// Fails if the tool cannot be called, if the tool fails, or if the tool's
/// output is not UTF-8.
pub fn psrchive(config: &Config, tool: &str, args: &[impl AsRef<OsStr>]) -> Result<String> {
    debug!(
        "Running psrchive::{}, with the following arguments: [{}\n]",
        tool,
        args.iter().fold(
            String::new(), 
            |acc, a|  acc + "\n\t" + &a.as_ref().to_string_lossy()
        ),
    );

    let tool_path = if config.paths.psrchive.is_empty() {
        tool.to_string()
    } else {
        format!("{}/{}", config.paths.psrchive, tool)
    };

    let output = Command::new(tool_path).args(args).output()?;

    // if !output.status. {
    //     return Err(ARPAError::ToolFailure(

    //     );
    // }

    if !output.stderr.is_empty() {
        warn!(
            "Tool printed the following to stderr: \n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    debug!(
        "-- stdout:\n{}\n-- stderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let result = String::from_utf8(output.stdout)?;
    Ok(result)
}

/// Calls `tempo2` to perform a fit.
/// # Errors
/// Fails if tempo fails.
pub fn tempo2_fit(par_file: &str, tim_file: &str) -> Result<()> {
    let result = Command::new("tempo2")
        .arg("-f")
        .arg(par_file)
        .arg(tim_file)
        .status()?;

    info!("{result}");

    Ok(())
}
