//! This module contains configuration that should ideally be loaded from a
//! file. It is subdivided into categories.
//!
//! [`crate::Archivist`] calls [`Config::load`] upon creation, so it should all be
//! automatic.

use std::path::Path;

use crate::ARPAError;
use serde::Deserialize;

#[derive(Deserialize)]
/// Supergroup of configuration options.
pub struct Config {
    /// Relating to the database connection.
    pub database: Database,
    /// Decsribing pipeline behaviour.
    pub behaviour: Behaviour,
    /// A collection of paths.
    pub paths: Paths,
}

#[derive(Deserialize)]
/// Relating to the database connection.
pub struct Database {
    /// Here we use a local postgre server (the postgres app) for testing
    pub url: String,
    /// Not too sure on what's a good number here...
    pub pool_connections: u32,
    /// 4 seconds is plenty, no? I hope so...
    pub connection_timeout: u64,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize)]
/// Decsribing pipeline behaviour.
pub struct Behaviour {
    /// Whether to archive raw files in a location determined by their header
    /// data.
    pub archive_rawfiles: bool,

    /// Whether to move raw files, instead of copying, when archiving.
    pub move_rawfiles: bool,

    /// Whether to automatically add unregistered encountered pulsars.
    pub auto_add_pulsars: bool,

    /// If a file is picked, but something with the same checksum is already  
    /// in the DB, do not thrown an error. Instead, pick the old file.
    pub auto_resolve_duplicate_uploads: bool,

    /// Which method to use for fitting TOAs.
    pub toa_fitting: String,

    /// The diagnostics to perform on cooked raw files.
    pub diagnostics: Vec<String>,

    /// The number of bytes to buffer when reading checksums.
    ///
    /// FYI, changing this after deployment will break compatibility with any
    /// previous files. I _strongly_ suggest you find a favourite value before
    /// then.
    pub checksum_block_size: usize,
}

#[derive(Deserialize)]
/// A collection of paths.
pub struct Paths {
    /// Path to psrchive executables.
    pub psrchive: String,
    /// The path of the root of the raw files' storage directory.
    pub rawfile_storage: String,
    /// The root directory for temporary files.
    pub temp_dir: String,
    /// The root dir for all diagnostics.
    pub diagnostics_dir: String,
}
impl Config {
    /// Reads config from a `.toml` file.
    ///
    /// # Errors
    /// File can't be read, or file contents don't match config struct.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ARPAError> {
        let data = std::fs::read_to_string(path)?;
        let config = toml::from_str(&data)?;

        Ok(config)
    }
}
