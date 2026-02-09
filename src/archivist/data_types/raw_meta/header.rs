use std::{fs::File, path::Path};

use crate::{
    ARPAError, Result, config::Config, conveniences::parse,
    external_tools::psrchive,
};
use psrutils::data_types::{J2000Dec, J2000Ra, Mjd};

#[derive(Debug)]
#[allow(missing_docs)]
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
    pub fn get(config: &Config, file_path: impl AsRef<Path>) -> Result<Self> {
        let filename = file_path
            .as_ref()
            .file_name()
            .map_or("unnamed".to_string(), |n| n.to_string_lossy().to_string());

        let keys = [
            "nbin", "nchan", "npol", "nsub", "type", "telescop", "name", "dec",
            "ra", "freq", "bw", "dm", "rm", "scale", "state", "length", "rcvr",
            "basis", "backend", "mjd",
        ];

        let values = Self::get_items(config, file_path, &keys)?;

        let mut i = 0;
        let header = Self {
            filename,
            bin_count: parse({
                i += 1;
                &values[i]
            })?,
            channel_count: parse({
                i += 1;
                &values[i]
            })?,
            polarization_count: parse({
                i += 1;
                &values[i]
            })?,
            sub_count: parse({
                i += 1;
                &values[i]
            })?,
            object_type: parse({
                i += 1;
                &values[i]
            })?,
            telescope: parse({
                i += 1;
                &values[i]
            })?,
            psr_name: parse({
                i += 1;
                &values[i]
            })?,
            dec: parse({
                i += 1;
                &values[i]
            })?,
            ra: parse({
                i += 1;
                &values[i]
            })?,
            frequency: parse({
                i += 1;
                &values[i]
            })?,
            bw: parse({
                i += 1;
                &values[i]
            })?,
            dm: parse({
                i += 1;
                &values[i]
            })?,
            rm: parse({
                i += 1;
                &values[i]
            })?,
            scale: parse({
                i += 1;
                &values[i]
            })?,
            state: parse({
                i += 1;
                &values[i]
            })?,
            length: parse({
                i += 1;
                &values[i]
            })?,
            receiver: parse({
                i += 1;
                &values[i]
            })?,
            basis: parse({
                i += 1;
                &values[i]
            })?,
            backend: parse({
                i += 1;
                &values[i]
            })?,
            date: parse({
                i += 1;
                &values[i]
            })?,
        };

        Ok(header)
    }

    /// Forms a directory structure suitable for this file.
    pub fn get_intended_directory(&self, config: &Config) -> String {
        format!(
            "{}/{}/{}/{}/{}",
            config.paths.diagnostics_dir,
            self.psr_name.to_uppercase(),
            self.telescope.to_lowercase(),
            self.receiver.to_lowercase(),
            self.backend.to_lowercase(),
        )
    }

    /// Calls `psrchive::vap` to get header items.
    ///
    /// # Errors
    /// Fails only if `psrchive` can't be called.
    pub fn get_items(
        config: &Config,
        path: impl AsRef<Path>,
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
                &path.as_ref().display().to_string(),
            ],
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
}

#[test]
fn bench_header() {
    // use crate::conveniences::{display_elapsed_time, progress_bar};
    let config = Config::default();

    let path = "/Users/samuelgullin/argos-software/arpa-dev/test-data/combine_B1929+10.ar";
    
    let old = RawFileHeader::get(&config, path).expect("Should read");
    let mut file = File::open(path).expect("No file");
    // let new = psrutils::timer_archive::header::Header::load(&mut file, true).expect("Should read");

    // println!("old\n{old:?}\n\nnew\n{new:?}");
    
    // let n = 32;
    // let n_inv = 1./n as f32;
    // let path = "/Users/samuelgullin/argos-software/arpa-dev/test-data/combine_B1929+10.ar";
    
    // let t0 = std::time::Instant::now();
    // for k in 0..n {
    //     _ = RawFileHeader::get(&config, path).expect("Should read");
    //     progress_bar("Running old get", k as f32 * n_inv, 32);
    // }
    // let dt_old = t0.elapsed();

    // let t0 = std::time::Instant::now();
    // for k in 0..n {
    //     let mut file = File::open(path).expect("No file");
    //     _ = psrutils::timer_archive::header::Header::load(&mut file, true).expect("Should read");
    //     progress_bar("Running new get", k as f32 * n_inv, 32);
    // }
    // let dt_new = t0.elapsed();

    // println!(
    //     "\n{n} iters: \n
    //     Old:\n\
    //     \ttotal: {}\n\taverage: {}\n\
    //     New:\n\
    //     \ttotal: {}\n\taverage: {}
    //     Improvement: {:.3} %",
    //     display_elapsed_time(dt_old),
    //     display_elapsed_time(dt_old / n as u32),
    //     display_elapsed_time(dt_new),
    //     display_elapsed_time(dt_new / n as u32),
    //     100.*((dt_old.as_secs_f64() - dt_new.as_secs_f64()) / dt_old.as_secs_f64()),
    // )
}
