use std::{
    any::type_name, 
    fs::File, 
    io::{stdout, BufReader, Read, Write}, 
    os::unix::fs::MetadataExt, 
    path::Path, 
    str::FromStr, 
    time::Instant
};

use md5::Digest;

use crate::{Result, config, ARPAError};

/// Checks a path for a file.
/// # Errors
/// The file does not exist, or there is an io problem.
pub fn assert_exists(path: &str) -> Result<()> {
    match std::fs::exists(path) {
        Ok(true) => Ok(()),
        Ok(false) => Err(ARPAError::MissingFileOrDirectory(path.into())),
        Err(err) => Err(ARPAError::IOFault(err)),
    }
}

#[allow(clippy::cast_possible_truncation, 
    clippy::cast_sign_loss,
    clippy::cast_precision_loss)]
/// Prints a progress bar with a prepended message.
pub fn progress_bar(
    message: &str,
    progress: f32,
    size: usize,
) {
    let counts = (size as f32 * progress).round() as usize;
    let start = counts > 0;
    let end = counts == size;
    let full = counts.saturating_sub(2);
    let empty = size - 2 - full;
    print!(
        "\r{} {}{}{}{}",
        message,
        if start { "\u{ee03}" } else { "\u{ee00}" },
        "\u{ee04}".repeat(full),
        "\u{ee01}".repeat(empty),
        if end { "\u{ee05}" } else { "\u{ee02}" },
    );

    // It's just a progress bar, if we run into errors here I think we can
    // live without them
    _ =  stdout().flush();
}

/// Forms a string from the elapsed time, mainly to get easily readable times.
pub fn display_elapsed_time(start: std::time::Instant) -> String {
    let dur = start.elapsed();
    let micros = dur.as_micros();
    
    if micros < 1000 {
        return format!("{micros} μs");
    }

    let millis = micros / 1000;
    if millis < 1000 {
        return format!("{millis} ms");
    }
    
    let mut seconds = micros / 1_000_000;
    if seconds < 60 {
        return format!("{seconds} s");
    }

    let minutes = seconds / 60;
    seconds -= 60 * minutes;
    
    format!("{minutes} m {seconds} s")
}

#[allow(clippy::missing_errors_doc)]
/// Wrapper for `parse` to get a nice error.
pub fn parse<T>(text: &str) -> Result<T> 
where T: FromStr + std::fmt::Debug {
    text
    .parse::<T>()
    .map_err(|_| ARPAError::ParseFailed(text.to_string(), type_name::<T>()))
}

/// Forms a string with comma separated digit triples. 
/// 
/// E.g. 
/// ```
/// # use arpa::conveniences::comma_separate;
/// assert_eq!(comma_separate(123u64),    "123");
/// assert_eq!(comma_separate(1234u64),   "1,234");
/// assert_eq!(comma_separate(12345u64),  "12,345");
/// assert_eq!(comma_separate(123456u64), "123,456");
/// ```
pub fn comma_separate<T>(value: &T) -> String where T: Into<u64> + ToString {
    value
        .to_string()
        .chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(3)
        .enumerate()
        .map(|(i, digits)| digits
            .iter()
            .fold(
                String::from(
                    if i == 0 { "" } else { "," }
                ), 
                |a, d| a + &d.to_string()
            ).chars()
            .rev()
            .collect::<String>()
        ).rev()
        .fold(String::new(), |a, d| a + &d)
}

#[allow(clippy::cast_precision_loss)]
/// Computes the MD5 checksum of a file.
/// 
/// # Errors
/// Possible io failure.
pub fn compute_checksum(
    path: impl AsRef<Path>, 
    verbose: bool,
) -> std::io::Result<u128> {
    let t0 = Instant::now();
    
    let file = File::open(path)?;
    let size = file.metadata()?.size();
    let mut reader = BufReader::new(file);

    let mut hasher = md5::Md5::new();

    // To show progress
    let len = (
        size as f32 / config::stable::CHECKSUM_BLOCK_SIZE as f32
    ).max(1.0);
    let mut read = 0.0;

    let mut buffer = vec![0u8; config::stable::CHECKSUM_BLOCK_SIZE ];
    while reader.read(&mut buffer)? > 0 {
        hasher.update(&buffer);

        read += 1.0;
        if verbose {
            progress_bar(
                "Computing MD5 checksum...", 
                read / len, 
                32,
            );
        }
    }

    if verbose {
        println!(
            "\nDone in {:<32}",
            display_elapsed_time(t0),
        );
}

    let hash = hasher
        .finalize()
        .iter()
        .fold(0, |a, b| (a << 8) + u128::from(*b));

    Ok(hash)
}
