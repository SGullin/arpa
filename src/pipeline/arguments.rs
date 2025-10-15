use std::path::Path;

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    ARPAError, Archivist, Result,
    conveniences::assert_exists,
    data_types::{ParMeta, RawMeta, TemplateMeta},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A struct to keep all the necessary settings for the pipeline to run.
pub struct PipelineSettings {
    pub(super) diagnostics: bool,

    // Pam settings
    pub(super) time_scrunch_mode: TimeScrunchMode,
    pub(super) bin_scrunch_mode: BinScrunchMode,
    pub(super) channel_count: usize,
    //calibration settings..?
}
impl PipelineSettings {
    /// Reads the settings from a `.toml` file.
    ///
    /// # Errors
    /// Forwarded from `toml` and `std::fs`.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;

        toml::from_str(&data).map_err(ARPAError::ConfigLoadFailure)
    }

    /// Writes the settings to a `.toml` file.
    ///
    /// # Errors
    /// Forwarded from `toml` and `std::fs`.
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let data = toml::to_string(self)?;

        std::fs::write(path, data).map_err(ARPAError::IOFault)
    }

    #[must_use]
    /// Disables the post-TOA diagnostics.
    pub const fn disable_diagnostics(mut self) -> Self {
        self.diagnostics = false;
        self
    }

    #[must_use]
    /// Sets time crunch options.
    pub const fn time_scrunch_mode(mut self, value: TimeScrunchMode) -> Self {
        self.time_scrunch_mode = value;
        self
    }

    #[must_use]
    /// Sets bin crunch options.
    pub const fn bin_scrunch_mode(mut self, value: BinScrunchMode) -> Self {
        self.bin_scrunch_mode = value;
        self
    }

    #[must_use]
    /// Sets number of channels.
    pub const fn channel_count(mut self, value: usize) -> Self {
        self.channel_count = value;
        self
    }
}
impl Default for PipelineSettings {
    fn default() -> Self {
        Self {
            diagnostics: true,
            time_scrunch_mode: TimeScrunchMode::None,
            bin_scrunch_mode: BinScrunchMode::None,
            channel_count: 1,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeScrunchMode {
    None,
    ByFactor(f32),
    SubIntCount(usize),
    SubIntLength(f32),
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinScrunchMode {
    None,
    Count(usize),
}

/// Parses `text` to load a `ParMeta`. This will try two things:
///  1) parsing as an `i32`: if successful, it will look for an existing entry
///     with that id; or
///  2) treating as a path: if a file is found, it will try to upload it and
///     then use it.
///
/// # Errors
/// Any error will come from either the `archivist` failing or a file not being
/// ok.
pub async fn parse_input_ephemeride(
    archivist: &mut Archivist,
    raw: &RawMeta,
    text: &str,
) -> Result<ParMeta> {
    match text.parse() {
        Ok(id) => archivist.get(id).await.map_err(Into::into),
        Err(_) => ephermeride_from_file(archivist, raw, text).await,
    }
}

async fn ephermeride_from_file(
    archivist: &mut Archivist,
    raw: &RawMeta,
    path: &str,
) -> Result<ParMeta> {
    debug!("Parsing ephemeride path");
    assert_exists(path)?;

    // Insert the file into the table
    info!("Inserting ephemeride {path}");
    let mut meta = ParMeta::new(path.to_string(), raw.pulsar_id)?;

    // If auto resolve dupes is off, we just insert
    if !archivist.config().behaviour.auto_resolve_duplicate_uploads {
        meta.id = archivist.insert(meta.clone()).await?;
        return Ok(meta);
    }

    // Otherwise, we check for pre-existing file
    let existing = archivist
        .find::<ParMeta>(&format!("checksum='{}'", meta.checksum))
        .await?;

    if let Some(pm) = existing {
        warn!(
            "Ephemeride with checksum {} already exists! Picking it instead.",
            pm.checksum,
        );
        Ok(pm)
    } else {
        meta.id = archivist.insert(meta.clone()).await?;
        Ok(meta)
    }
}

/// Parses `text` to load a `TemplateMeta`. This will try two things:
///  1) parsing as an `i32`: if successful, it will look for an existing entry
///     with that id; or
///  2) treating as a path: if a file is found, it will try to upload it and
///     then use it.
///
/// # Errors
/// Any error will come from either the `archivist` failing or a file not being
/// ok.
pub async fn parse_input_template(
    archivist: &mut Archivist,
    raw: &RawMeta,
    text: &str,
) -> Result<TemplateMeta> {
    match text.parse() {
        Ok(id) => archivist.get(id).await.map_err(Into::into),
        Err(_) => template_from_file(archivist, raw, text).await,
    }
}

async fn template_from_file(
    archivist: &mut Archivist,
    raw: &RawMeta,
    path: &str,
) -> Result<TemplateMeta> {
    debug!("Picking template by path");
    assert_exists(path)?;

    // Insert the file into the table
    info!("Inserting new template {path}");
    let mut meta = TemplateMeta::new(path.to_string(), raw.pulsar_id)?;

    // If auto resolve dupes is off, we just insert
    if !archivist.config().behaviour.auto_resolve_duplicate_uploads {
        meta.id = archivist.insert(meta.clone()).await?;
        return Ok(meta);
    }

    // Otherwise, we check for pre-existing file
    let existing = archivist
        .find::<TemplateMeta>(&format!("checksum='{}'", meta.checksum))
        .await?;

    if let Some(tm) = existing {
        warn!(
            "Template with checksum {} already exists! Picking it instead.",
            tm.checksum,
        );
        Ok(tm)
    } else {
        meta.id = archivist.insert(meta.clone()).await?;
        Ok(meta)
    }
}
