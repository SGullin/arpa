//! The main TOA generation pipeline.
//! 
//! The function `cook` is probably what you're looking for -- it takes 
//! preparatory information and runs all the necessary functions. 
//! 
//! The `parse_input_` functions are helpers to parse text as either `id`s or 
//! paths and take the corresponding actions.

use std::{process::Command, time::Instant};

use log::{debug, error, warn};
use psrutils::{error::PsruError, timfile::TOAInfo as TOA};
use crate::{config::Config, conveniences::{assert_exists, compute_checksum, parse}, data_types::{DiagnosticPlot, ParMeta, ProcessInfo, PulsarMeta, RawFileHeader, RawMeta, TOAInfo, TemplateMeta}, diagnostics::run_diagnostic, external_tools::psrchive, ARPAError, Archivist};

mod arguments;
mod progress;
pub use arguments::{parse_input_raw, parse_input_ephemeride, parse_input_template};
pub use progress::Status;

/// Runs the toa-generation pipeline.
/// 
/// The `status_callback` is just for information on the progress of the 
/// pipeline, the minimal (informing) case would be `|s: Status| s.log`.
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
pub async fn cook<F: Fn(Status)+Send+Sync>(
    archivist: &mut Archivist,
    raw: RawMeta,
    ephemeride: Option<ParMeta>,
    template: TemplateMeta,
    diagnostics: bool,
    status_callback: F,
) -> Result<(), ARPAError> {
    let start = Instant::now();
    let pulsar_name = archivist.get::<PulsarMeta>(raw.pulsar_id).await?.alias;

    status_callback(Status::Starting {
        raw: (raw.file_path.clone(), raw.id),
        pulsar: (pulsar_name, raw.pulsar_id),
        ephemeride: ephemeride.clone().map(|e| (e.file_path, e.id)),
        template: template.id,
    });

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
        &status_callback,
    )?;

    let toa_meta = generate_toas(
        archivist.config(),
        &template,
        &new_path,
        diagnostics,
        &status_callback,
    )?;

    archivist.start_transaction().await?;

    let (process_id, toa_ids) = archive_toas(
        archivist, 
        &toa_meta,
        user_id,
        &raw,
        ephemeride.as_ref(),
        &template,
        &status_callback,
    ).await?;

    // > Create diagnostics & register plots ------------------------------
    if diagnostics {
        do_diagnostics(
            archivist,
            &new_path,
            process_id,
            toa_meta,
            toa_ids,
            &status_callback,
        ).await?;
    }
    archivist.commit_transaction().await?;

    status_callback(Status::Finished(start.elapsed()));
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

fn manipulate<F: Fn(Status)>(
    config: &Config,
    raw: &RawMeta,
    ephemeride: Option<&ParMeta>,
    adjust_path: &str,
    status_callback: F,
) -> Result<(), ARPAError> {
    // Make a new file for adjusting
    status_callback(Status::Copying (
        raw.file_path.clone(),
        adjust_path.to_string(),
    ));
    std::fs::copy(&raw.file_path, adjust_path)?;

    // > If parfile: reinstall ephemerides with pam -----------------------
    if let Some(par) = ephemeride {
        status_callback(Status::InstallingEphemeride);
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
        status_callback,
    )
}

fn manipulate_pam<F: Fn(Status)>(
    config: &Config,
    in_path: &str,
    n_subints: usize,
    n_channels: usize,
    set_n_bins: Option<usize>,
    set_t_subints: Option<usize>,
    status_callback: F,
) -> Result<(), ARPAError> {
    // We need to copy in->out. pam will just say "no filenames were specified"
    // if a file is specified, but doesn't exist. I guess it works in-place
    // std::fs::copy(in_path, out_path)?;
    status_callback(Status::Manipulating);

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
fn generate_toas<F: Fn(Status)>(
    config: &Config,
    template: &TemplateMeta,
    manip_path: &str,
    plot: bool,
    status_callback: F,
) -> Result<TOAMeta, ARPAError> {
    status_callback(Status::VerifyingTemplate);

    // Double check cheksum
    let checksum = compute_checksum(&template.file_path, true)?;
    if checksum != template.checksum.as_u128() {
        return Err(ARPAError::ChecksumFail(template.file_path.clone()));
    }

    status_callback(Status::GeneratingTOAs);
    let plot_file = format!("{}/toa_diag.png/PNG", config.paths.temp_dir);
    let mut args = vec![
        "-f",
        "tempo2",
        "-A",
        &config.behaviour.toa_fitting,
        "-s",
        &template.file_path,
        "-C",
        "gof length bw nbin nchan nsubint",
    ];

    if plot {
        args.append(&mut vec![
            "-t", // plot
            "-K", //plot device
            &plot_file,
        ]);
    }
    args.push(manip_path);

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

    status_callback(Status::GotTOAs(toas.len()));

    Ok(TOAMeta {
        toas,
        name: header[3].clone(),
        channels: parse(&header[1])?,
        subints: parse(&header[2])?,
        intmjd: parse(&header[4])?,
        secs,
    })
}

async fn archive_toas<F: Fn(Status)>(
    archivist: &mut Archivist, 
    toa_meta: &TOAMeta,
    user_id: i32, 
    raw: &RawMeta, 
    ephemeride: Option<&ParMeta>, 
    template: &TemplateMeta, 
    status_callback: F,
) -> Result<(i32, Vec<i32>), ARPAError> {
    status_callback(Status::LoggingProcess);
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
    status_callback(Status::ParsingTOAs);
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
    status_callback(Status::ArchivedTOAs(ids.len()));

    Ok((process_id, ids))
}

async fn do_diagnostics<F: Fn(Status)>(
    archivist: &mut Archivist,
    adjust_path: &str,
    process_id: i32,
    toa_meta: TOAMeta,
    toa_ids: Vec<i32>,
    status_callback: F,
) -> Result<(), ARPAError> {
    status_callback(Status::Diagnosing(
        archivist.config().behaviour.diagnostics.len()
    ));

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
    for diagnostic in diagnostics {
        let status = run_diagnostic(
            archivist,
            &diagnostic,
            process_id,
            adjust_path,
            &diag_path,
        ).await;

        status_callback(Status::FinishedDiagnostic{
            diagnostic,
            passed: status.is_ok(),
        });

        if let Err(err) = status {
            error!("{err}\n\nContinuing anyway...");
        }
    }

    // Move toa diagplot too
    let toa_diag_path = &format!(
        "{}/toa_diag.png", 
        archivist.config().paths.temp_dir,
    );

    if assert_exists(toa_diag_path).is_err() {
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

    status_callback(Status::ArchivedTOAPlots(toa_ids.len()));

    Ok(())
}
