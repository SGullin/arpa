//! Metadata of a stored rawfile.

use crate::{
    ARPAError, Archivist, Result,
    archivist::table::TableItem,
    config::Config,
    conveniences::{assert_exists, check_file_equality, compute_checksum},
    data_types::{ObsSystem, PulsarMeta},
};
use item_macro::TableItem;
use log::{debug, info, warn};
use sqlx::{prelude::FromRow, types::uuid};
use std::fs::File;
use std::os::unix::fs::MetadataExt;

mod header;
pub use header::RawFileHeader;

#[derive(FromRow, Clone, TableItem)]
#[table(RawMetas)]
/// Metadata of a stored raw file.
pub struct RawMeta {
    /// Mandatory id.
    #[derived]
    pub id: i32,

    /// Path to file.
    #[unique]
    pub file_path: String,
    /// 128 bit checksum.
    #[unique]
    pub checksum: uuid::Uuid,

    /// ID of pulsar it refers to.
    pub pulsar_id: i32,
    /// ID of observation unit that produced file.
    pub observer_id: i32,
}

impl RawMeta {
    /// Prepares a raw file and returns its meta.
    /// # Errors
    /// Fails if
    ///  - the specified path does not exist;
    ///  - the header can't be read;
    ///  - the observation system is missing;
    ///  - the `archivist` encounters an error.
    pub async fn prepare_raw_meta(
        archivist: &mut Archivist,
        path: &str,
    ) -> Result<Self> {
        assert_exists(path)?;

        // Check that the file is ok
        let header = RawFileHeader::get(archivist.config(), path)?;
        debug!("Got raw header info.");

        // TODO also get user id and put it into meta

        // Get telescope name
        let obs_system = ObsSystem::find(
            archivist,
            &header.telescope.to_lowercase(),
            &header.receiver.to_lowercase(),
            &header.backend.to_lowercase(),
        )
        .await?
        .ok_or(ARPAError::CantFind(format!(
            "Obssystem in registry... \n\
            (Telescope: {}, frontend: {}, backend: {}).",
            &header.telescope, &header.receiver, &header.backend,
        )))?;
        let observer_id = obs_system.id;
        debug!("Found observation system.");

        // Get pulsar name
        let res = archivist
            .find::<PulsarMeta>(&format!("j_name='{}'", &header.psr_name,))
            .await?;

        let pulsar_id = if let Some(r) = res {
            r.id()
        } else {
            debug!("Unrecognised pulsar.");

            if !archivist.config().behaviour.auto_add_pulsars {
                return Err(ARPAError::CantFind(format!(
                    "Pulsar with name '{}', and we're not set to auto-add.",
                    &header.psr_name,
                )));
            }

            info!("Adding pulsar '{}'", &header.psr_name);
            let mut meta = PulsarMeta {
                id: 0,
                alias: header.psr_name.to_string(),
                j_name: None,
                b_name: None,
                j2000_ra: None,
                j2000_dec: None,
                master_parfile_id: None,
            };
            meta.verify()?;
            archivist.insert(meta).await?
        };

        // Move the file into a better spot in the archive
        let mut file_path = path.to_string();
        let checksum = if archivist.config().behaviour.archive_rawfiles {
            info!("Archiving file...");
            let directory = header.get_intended_directory(archivist.config());
            archive_file(
                archivist.config(),
                &mut file_path,
                &directory,
                &header.filename,
            )?
        } else {
            info!("Currently set to not archive raw files...");
            compute_checksum(
                &file_path,
                archivist.config().behaviour.checksum_block_size,
                true,
            )?
        };

        let checksum = uuid::Uuid::from_u128(checksum);

        Ok(RawMeta {
            id: 0,
            file_path,
            checksum,
            pulsar_id,
            observer_id,
        })
    }
}

/// Puts the file in a good spot. To speed up copying and checksum calculations
/// some thigns are done concurrently.
///
/// # Errors
/// There are only two cases:
///  1) the io calls fail; and
///  2) the threads can't be joined.
pub fn archive_file(
    config: &Config,
    source: &mut String,
    directory: &str,
    name: &str,
) -> Result<u128> {
    let path = format!("{directory}/{name}");

    if source == &path {
        warn!("File is already where it should be ({source}).");
        return Ok(0);
    }
    let block_size = config.behaviour.checksum_block_size;

    std::fs::create_dir_all(directory)?;
    if std::fs::exists(&path)? {
        return check_file_equality(source, path, block_size);
    }

    // Both of these tasks can take some time, so they might as well run
    // concurrently. Even though they access the same file, they are both only
    // reading it. Should be ok.
    let sc = source.clone();
    let dc = path.clone();
    let copy_handle = std::thread::spawn(|| std::fs::copy(sc, dc));
    let sc = source.clone();
    let src_checksum_handle =
        std::thread::spawn(move || compute_checksum(sc, block_size, true));

    // If it turns out the copy is faster than the src checksum, we can start
    // the dst checksum early. If not, we haven't lost anyhting here.
    let dst_size = copy_handle
        .join()
        .map_err(|err| ARPAError::JoinThread(format!("{err:?}")))??;

    let dc = path.clone();
    let dst_checksum_handle =
        std::thread::spawn(move || compute_checksum(dc, block_size, false));

    let src_size = File::open(&source)?.metadata()?.size();

    let src_checksum = src_checksum_handle
        .join()
        .map_err(|err| ARPAError::JoinThread(format!("{err:?}")))??;
    let dst_checksum = dst_checksum_handle
        .join()
        .map_err(|err| ARPAError::JoinThread(format!("{err:?}")))??;

    if src_checksum != dst_checksum || src_size != dst_size {
        return Err(ARPAError::FileCopy(
            src_checksum,
            dst_checksum,
            src_size,
            dst_size,
        ));
    }

    if config.behaviour.move_rawfiles {
        std::fs::remove_file(&source)?;
        info!("Successfully moved {source} to {path}");
    } else {
        info!("Successfully copied {source} to {path}");
    }

    *source = path;

    Ok(src_checksum)
}
