use std::path::PathBuf;

use crate::Error;

#[derive(clap::Args)]
pub struct Args {
    /// PAA to convert
    paa: String,
    /// Where to save the file
    output: String,
}

/// Execute the convert command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(args: &Args) -> Result<(), Error> {
    let paa = PathBuf::from(&args.paa);
    let output = PathBuf::from(&args.output);
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
