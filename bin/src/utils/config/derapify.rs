use std::{io::Write as _, path::PathBuf};

use hemtt_config::rapify::Derapify;

use crate::Error;

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
pub struct DerapifyArgs {
    /// file to derapify
    pub(crate) file: String,
    /// output file
    pub(crate) output: Option<String>,
}

/// Derapify a config file
pub fn derapify(path: &PathBuf, output: Option<&String>) -> Result<(), Error> {
    let mut file = std::fs::File::open(path)?;
    let config = hemtt_config::Config::derapify(&mut file)?;
    let output = output.map_or_else(
        || {
            let mut path = path.clone();
            path.set_extension("cpp");
            path
        },
        PathBuf::from,
    );
    let mut output = std::fs::File::create(output)?;
    output.write_all(config.to_string().as_bytes())?;
    output.flush()?;
    Ok(())
}
