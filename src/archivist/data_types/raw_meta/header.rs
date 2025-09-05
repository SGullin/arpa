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
    pub fn get(config: &Config, file_path: &str) -> Result<Self> {
        let index = file_path.rfind('/').map_or(0, |i| i + 1);
        let filename = file_path[index..].to_string();

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
            config.paths.rawfile_storage,
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
        path: &str,
        keys: &[&str],
    ) -> Result<Vec<String>> {
        let column_string = keys.join(",");
        let result = psrchive(config, "vap", &["-n", "-c", &column_string, path])?;
        
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
