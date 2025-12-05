use serde_json;
use std::{io::Write as _, path::PathBuf};

use hemtt_config::rapify::Derapify;

use crate::Error;

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Cpp,
    Json,
    JsonPretty,
}

impl OutputFormat {
    pub const fn default_extension(&self) -> &str {
        match self {
            Self::Cpp => "cpp",
            Self::JsonPretty | Self::Json => "json",
        }
    }
}

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
/// Convert binary config files to readable text
pub struct DerapifyArgs {
    /// File to derapify (typically config.bin)
    pub(crate) file: String,
    /// Output format: cpp, json, or json-pretty
    #[arg(short = 'f', long = "format", default_value = "cpp")]
    pub(crate) output_format: OutputFormat,
    /// Output file path
    ///
    /// If not specified, uses the input filename with appropriate extension.
    pub(crate) output: Option<String>,
}

/// Derapify a config file
pub fn derapify(path: &PathBuf, output: Option<&str>, format: OutputFormat) -> Result<(), Error> {
    let mut file = fs_err::File::open(path)?;
    let config = hemtt_config::Config::derapify(&mut file)?;
    let output = output.map_or_else(
        || {
            let mut path = path.clone();
            path.set_extension(format.default_extension());
            path
        },
        PathBuf::from,
    );
    let _ = fs_err::create_dir_all(
        output
            .parent()
            .expect("Output file has no parent directory"),
    );
    let mut output = fs_err::File::create(output)?;
    match format {
        OutputFormat::Cpp => output.write_all(config.to_string().as_bytes())?,
        OutputFormat::Json => {
            output.write_all(serde_json::to_string(&config)?.as_bytes())?;
        }
        OutputFormat::JsonPretty => {
            output.write_all(serde_json::to_string_pretty(&config)?.as_bytes())?;
        }
    }
    output.flush()?;
    Ok(())
}
