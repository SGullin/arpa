use log::info;

use super::DiagnosticOut;
use crate::config::Config;
use crate::conveniences::assert_exists;
use crate::data_types::RawFileHeader;
use crate::external_tools::psrchive;
use crate::{ARPAError, Result};

/// Tries to create diagnostic plots.
///
/// # Errors
/// Fails if the fils is unreadable or the plotter fails.
pub fn run(config: &Config, file: &str) -> Result<DiagnosticOut> {
    info!("Creating composite plots for {file}...");

    let fname = file.rfind('/').map_or(file, |i| &file[i + 1..]);
    let tmp = format!("{}/tmp.png", config.paths.temp_dir);
    let tmpcmd = format!("{tmp}/PNG");
    let header = RawFileHeader::get(config, file)?;
    let info = format!(
        "above:l='{}\n\
        {}    {} ({})\n\
        Length={:.1} s    BW={:.1} MHz\n\
        N\\dbin\\u=$nbin    N\\dchan\\u=$nchan    N\\dsub\\u=$nsubint',\
        above:off=3.5",
        fname,
        header.telescope,
        header.receiver,
        header.backend,
        header.length,
        header.bw,
    );

    if header.sub_count * header.channel_count == 0 {
        return Err(ARPAError::DiagnosticPlotBadFile(file.to_string()));
    }
    match (header.sub_count > 1, header.channel_count > 1) {
        (true, true) => plot_all(config, file, &tmpcmd, &info)?,
        (true, false) => plot_no_freq(config, file, &tmpcmd, &info)?,
        (false, true) => plot_no_time(config, file, &tmpcmd, &info)?,
        (false, false) => plot_prof_only(config, file, &tmpcmd, &info)?,
    }

    assert_exists(&tmp)?;

    Ok(DiagnosticOut::Plot(tmp))
}

fn plot_all(
    config: &Config,
    path: &str,
    outcmd: &str,
    info: &str,
) -> Result<()> {
    let args = [
        "-O",
        "-j",
        "D",
        "-c",
        "above:c=,x:range=0:2",
        path,
        "-D",
        outcmd,
        "-p",
        "flux",
        "-c",
        ":0:x:view=0.575:0.95,",
        "y:view=0.7:0.9,",
        "subint=I,",
        "chan=I,",
        "pol=I,",
        "x:opt=BCTS,",
        "x:lab=,",
        "below:l=",
        "-p",
        "freq",
        "-c",
        ":1:x:view=0.075:0.45,",
        "y:view=0.15:0.7,",
        "subint=I,",
        "pol=I,",
        info,
        "cmap:map=plasma",
        "-p",
        "time",
        "-c",
        ":2:x:view=0.575:0.95,",
        "y:view=0.15:0.7,",
        "chan=I,",
        "pol=I,",
        "cmap:map=plasma",
    ];
    _ = psrchive(config, "psrplot", &args)?;

    Ok(())
}

fn plot_no_freq(
    config: &Config,
    path: &str,
    outcmd: &str,
    info: &str,
) -> Result<()> {
    let args = [
        "-O",
        "-j",
        "D",
        "-c",
        "above:c=,x:range=0:2",
        path,
        "-D",
        outcmd,
        "-p",
        "flux",
        "-c",
        ":0:x:view=0.075:0.95,",
        "y:view=0.5:0.7,",
        "subint=I,",
        "chan=I,",
        "pol=I,",
        "x:opt=BCTS,",
        "x:lab=,",
        "below:l=,",
        info,
        "-p",
        "time",
        "-c",
        ":1:x:view=0.075:0.95,",
        "y:view=0.15:0.5,",
        "chan=I,",
        "pol=I,",
        "cmap:map=plasma",
    ];
    _ = psrchive(config, "psrplot", &args)?;

    Ok(())
}

fn plot_no_time(
    config: &Config,
    path: &str,
    outcmd: &str,
    info: &str,
) -> Result<()> {
    let args = [
        "-O",
        "-j",
        "D",
        "-c",
        "above:c=,x:range=0:2",
        path,
        "-D",
        outcmd,
        "-p",
        "flux",
        "-c",
        &format!(
            ":0:x:view=0.075:0.95,\
            y:view=0.5:0.7,\
            subint=I,\
            chan=I,\
            pol=I,\
            x:opt=BCTS,\
            x:lab=,\
            below:l=,{info}",
        ),
        "-p",
        "freq",
        "-c",
        ":1:x:view=0.075:0.95,\
        y:view=0.15:0.5,\
        subint=I,\
        pol=I,\
        cmap:map=plasma",
    ];
    let res = psrchive(config, "psrplot", &args)?;
    info!("psrplot responded with '{res}'");

    Ok(())
}

fn plot_prof_only(
    config: &Config,
    path: &str,
    outcmd: &str,
    info: &str,
) -> Result<()> {
    let args = [
        "-O",
        "-j",
        "D",
        "-c",
        "above:c=,x:range=0:2",
        path,
        "-D",
        outcmd,
        "-p",
        "flux",
        "-c",
        ":0:x:view=0.075:0.95,",
        "y:view=0.15:0.7,",
        "subint=I,",
        "chan=I,",
        "pol=I,",
        "below:l=,",
        info,
    ];
    _ = psrchive(config, "psrplot", &args)?;

    Ok(())
}
