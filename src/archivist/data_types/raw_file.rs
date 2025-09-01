use std::{fs::File, os::unix::fs::MetadataExt};

use log::{debug, info, warn};
use psrutils::data_types::{J2000Dec, J2000Ra, Mjd};
use sqlx::types::uuid;

use crate::{
    config::VolatileConfig, 
    conveniences::{assert_exists, compute_checksum, parse}, 
    external_tools::psrchive, 
    ARPAError, Archivist, Result, TableItem,
};
use super::{pulsar_meta::PulsarMeta, raw_meta::RawMeta, telescope::ObsSystem};

#[derive(Debug)]
pub struct RawFileHeader {
    // Not the whole path, mind you.
    pub filename: String,

    pub bin_count: u32,
    pub channel_count: u32,
    pub polarization_count: u8,
    pub sub_count: u32,
    pub object_type: String,
    pub telescope: String,
    
    pub psr_name: String, 
    pub ra: J2000Ra,
    pub dec: J2000Dec,
    pub frequency: f32,
    pub bw: f32,
    pub dm: f32,
    pub rm: f32,

    pub scale: String,
    pub state: String,
    pub length: f32,

    pub receiver: String,
    pub basis: String, 
    pub backend: String,
    pub date: Mjd,
}
impl RawFileHeader {
    /// Calls `psrchive::vap` to get the header of a raw file.
    /// # Errors
    /// This depends on a call to `psrchive` that may fail for various reasons,
    /// but there are also many `parse` calls that fail.
    pub fn get(config: &VolatileConfig, file_path: &str) -> Result<Self> {
        let index = file_path
            .rfind('/')
            .map_or(0, |i| i + 1);
        let filename = file_path[index..].to_string();

        let keys = [
            "nbin", "nchan", "npol", "nsub", "type", "telescop",
            "name", "dec", "ra", "freq", "bw", "dm", "rm",
            "scale", "state", "length",
            "rcvr", "basis", "backend", "mjd",
        ];

        let values = get_header_items(config, file_path, &keys)?;
        
        let mut i = 0;
        let header = Self {
            filename,
            bin_count:          parse({ i += 1; &values[i] })?,
            channel_count:      parse({ i += 1; &values[i] })?,
            polarization_count: parse({ i += 1; &values[i] })?,
            sub_count:          parse({ i += 1; &values[i] })?,
            object_type:        parse({ i += 1; &values[i] })?,
            telescope:          parse({ i += 1; &values[i] })?,
            psr_name:           parse({ i += 1; &values[i] })?,
            dec:                parse({ i += 1; &values[i] })?,
            ra:                 parse({ i += 1; &values[i] })?,
            frequency:          parse({ i += 1; &values[i] })?,
            bw:                 parse({ i += 1; &values[i] })?,
            dm:                 parse({ i += 1; &values[i] })?,
            rm:                 parse({ i += 1; &values[i] })?,
            scale:              parse({ i += 1; &values[i] })?,
            state:              parse({ i += 1; &values[i] })?,
            length:             parse({ i += 1; &values[i] })?,
            receiver:           parse({ i += 1; &values[i] })?,
            basis:              parse({ i += 1; &values[i] })?,
            backend:            parse({ i += 1; &values[i] })?,
            date:               parse({ i += 1; &values[i] })?,
        };

        Ok(header)
    }

    /// Forms a directory structure suitable for this file.
    pub fn get_intended_directory(&self, config: &VolatileConfig) -> String {
        format!(
            "{}/{}/{}/{}/{}",
            config.paths.rawfile_storage,
            self.psr_name.to_uppercase(),
            self.telescope.to_lowercase(),
            self.receiver.to_lowercase(),
            self.backend.to_lowercase(),
        )
    }
}

