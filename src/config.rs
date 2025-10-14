//! This module contains configuration that should ideally be loaded from a
//! file. It is subdivided into categories.
//!
//! [`crate::Archivist`] calls [`Config::load`] upon creation, so it should all be
//! automatic.

use std::path::Path;

use crate::ARPAError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
/// Supergroup of configuration options.
pub struct Config {
    /// Relating to the database connection.
    pub database: Database,
    /// Decsribing pipeline behaviour.
    pub behaviour: Behaviour,
    /// A collection of paths.
    pub paths: Paths,
}

#[derive(Deserialize, Serialize)]
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
#[derive(Deserialize, Serialize)]
/// Decsribing pipeline behaviour.
pub struct Behaviour {
    /// Whether to automatically add unregistered encountered pulsars.
    pub auto_add_pulsars: bool,

    /// If a file is picked, but something with the same checksum is already  
    /// in the DB, do not thrown an error. Instead, pick the old file.
    pub auto_resolve_duplicate_uploads: bool,

    /// Which method to use for fitting TOAs.
    pub toa_fitting: String,

    /// The diagnostics to perform on cooked raw files.
    pub diagnostics: Vec<String>,
}

#[derive(Deserialize, Serialize)]
/// A collection of paths.
pub struct Paths {
    /// Path to psrchive executables.
    pub psrchive: String,
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

    /// Saves the configuration to a file.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ARPAError> {
        let text = toml::to_string(self)?;
        std::fs::write(path, text)?;

        Ok(())
    }
}
impl Default for Config {
    /// Provides sensible default settings. 
    /// Note that this does not set paths, however.
    fn default() -> Self {
        let database = Database {
            url: "postgresql://localhost:5437".into(),
            pool_connections: 4,
            connection_timeout: 4000,
        };

        let behaviour = Behaviour {
            auto_add_pulsars: true,
            auto_resolve_duplicate_uploads: true,
            toa_fitting: "FDM".into(),
            diagnostics: vec!["snr".into(), "composite".into()],
        };

        let paths = Paths {
            psrchive: String::new(),
            temp_dir: String::new(),
            diagnostics_dir: String::new(),
        };

        Self { 
            database, 
            behaviour,
            paths,
        }
    }
}
