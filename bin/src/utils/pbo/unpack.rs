use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
};

use hemtt_pbo::ReadablePbo;

use crate::Error;

#[derive(clap::Args)]
pub struct PboUnpackArgs {
    /// PBO file to unpack
    pbo: String,
    /// Directory to unpack to
    output: String,
}

/// Execute the unpack command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(args: &PboUnpackArgs) -> Result<(), Error> {
    let path = PathBuf::from(&args.pbo);
    let mut pbo = ReadablePbo::from(File::open(path)?)?;
    let output = PathBuf::from(&args.output);
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
        std::fs::create_dir_all(path.parent().expect("must have parent, just joined"))?;
        let mut out = File::create(path)?;
        let mut file = pbo
            .file(header.filename())?
            .expect("file must exist if header exists");
        std::io::copy(&mut file, &mut out)?;
    }
    Ok(())
}
