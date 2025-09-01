//! This module contains two sets of configuration parameters: `volatile`, 
//! loaded from a file at boot; and `stable`, set as constants in this file
//! that need a recompile to change.

use serde::Deserialize;
use crate::ARPAError;

#[derive(Deserialize)]
pub struct VolatileConfig {
    pub database: Database,
    pub behaviour: Behaviour,
    pub paths: Paths,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize)]
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
}

#[derive(Deserialize)]
pub struct Database {
    /// Here we use a local postgre server (the postgres app) for testing
    pub url: String,
    /// Not too sure on what's a good number here...
    pub pool_connections: u32,
    /// 4 seconds is plenty, no? I hope so...
    pub connection_timeout: u64,
}

#[derive(Deserialize)]
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
impl VolatileConfig {
    /// Reads config from a `.toml` file. 
    /// # Errors
    /// File can't be read, or file contents don't match config struct.
    pub fn load() -> Result<Self, ARPAError> {     
        let data = std::fs::read_to_string(stable::CONFIG_PATH)?;
        let config = toml::from_str(&data)?;

        Ok(config)
    }
}

pub mod stable {
    // --- Database settings --------------------------------------------------
    /// Name of the directory where sql setup commands live.
    pub const SQL_SETUP_DIR: &str = "sql";
    /// Path to the config file.
    pub const CONFIG_PATH: &str = "./config.toml";

    // --- Behaviour ----------------------------------------------------------
    /// The number of bytes to buffer when reading checksums. 
    /// 
    /// FYI, changing this after deployment will break compatibility with any 
    /// previous files. I _strongly_ suggest you find a favourite value before
    /// then.
    pub const CHECKSUM_BLOCK_SIZE: usize = 16 * 16 * 8192; // 2 Mb
}
