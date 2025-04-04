use std::path::PathBuf;

use crate::Error;

#[derive(clap::Args)]
pub struct JsonArgs {
    /// P3d to convert
    p3d: String,
    /// Where to save the file
    output: String,
}

/// Execute the convert command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(args: &JsonArgs) -> Result<(), Error> {
    let p3d = PathBuf::from(&args.p3d);
    let output = PathBuf::from(&args.output);
    if output.exists() {
        error!("Output file already exists");
        return Ok(());
    }
    let p3d = hemtt_p3d::P3D::read(&mut std::fs::File::open(p3d)?).expect("Failed to read P3D");
    serde_json::to_writer(std::fs::File::create(output)?, &p3d)?;
    Ok(())
}
