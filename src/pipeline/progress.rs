use crate::conveniences::display_elapsed_time;

#[derive(Debug, Default)]
/// Represents the current status of the pipeline.
pub enum Status {
    /// The pipeline is not active. Don't expect to ever receive this status
    /// in a callback or such, it is just here for completeness sake.
    #[default]
    Idle,

    /// Some error ocurred in the process.
    Error(String),

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

    /// Copying a file from `.0` to `.1`.
    Copying(String, String),

    /// An ephemeride was provided, and so it is being installed.
    InstallingEphemeride,

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
        passed: bool,
    },

    /// Archived the plots from `psrchive::pat` (with count and whether it
    /// passed provided).
    ArchivedTOAPlots(Option<usize>),

    /// The pipeline just finished (with total duration provided).
    Finished(std::time::Duration),
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idling..."),
            Self::Error(err) => write!(f, "Encountered error: {err}"),

            Self::Starting {
                raw,
                pulsar,
                ephemeride,
                template,
            } => write!(
                f,
                "Cooking with the following:\
                \n * Raw file:   {}\
                \n               id = {}\
                \n * Pulsar:     {} \
                \n               id = {}\
                \n * Ephemeride: {}\
                \n * Template:   id = {}\n",
                raw.0,
                raw.1,
                pulsar.0,
                pulsar.1,
                ephemeride.as_ref().map_or_else(
                    || "(None)\n".into(),
                    |e| format!("{}\n               id = {}", e.0, e.1),
                ),
                template,
            ),

            Self::InstallingEphemeride => write!(f, "Installing ephemeride..."),
            Self::Copying(src, dst) => write!(f, "Copying from {src} to {dst}"),
            Self::Manipulating => write!(f, "Manipulating..."),
            Self::VerifyingTemplate => write!(f, "Verifying template..."),
            Self::GeneratingTOAs => write!(f, "Generating TOAs..."),
            Self::GotTOAs(n) => write!(f, "Got {n} TOA(s)!"),
            Self::LoggingProcess => write!(f, "Logging process..."),
            Self::ParsingTOAs => write!(f, "Parsing TOAs..."),
            Self::ArchivedTOAs(n) => write!(f, "Archived {n} TOA(s)!"),
            Self::Diagnosing(n) => write!(f, "Running {n} diagnostic(s)..."),

            Self::FinishedDiagnostic { diagnostic, passed } => write!(
                f,
                "Finished diagnostic {diagnostic}{}",
                if *passed {
                    " with no problems."
                } else {
                    ", but an error ocurred."
                },
            ),

            Self::ArchivedTOAPlots(Some(n)) => {
                write!(f, "Archived {n} plot(s) from psrchive::pat.")
            }
            Self::ArchivedTOAPlots(None) => {
                write!(f, "Failed to archive plot(s) from psrchive::pat.")
            }

            Self::Finished(dt) => {
                write!(f, "Finished in {}!", display_elapsed_time(*dt))
            }
        }
    }
}
