use log::{debug, info, warn};

use crate::{
    ARPAError, Archivist,
    conveniences::assert_exists,
    data_types::{ParMeta, RawMeta, TemplateMeta},
};

/// Parses `text` to load a `RawMeta`. This will try two things:
///  1) parsing as an `i32`: if successful, it will look for an existing entry
///     with that id; or
///  2) treating as a path: if a file is found, it will try to upload it and
///     then use it.
///
/// # Errors
/// Any error will come from either the `archivist` failing or a file not being
/// ok.
pub async fn parse_input_raw(
    archivist: &mut Archivist,
    text: &str,
) -> Result<RawMeta, ARPAError> {
    match text.parse() {
        Ok(id) => archivist.get(id).await.map_err(Into::into),
        Err(_) => raw_from_file(archivist, text).await,
    }
}

async fn raw_from_file(
    archivist: &mut Archivist,
    path: &str,
) -> Result<RawMeta, ARPAError> {
    debug!("Picking raw file by path");

    // Insert the file into the table
    let mut raw = RawMeta::parse(archivist, path).await?;
    info!("Inserting raw file {path}");

    raw.id = archivist.insert(raw.clone()).await?;

    Ok(raw)
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
) -> Result<ParMeta, ARPAError> {
    match text.parse() {
        Ok(id) => archivist.get(id).await.map_err(Into::into),
        Err(_) => ephermeride_from_file(archivist, raw, text).await,
    }
}

async fn ephermeride_from_file(
    archivist: &mut Archivist,
    raw: &RawMeta,
    path: &str,
) -> Result<ParMeta, ARPAError> {
    debug!("Parsing ephemeride path");
    assert_exists(path)?;

    // Insert the file into the table
    let mut meta = ParMeta::new(path.to_string(), raw.pulsar_id)?;
    info!("Inserting ephemeride {path}");

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
) -> Result<TemplateMeta, ARPAError> {
    match text.parse() {
        Ok(id) => archivist.get(id).await.map_err(Into::into),
        Err(_) => template_from_file(archivist, raw, text).await,
    }
}

async fn template_from_file(
    archivist: &mut Archivist,
    raw: &RawMeta,
    path: &str,
) -> Result<TemplateMeta, ARPAError> {
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
