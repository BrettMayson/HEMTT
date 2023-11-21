use std::{fs::File, path::PathBuf};

use clap::{ArgMatches, Command};
use hemtt_pbo::ReadablePbo;

use crate::Error;

#[must_use]
pub fn cli() -> Command {
    Command::new("extract")
        .about("Extract a file from a PBO")
        .arg(
            clap::Arg::new("pbo")
                .help("PBO file to extract from")
                .required(true),
        )
        .arg(
            clap::Arg::new("file")
                .help("File to extract")
                .required(true),
        )
        .arg(clap::Arg::new("output").help("Where to save the extracted file"))
}

/// Execute the extract command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    let path = PathBuf::from(matches.get_one::<String>("pbo").expect("required"));
    let mut pbo = ReadablePbo::from(File::open(path)?)?;
    let file = matches.get_one::<String>("file").expect("required");
    let Some(mut file) = pbo.file(file)? else {
        error!("File `{file}` not found in PBO");
        return Ok(());
    };
    let output = matches.get_one::<String>("output").map(PathBuf::from);
    if let Some(output) = output {
        std::io::copy(&mut file, &mut File::create(output)?)?;
    } else {
        std::io::copy(&mut file, &mut std::io::stdout())?;
    }
    Ok(())
}
