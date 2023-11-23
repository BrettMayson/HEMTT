use std::path::PathBuf;

use clap::{ArgMatches, Command};

use crate::Error;

#[must_use]
pub fn cli() -> Command {
    Command::new("convert")
        .about("Convert a PAA to another format")
        .arg(clap::Arg::new("paa").help("PAA to convert").required(true))
        .arg(
            clap::Arg::new("output")
                .help("Where to save the file")
                .required(true),
        )
}

/// Execute the convert command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    let paa = PathBuf::from(matches.get_one::<String>("paa").expect("required"));
    let output = PathBuf::from(matches.get_one::<String>("output").expect("required"));
    if output.exists() {
        error!("Output file already exists");
        return Ok(());
    }
    let paa = hemtt_paa::Paa::read(std::fs::File::open(paa)?)?;
    if let Err(e) = paa.maps()[0].get_image().save(output) {
        error!("Failed to save PNG: {}", e);
    } else {
        info!("PAA converted");
    }
    Ok(())
}
