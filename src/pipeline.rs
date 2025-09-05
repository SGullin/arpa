//! The main TOA generation pipeline.
//! 
//! The function `cook` is probably what you're looking for -- it takes 
//! preparatory information and runs all the necessary functions. 
//! 
//! The `parse_input_` functions are helpers to parse text as either `id`s or 
//! paths and take the corresponding actions.

use std::{process::Command, time::Instant};

use log::{debug, error, info, warn};
use psrutils::{error::PsruError, timfile::TOAInfo as TOA};
use crate::{config::Config, conveniences::{assert_exists, compute_checksum, display_elapsed_time, parse}, data_types::{DiagnosticPlot, ParMeta, ProcessInfo, PulsarMeta, RawFileHeader, RawMeta, TOAInfo, TemplateMeta}, diagnostics::run_diagnostic, external_tools::psrchive, ARPAError, Archivist};

mod arguments;
pub use arguments::{parse_input_raw, parse_input_ephemeride, parse_input_template};

/// Runs the toa-generation pipeline.
/// 
/// # Notes
/// While it is possible to create the different `meta`s without uploading them
/// to the database, doing so might cause errors down the line. Things like 
/// [`ProcessInfo`] are set to use `sql` references, so `sqlx` will complain if
/// they do not exists. This will cause the whole pipeline to fail and any 
/// previous actions to be rolled back.
/// 
/// # Errors
/// There are many ways this can fail, e.g.:
///  - the `archivist` fails;
///  - a path is not reachable;
///  - the database information is out of date.
/// 
/// It should not fail because of bad luck though :)
pub async fn cook(
    archivist: &mut Archivist,
    raw: RawMeta,
    ephemeride: Option<ParMeta>,
    template: TemplateMeta,
) -> Result<(), ARPAError> {
    let start = Instant::now();
    let pulsar_name = archivist.get::<PulsarMeta>(raw.pulsar_id).await?.alias;

    info!(
        "Cooking with the following:\
        \n * Raw file:   {}\
        \n               id = {}\
        \n * Pulsar:     {} \
        \n               id = {}\
        \n * Ephemeride: {}\
        \n * Template:   id = {}\n",
        raw.file_path, raw.id,
        pulsar_name, raw.pulsar_id,
        ephemeride.as_ref().map_or_else(
            || "(None)\n".into(), 
            |e| format!("{}\n               id = {}", e.file_path, e.id),
        ),
        template.id,
    );

    let user_id = 0;
    let new_path = format!(
        "{}/working.ar", 
        archivist.config().paths.temp_dir
    );

    manipulate(
        archivist.config(),
        &raw,
        ephemeride.as_ref(),
        &new_path,
    )?;

    let toa_meta = generate_toas(
        archivist.config(),
        &template,
        &new_path,
    )?;

    archivist.start_transaction().await?;

    let (process_id, toa_ids) = archive_toas(
        archivist, 
        &toa_meta,
        user_id,
        &raw,
        ephemeride.as_ref(),
        &template,
    ).await?;

    // > Create diagnostics & register plots ------------------------------
    do_diagnostics(
        archivist,
        &new_path,
        process_id,
        toa_meta,
        toa_ids,
    ).await?;
    
    archivist.commit_transaction().await?;

    info!("Finished in {}", display_elapsed_time(start));
    Ok(())
}

struct TOAMeta {
    toas: Vec<String>,
    name: String,
    channels: i16,
    subints: i16,
    intmjd: u16,
    secs: u32,
}

fn manipulate(
    config: &Config,
    raw: &RawMeta,
    ephemeride: Option<&ParMeta>,
    adjust_path: &str,
) -> Result<(), ARPAError> {
    // Make a new file for adjusting
    debug!("Copying from {} to {}", raw.file_path, adjust_path);
    std::fs::copy(&raw.file_path, adjust_path)?;

    // > If parfile: reinstall ephemerides with pam -----------------------
    if let Some(par) = ephemeride {
        // Threre's no output...
        _ = psrchive(
            config,
            "pam",
            &["-m", "-E", &par.file_path, "--update_dm", adjust_path],
        )?;
    }

    // Make a new file for manipulating
    manipulate_pam(
        config, 
        adjust_path, 
        1,    4, 
        None, None,
    )
}

