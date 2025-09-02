use std::{process::Output, string::FromUtf8Error};

use crate::{archivist::ArchivistError, conveniences::comma_separate};

#[derive(Debug)]
#[allow(missing_docs)]
pub enum ARPAError {
    TokioJoinError(tokio::task::JoinError),
    IOFault(std::io::Error),
    PSRUtils(psrutils::error::PsruError),
    ToolFailure(String, Output),
    JoinThread(String),
    ConfigFailure(toml::de::Error),
    MissingFileOrDirectory(String),
    StringConversion(Vec<u8>),
    ArchivistError(ArchivistError),

    MalformedInput(String),
    ParseFailed(String, &'static str),
    FileCopy(u128, u128, u64, u64),

    CantFind(String),

    ChefNoEphemeride,
    ChefNoTemplate,
    ChefNoRaw,
    VapKeyCount(usize, usize),

    UnknownDiagnostic(String),
    DiagnosticPlotBadFile(String),
}

impl std::fmt::Display for ARPAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TokioJoinError(error) => write!(f, "[tokio] {error}",),
            Self::IOFault(error) => write!(f, "[std::io] {error}",),
            Self::PSRUtils(error) => write!(f, "[psrutils] {error}",),
            Self::ToolFailure(tool, out) => write!(
                f,
                "Tool \"{}\" failed{}\n-- stdout:\n{}\n-- stderr:\n{}",
                tool,
                out.status.code().map_or_else(
                    || "(codeless)".into(),
                    |c| format!("(code: {c})")
                ),
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr),
            ),
            Self::JoinThread(msg) => write!(
                f,
                "One of your threads was unable to join, saying: \"{msg}\"",
            ),
            Self::ConfigFailure(err) => {
                write!(f, "Encountered error reading config file: {err}",)
            }
            Self::MissingFileOrDirectory(path) => {
                write!(f, "File or directory \"{path}\" is missing.",)
            }
            Self::StringConversion(bytes) => {
                write!(f, "Failed to parse string from bytes: {bytes:?}",)
            }
            Self::ArchivistError(err) => {
                write!(f, "Archivist failed action.\n{err}")
            }

            Self::MalformedInput(comment) => {
                write!(f, "Malformed input: {comment}.",)
            }
            Self::ParseFailed(data, type_) => {
                write!(f, "Failed to parse \"{data}\" as {type_}",)
            }
            Self::FileCopy(src_cs, dst_cs, src_sz, dst_sz) => write!(
                f,
                "Copying file failed! \n\tchecksum: {} -> {}\n\tsize: {} -> \
                {}",
                src_cs,
                dst_cs,
                comma_separate(src_sz),
                comma_separate(dst_sz),
            ),

            Self::CantFind(thing) => write!(f, "Could not find {thing}.",),

            Self::ChefNoRaw => {
                write!(f, "Somehow we got here without a rawfile...")
            }
            Self::ChefNoEphemeride => {
                write!(f, "Cannot build chef without ephemeride.")
            }
            Self::ChefNoTemplate => {
                write!(f, "Cannot build chef without template.")
            }
            Self::VapKeyCount(keys, values) => write!(
                f,
                "Psrchive::vap was asked for {keys} values but returned \
                {values}.",
            ),

            Self::UnknownDiagnostic(dia) => {
                write!(f, "\"{dia}\" is not a recognised diagnostic tool.",)
            }
            Self::DiagnosticPlotBadFile(file) => {
                write!(f, "Can't figure out what you want to plot from {file}.",)
            }
        }
    }
}

impl From<tokio::task::JoinError> for ARPAError {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::TokioJoinError(value)
    }
}
impl From<std::io::Error> for ARPAError {
    fn from(value: std::io::Error) -> Self {
        Self::IOFault(value)
    }
}
impl From<psrutils::error::PsruError> for ARPAError {
    fn from(value: psrutils::error::PsruError) -> Self {
        Self::PSRUtils(value)
    }
}
impl From<FromUtf8Error> for ARPAError {
    fn from(value: FromUtf8Error) -> Self {
        Self::StringConversion(value.into_bytes())
    }
}
impl From<toml::de::Error> for ARPAError {
    fn from(value: toml::de::Error) -> Self {
        Self::ConfigFailure(value)
    }
}
impl From<ArchivistError> for ARPAError {
    fn from(value: ArchivistError) -> Self {
        Self::ArchivistError(value)
    }
}