fn get_header_items(
    config: &VolatileConfig,
    path: &str,
    keys: &[&str],
) -> Result<Vec<String>> {
    let column_string = keys.join(",");
    let result = psrchive(
        config, 
        "vap",
        &[
            "-n", 
            "-c",
            &column_string,
            path,
        ]
    )?;

    // We get a string of values
    let values = result
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();

    if values.len() != keys.len() + 1 {
        return Err(ARPAError::VapKeyCount(keys.len() + 1, values.len()));
    }

    Ok(values)
}

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
) -> Result<RawMeta> {
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
    ).await?
    .ok_or(ARPAError::CantFind(format!(
        "Obssystem in registry... \n\
        (Telescope: {}, frontend: {}, backend: {}).",
        &header.telescope,
        &header.receiver,
        &header.backend,
    )))?;
    let observer_id = obs_system.id;
    debug!("Found observation system.");

    // Get pulsar name
    let res = archivist.find::<PulsarMeta>(&format!(
        "j_name='{}'",
        &header.psr_name,
    )).await?;

    let pulsar_id = if let Some(r) = res { r.id() } else {
        debug!("Unrecognised pulsar.");

        if !archivist.config().behaviour.auto_add_pulsars {
            return Err(ARPAError::CantFind(format!(
                "Pulsar with name '{}', and we're not set to auto-add.",
                &header.psr_name,
            )))
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
    }
    else {
        info!("Currently set to not archive raw files...");
        compute_checksum(&file_path, true)?
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

/// Puts the file in a good spot. To speed up copying and checksum calculations
/// some thigns are done concurrently.
/// 
/// # Errors 
/// There are only two cases: 
///  1) the io calls fail; and
///  2) the threads can't be joined.
pub fn archive_file(
    config: &VolatileConfig,
    source: &mut String, 
    directory: &str,
    name: &str,
) -> Result<u128> {
    let path = format!("{directory}/{name}");

    if source == &path {
        warn!("File is already where it should be ({source}).");
        return Ok(0);
    }

    std::fs::create_dir_all(directory)?;
    if std::fs::exists(&path)? {
        return check_file_equality(source, path);
    }

    // Both of these tasks can take some time, so they might as well run 
    // concurrently. Even though they access the same file, they are both only
    // reading it. Should be ok.
    let sc = source.clone();
    let dc = path.clone();
    let copy_handle = std::thread::spawn(|| std::fs::copy(sc, dc));
    let sc = source.clone();
    let src_checksum_handle = std::thread::spawn(|| 
        compute_checksum(sc, true)
    );

    // If it turns out the copy is faster than the src checksum, we can start
    // the dst checksum early. If not, we haven't lost anyhting here.
    let dst_size = copy_handle
        .join()
        .map_err(|err| ARPAError::JoinThread(format!("{err:?}")))??;

    let dc = path.clone();
    let dst_checksum_handle = std::thread::spawn(|| 
        compute_checksum(dc, false)
    );

    let src_size = File::open(&source)?
        .metadata()?
        .size();

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
        ))
    }

    if config.behaviour.move_rawfiles {
        std::fs::remove_file(&source)?;
        info!("Successfully moved {source} to {path}");
    }
    else {
        info!("Successfully copied {source} to {path}");
    }

    *source = path;

    Ok(src_checksum)
}

fn check_file_equality(source: &str, path: String) -> Result<u128> {
    warn!("File already exists: '{path}'! Will not overwrite.");
    let src_size = File::open(source)?
        .metadata()?
        .size();
    let dst_size = File::open(&path)?
        .metadata()?
        .size();

    if src_size != dst_size {
        warn!("Old file is {dst_size} bytes and new is {src_size} bytes."); 
        return Ok(0);
    }

    let sc = source.to_string();
    let src_checksum_handle = std::thread::spawn(|| 
        compute_checksum(sc, true)
    );
    let dst_checksum_handle = std::thread::spawn(|| 
        compute_checksum(path, false)
    );

    let src_checksum = src_checksum_handle
        .join()
        .map_err(|err| ARPAError::JoinThread(format!("{err:?}")))??;
    let dst_checksum = dst_checksum_handle
        .join()
        .map_err(|err| ARPAError::JoinThread(format!("{err:?}")))??;

    if src_checksum == dst_checksum {
        info!("The files seem to be the same, so you can probably remove \
            the source.");
    }
    
    Ok(src_checksum)
}
