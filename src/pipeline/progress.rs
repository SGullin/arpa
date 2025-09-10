use log::{debug, info};

use crate::conveniences::display_elapsed_time;

#[derive(Debug)]
/// Represents the current status of the pipeline.
pub enum Status {
    /// The pipeline is just starting.
    Starting {
        /// Raw file path and id.
        raw: (String, i32),
        /// Pulsar alias and id.
        pulsar: (String, i32),
        /// Ephemeride path and id, if any.
        ephemeride: Option<(String, i32)>,
        /// Template id.
        template: i32,
    },

    /// An ephemeride was provided, and so it is being installed.
    InstallingEphemeride,

    /// Copying a file from `.0` to `.1`.
    Copying(String, String),

    /// Manipulating the file using `psrchive::pam`.
    Manipulating,
    
    /// Verifying that the template is safe and sound.
    VerifyingTemplate,

    /// Generating TOAs with `psrchive::pat`.
    GeneratingTOAs,

    /// TOAs received (with count provided).
    GotTOAs(usize),
    
    /// Logging `ProcessMeta` to DB.
    LoggingProcess,

    /// Parsing the TOA information from `psrchive::pat`.
    ParsingTOAs,

    /// Successfully archived TOAs (with count provided).
    ArchivedTOAs(usize),

    /// Starts diagnosing (with count provided).
    Diagnosing(usize),

    /// Finished a diagnostic task.
    FinishedDiagnostic { 
        /// The kind of diagnostic performed.
        diagnostic: String, 
        /// Whether it ran ok.
        passed: bool 
    },
    
    /// Archived the plots from `psrchive::pat` (with count provided).
    ArchivedTOAPlots(usize),

    /// The pipeline just finished (with total duration provided).
    Finished(std::time::Duration),
}

impl Status {
    /// Logs itself via `log::info` or `log::debug`, depending on perceived 
    /// importance.
    pub fn log(self) {
        match self {
            Self::Starting { raw, pulsar, ephemeride, template } => info!(
                "Cooking with the following:\
                \n * Raw file:   {}\
                \n               id = {}\
                \n * Pulsar:     {} \
                \n               id = {}\
                \n * Ephemeride: {}\
                \n * Template:   id = {}\n",
                raw.0, raw.1,
                pulsar.0, pulsar.1,
                ephemeride.as_ref().map_or_else(
                    || "(None)\n".into(), 
                    |e| format!("{}\\n               id = {}", e.0, e.1),
                ),
                template,
            ),

            Self::InstallingEphemeride => info!("Installing ephemeride..."),
            Self::Copying(src, dst) => debug!("Copying from {src} to {dst}"),
            Self::Manipulating => info!("Manipulating..."),
            Self::VerifyingTemplate => info!("Verifying template..."),
            Self::GeneratingTOAs => info!("Generating TOAs..."),
            Self::GotTOAs(n) => info!("Got {n} TOA(s)!"),
            Self::LoggingProcess => info!("Logging process..."),
            Self::ParsingTOAs => info!("Parsing TOAs..."),
            Self::ArchivedTOAs(n) => info!("Archived {n} TOA(s)!"),
            Self::Diagnosing(n) => info!("Running {n} diagnostic(s)..."),
            
            Self::FinishedDiagnostic { diagnostic, passed } => info!(
                "Finished diagnostic {diagnostic}{}",
                if passed { " with no problems." }
                else { ", but an error ocurred." },
            ),

            Self::ArchivedTOAPlots(n) => 
                info!("Archived {n} plot(s) from psrchive::pat."),

            Self::Finished(dt) => info!(
                "Finished in {}!", 
                display_elapsed_time(dt)
            ),
        }
    }
}

