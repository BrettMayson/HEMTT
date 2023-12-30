use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
};

use clap::{ArgMatches, Command};
use hemtt_pbo::ReadablePbo;

use crate::Error;

#[must_use]
pub fn cli() -> Command {
    Command::new("unpack")
        .about("Unpack a PBO")
        .arg(
            clap::Arg::new("pbo")
                .help("PBO file to unpack")
                .required(true),
        )
        .arg(
            clap::Arg::new("output")
                .help("Directory to unpack to")
                .required(true),
        )
}

/// Execute the unpack command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    let path = PathBuf::from(matches.get_one::<String>("pbo").expect("required"));
    let mut pbo = ReadablePbo::from(File::open(path)?)?;
    let output = PathBuf::from(matches.get_one::<String>("output").expect("required"));
    if output.exists() {
        error!("Output directory already exists");
        return Ok(());
    }
    std::fs::create_dir_all(&output)?;
    for (key, value) in pbo.properties() {
        debug!("{}: {}", key, value);
        if key == "prefix" {
            let mut file = File::create(output.join("$PBOPREFIX$"))?;
            file.write_all(value.as_bytes())?;
        } else {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(output.join("properties.txt"))?;
            file.write_all(format!("{key}={value}\n").as_bytes())?;
        }
    }
    for header in pbo.files() {
        let path = output.join(header.filename().replace('\\', "/"));
        std::fs::create_dir_all(path.parent().unwrap())?;
        let mut out = File::create(path)?;
        let mut file = pbo
            .file(header.filename())?
            .expect("file must exist if header exists");
        std::io::copy(&mut file, &mut out)?;
    }
    Ok(())
}
