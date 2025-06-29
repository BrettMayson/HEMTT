use std::{io::Write as _, path::PathBuf};
use serde_json;

use hemtt_config::rapify::Derapify;

use crate::Error;

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Debin,
    Json,
    JsonPretty
}

impl OutputFormat {
    fn default_extension(&self) -> &str {
        match self {
            Self::Debin => "cpp",
            Self::JsonPretty | Self::Json => "json",
        }
    }
}

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
pub struct DerapifyArgs {
    /// file to derapify
    pub(crate) file: String,
    /// output format
    #[arg(short='f', long="format", default_value="debin")]
    pub(crate) output_format: OutputFormat,
    /// output file
    pub(crate) output: Option<String>,
}

/// Derapify a config file
pub fn derapify(path: &PathBuf, output: Option<&str>, format: OutputFormat) -> Result<(), Error> {
    let mut file = std::fs::File::open(path)?;
    let config = hemtt_config::Config::derapify(&mut file)?;
    let output = output.map_or_else(
        || {
            let mut path = path.clone();
            path.set_extension(format.default_extension());
            path
        },
        PathBuf::from,
    );
    let mut output = std::fs::File::create(output)?;
    match format {
        OutputFormat::Debin => output.write_all(config.to_string().as_bytes())?,
        OutputFormat::Json => {
            output.write_all(serde_json::to_string(&config)?.as_bytes())?;
        },
        OutputFormat::JsonPretty => {
            output.write_all(serde_json::to_string_pretty(&config)?.as_bytes())?;
        },
    }
    output.flush()?;
    Ok(())
}