fn manipulate_pam(
    config: &Config,
    in_path: &str,
    n_subints: usize,
    n_channels: usize,
    set_n_bins: Option<usize>,
    set_t_subints: Option<usize>,
) -> Result<(), ARPAError> {
    debug!("Manipulating file...");

    // We need to copy in->out. pam will just say "no filenames were specified"
    // if a file is specified, but doesn't exist. I guess it works in-place
    // std::fs::copy(in_path, out_path)?;

    let mut args = vec![
        "-m".to_string(),
        "-p".to_string(),
        // "ar2".to_string(), // what does this do..?
        "--setnchn".to_string(),
        n_channels.to_string(),
    ];
    if let Some(n) = set_t_subints {
        args.append(&mut vec!["--settsub".to_string(), n.to_string()]);
    } else {
        args.append(&mut vec!["--setnsub".to_string(), n_subints.to_string()]);
    }
    if let Some(n) = set_n_bins {
        args.append(&mut vec!["--setnbin".to_string(), n.to_string()]);
    }
    args.push(in_path.to_string());
    
    psrchive(config, "pam", &args)?;

    Ok(())
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn generate_toas(
    config: &Config,
    template: &TemplateMeta,
    manip_path: &str,
) -> Result<TOAMeta, ARPAError> {
    info!("Generating TOAs...");

    // Double check cheksum
    let checksum = compute_checksum(&template.file_path, true)?;
    if checksum != template.checksum.as_u128() {
        return Err(ARPAError::ChecksumFail(template.file_path.clone()));
    }

    let args = [
        "-f",
        "tempo2",
        "-A",
        &config.behaviour.toa_fitting,
        "-s",
        &template.file_path,
        "-C",
        "gof length bw nbin nchan nsubint",
        // "-t", // plot
        // "-K", //plot device
        // &format!("{}/toa_diag.png/PNG", config.paths.temp_dir),
        manip_path,
    ];

    let result = psrchive(config, "pat", &args)?;
    if !result.starts_with("FORMAT 1") {
        return Err(ARPAError::TOAExpectedFormat(result));
    }
    debug!("Got toas!");

    // Now pat has modified the manip file, so we can read from it
    let header = RawFileHeader::get_items(
        config,
        manip_path,
        &["nchan", "nsub", "name", "intmjd", "fracmjd"],
    )?;
    debug!("Got header!");

    let secs = (parse::<f32>(&header[4])? * 24. * 3600.).round() as u32;
    let mut toas: Vec<String> = result.lines().map(ToString::to_string).collect();
    toas.remove(0); // The format specifier

    Ok(TOAMeta {
        toas,
        name: header[3].clone(),
        channels: parse(&header[1])?,
        subints: parse(&header[2])?,
        intmjd: parse(&header[4])?,
        secs,
    })
}

async fn archive_toas(
    archivist: &mut Archivist, 
    toa_meta: &TOAMeta,
    user_id: i32, 
    raw: &RawMeta, 
    ephemeride: Option<&ParMeta>, 
    template: &TemplateMeta, 
) -> Result<(i32, Vec<i32>), ARPAError> {
    debug!("Inserting process info...");
    let meta = ProcessInfo::new(
        user_id,
        raw,
        ephemeride,
        template,
        toa_meta.channels,
        toa_meta.subints,
        &archivist.config().behaviour.toa_fitting,
    );
    let process_id = archivist.insert(meta).await?;

    // > Parse the output of psrchive::pat and insert toas ----------------
    debug!("Parsing toas...");
    let toas = toa_meta
        .toas
        .iter()
        .map(|l| TOA::from_line_tempo2(l)
            .map(|toa| TOAInfo::extract(
                &toa,
                raw.pulsar_id,
                raw.observer_id,
                process_id,
                template.id,
                raw.id,
            )))
        .collect::<Result<Vec<_>, PsruError>>()?;

    let mut ids = Vec::with_capacity(toas.len());
    for toa in toas {
        ids.push(archivist.insert(toa).await?);
    }
    info!(
        "Uploaded {} TOA{}!",
        ids.len(),
        if ids.len() > 1 { "s" } else { "" }
    );

    Ok((process_id, ids))
}

async fn do_diagnostics(
    archivist: &mut Archivist,
    adjust_path: &str,
    process_id: i32,
    toa_meta: TOAMeta,
    toa_ids: Vec<i32>,
) -> Result<(), ARPAError> {
    info!("Creating diagnostics...");
    let header = RawFileHeader::get(archivist.config(), adjust_path)?;
    let dir = header.get_intended_directory(archivist.config());
    // We put the diagnostic together with the rawfile
    let diag_path = format!("{dir}/process{process_id}");
    // And add a symlink at the top
    let crossref_path = format!(
        "{}/process{}", 
        archivist.config().paths.diagnostics_dir, 
        process_id,
    );
    _ = Command::new("ln")
        .args(["-s", &diag_path, &crossref_path])
        .output()?;

    let diagnostics = archivist.config().behaviour.diagnostics.clone();
    for diagnostic in &diagnostics {
        if let Err(err) = run_diagnostic(
            archivist,
            diagnostic,
            process_id,
            adjust_path,
            &diag_path,
        ).await {
            error!("{err}\n\nContinuing anyway...");
        }
    }

    // Move toa diagplot too
    let toa_diag_path = &format!(
        "{}/toa_diag.png", 
        archivist.config().paths.temp_dir,
    );

    if let Err(_) = assert_exists(&toa_diag_path) {
        warn!("TOA diagnostic plot not found.");
        return Ok(())
    }

    let base_path = format!(
        "{}/{}_{:05}_{:05}",
        diag_path,
        toa_meta.name, toa_meta.intmjd, toa_meta.secs,
    );
    for (i, id) in toa_ids.iter().enumerate() {
        let dst = format!("{base_path}.TOA{id}.png");
        let src = if i == 0 {
            toa_diag_path.clone()
        } else {
            format!("{}_{}", toa_diag_path, i + 1)
        };

        std::fs::rename(&src, &dst)?;
        let meta = DiagnosticPlot {
            id: 0,
            process: process_id,
            diagnostic: String::from("Prof-Temp Residuals"),
            filepath: dst,
        };
        archivist.insert(meta).await?;
    }

    info!("Inserted {} plots!", toa_ids.len());

    Ok(())
}
