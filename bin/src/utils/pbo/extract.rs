use std::{fs::File, path::PathBuf};

use hemtt_pbo::ReadablePbo;

use crate::Error;

#[derive(clap::Args)]
/// Extract a single file from a PBO
pub struct PboExtractArgs {
    /// PBO file to extract from
    pbo: String,
    /// File to extract (use path relative to the PBO prefix)
    file: String,
    /// Where to save the extracted file, if not provided outputs to stdout
    output: Option<String>,
}

/// Execute the extract command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(args: &PboExtractArgs) -> Result<(), Error> {
    let path = PathBuf::from(&args.pbo);
    let mut pbo = ReadablePbo::from(File::open(path)?)?;
    let Some(mut file) = pbo.file(&args.file)? else {
        return Err(Error::NotFound(format!("{} in {}", args.file, args.pbo)));
    };
    let output = args.output.as_ref().map(PathBuf::from);
    if let Some(output) = output {
        if output.exists() {
            return Err(Error::OutputExists(output.display().to_string()));
        }
        let _ = fs_err::create_dir_all(
            output
                .parent()
                .expect("Output file has no parent directory"),
        );
        std::io::copy(&mut file, &mut File::create(output)?)?;
    } else {
        std::io::copy(&mut file, &mut std::io::stdout())?;
    }
    Ok(())
}
