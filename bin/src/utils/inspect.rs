use std::{
    fs::File,
    io::{Read, Seek},
    path::PathBuf,
};

use clap::{ArgMatches, Command};
use hemtt_signing::{BIPublicKey, BISign};

use crate::Error;

#[must_use]
pub fn cli() -> Command {
    Command::new("inspect")
        .about("Inspect an Arma file")
        .long_about("Provides information about supported files. Supported: pbo, bikey, bisign")
        .arg(
            clap::Arg::new("file")
                .help("File to inspect")
                .required(true),
        )
}

/// Execute the inspect command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    let path = PathBuf::from(matches.get_one::<String>("file").expect("required"));
    match path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
    {
        "paa" => {
            super::paa::inspect(File::open(&path)?)?;
        }
        "pbo" => {
            super::pbo::inspect(File::open(&path)?)?;
        }
        "bikey" => {
            bikey(File::open(&path)?, &path)?;
        }
        "bisign" => {
            bisign(File::open(&path)?, &path)?;
        }
        "cpp" | "hpp" => {
            super::config::inspect(&path)?;
        }
        _ => {
            let mut file = File::open(&path)?;
            let buf = &mut [0u8; 6];
            file.read_exact(buf)?;
            file.seek(std::io::SeekFrom::Start(0))?;
            // PBO
            if buf == b"\x00sreV\x00" {
                warn!("The file appears to be a PBO but does not have the .pbo extension.");
                super::pbo::inspect(file)?;
                return Ok(());
            }
            // PAA (skip first two bytes)
            if &buf[2..] == b"GGAT" {
                warn!("The file appears to be a PAA but does not have the .paa extension.");
                file.seek(std::io::SeekFrom::Start(0))?;
                super::paa::inspect(file)?;
                return Ok(());
            }
            // BiSign
            if BISign::read(&mut file).is_ok() {
                warn!("The file appears to be a BiSign but does not have the .bisign extension.");
                file.seek(std::io::SeekFrom::Start(0))?;
                bisign(file, &path)?;
                return Ok(());
            }
            file.seek(std::io::SeekFrom::Start(0))?;
            // BiPublicKey
            if BIPublicKey::read(&mut file).is_ok() {
                warn!(
                    "The file appears to be a BiPublicKey but does not have the .bikey extension."
                );
                file.seek(std::io::SeekFrom::Start(0))?;
                bikey(file, &path)?;
                return Ok(());
            }
            println!("Unsupported file type");
        }
    }
    Ok(())
}

/// Prints information about a [`BIPublicKey`] to stdout
///
/// # Errors
/// [`hemtt_signing::Error`] if the file is not a valid [`BIPublicKey`]
pub fn bikey(mut file: File, path: &PathBuf) -> Result<BIPublicKey, Error> {
    let publickey = BIPublicKey::read(&mut file)?;
    println!("Public Key: {path:?}");
    println!("  - Authority: {}", publickey.authority());
    println!("  - Length: {}", publickey.length());
    println!("  - Exponent: {}", publickey.exponent());
    println!("  - Modulus: {}", publickey.modulus_display(13));
    Ok(publickey)
}

/// Prints information about a [`BISign`] to stdout
///
/// # Errors
/// [`hemtt_signing::Error`] if the file is not a valid [`BISign`]
pub fn bisign(mut file: File, path: &PathBuf) -> Result<BISign, Error> {
    let signature = BISign::read(&mut file)?;
    println!("Signature: {path:?}");
    println!("  - Authority: {}", signature.authority());
    println!("  - Version: {}", signature.version());
    println!("  - Length: {}", signature.length());
    println!("  - Exponent: {}", signature.exponent());
    println!("  - Modulus: {}", signature.modulus_display(13));
    Ok(signature)
}
